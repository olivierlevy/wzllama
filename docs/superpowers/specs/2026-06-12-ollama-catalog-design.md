# Design Spec — Ollama Integrations Catalog

**Date:** 2026-06-12  
**Status:** Approved  
**Scope:** Automatically discover and add all tools from docs.ollama.com/integrations into wzllama's tool list.

---

## Problem

wzllama has a fixed list of ~14 hardcoded tools in `src/tools/`. The Ollama integrations page at `docs.ollama.com/integrations` lists ~20+ tools across 6 categories. New tools are added regularly. Without automation, wzllama becomes stale.

---

## Goal

Any tool listed on `docs.ollama.com/integrations` is automatically available in wzllama (menu, API, CLI), launchable via `ollama launch <slug>`, without requiring a code change.

---

## Approach: Versioned JSON Catalog + HTTP Refresh with 24h Cache

A JSON catalog file embedded in the binary serves as the offline seed. A background thread refreshes it from the web at startup if the cache is older than 24h. A CLI command forces an immediate refresh.

---

## Module Structure

```
src/tools/catalog/
  mod.rs              — CatalogEntry, ToolCatalog, load/save/merge logic
  catalog.json        — embedded seed catalog (committed to repo)
  ollama_native.rs    — OllamaNativeTool: impl Tool using catalog entries

src/core/catalog_refresh.rs   — CatalogRefresher: HTTP fetch, parse, cache write

~/.wzllama/
  catalog_cache.json  — runtime cache, TTL 24h

src/tools/mod.rs      — get_all_tools() merges static + catalog tools
src/cli.rs            — wzllama catalog refresh | list
```

---

## Data Model

### `CatalogEntry`

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct CatalogEntry {
    pub id: String,                          // unique tool id, e.g. "cline"
    pub name: String,                        // display name, e.g. "Cline CLI"
    pub slug: String,                        // ollama launch slug, e.g. "cline"
    pub category: ToolCategory,              // coding_agent | assistant | ide | chat_rag | automation | notebook
    pub install_cmd: Option<String>,         // e.g. "npm install -g cline"
    pub description_fallback: String,        // English fallback description
}
```

### `ToolCatalog`

```rust
pub struct ToolCatalog {
    pub version: String,       // ISO date string, e.g. "2026-06-12"
    pub tools: Vec<CatalogEntry>,
}

impl ToolCatalog {
    pub fn load() -> Self;     // cache (if fresh) → embedded catalog.json
    pub fn save_cache(&self);  // write to ~/.wzllama/catalog_cache.json
}
```

### Embedded seed catalog (`catalog.json`)

Contains all tools from `docs.ollama.com/integrations` at spec-write time, including existing static tools (for completeness), but static tools take precedence when merging.

---

## `OllamaNativeTool`

```rust
pub struct OllamaNativeTool {
    entry: CatalogEntry,
}

impl Tool for OllamaNativeTool {
    fn id(&self) -> &str { &self.entry.id }
    fn name(&self) -> &str { &self.entry.name }
    fn description(&self, _i18n: &I18n) -> String { self.entry.description_fallback.clone() }

    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        ToolStatus::from_installed(shell::is_installed_with_local_bin(&self.entry.slug))
    }

    fn install(&self, _i18n: &I18n) -> Result<()> {
        match &self.entry.install_cmd {
            Some(cmd) => shell::run_live(cmd),
            None => {
                display::info(&format!("Use: ollama launch {}", self.entry.slug));
                Ok(())
            }
        }
    }

    fn launch(&self, _i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        let cmd = match model {
            Some(m) => format!("ollama launch {} --model {}", self.entry.slug, m),
            None    => format!("ollama launch {}", self.entry.slug),
        };
        shell::exec(&cmd)
    }

    fn update(&self, _i18n: &I18n) -> Result<()> {
        // Re-run install_cmd if available, otherwise unsupported
        match &self.entry.install_cmd {
            Some(cmd) => shell::run_live(cmd),
            None => anyhow::bail!("Update not supported for {}", self.entry.name),
        }
    }
}
```

---

## `get_all_tools()` Merge Logic

```rust
pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    let mut tools = get_static_tools();           // existing hardcoded tools
    let catalog = ToolCatalog::load();
    for entry in catalog.tools {
        if !tools.iter().any(|t| t.id() == entry.id) {
            tools.push(Box::new(OllamaNativeTool::new(entry)));
        }
    }
    tools
}
```

Static tools are always listed first and never replaced by catalog entries.

---

## `CatalogRefresher`

```rust
pub struct CatalogRefresher;

impl CatalogRefresher {
    /// Non-blocking: spawns a background thread.
    /// Runs only if cache is absent or older than 24h.
    pub fn spawn_background_check();

    /// Blocking: used by `wzllama catalog refresh`.
    /// Displays progress. Always fetches, even if cache is fresh.
    pub fn force_refresh() -> Result<ToolCatalog>;

    /// Core: fetch + parse docs.ollama.com/integrations.
    /// For each tool, optionally fetch its individual page for install_cmd.
    fn fetch_and_parse() -> Result<Vec<CatalogEntry>>;
}
```

**Parsing strategy:**
1. GET `https://docs.ollama.com/integrations` (markdown-rendered)
2. Extract h2 headings → category names
3. Extract bullet links `/integrations/<slug>` → tool slugs + display names
4. For `force_refresh` only: GET each `/integrations/<slug>` page, extract first `npm install` or `winget install` block as `install_cmd`
5. Merge with existing catalog entries (preserve install_cmd if page has none)

**Error handling:**
- Network failure → `log::warn!`, silently use embedded catalog. No crash, no user-visible error.
- Parse failure on individual tool page → skip install_cmd, use None.
- Partial success → write whatever was successfully parsed to cache.

---

## Auto-Update of Installed Tools

### Trigger modes

| Mode | When | Behavior |
|------|------|----------|
| Background | At startup, if last update > 24h | Silent (no user output); errors logged only |
| Manual | `wzllama update-all` | Blocking, with progress per tool |

TTL stored in `~/.wzllama/last_update.txt` (ISO timestamp).

### Update logic per tool type

| Tool type | Update command |
|-----------|---------------|
| Static tool (implements `update()`) | Call `tool.update()` |
| OllamaNativeTool with `install_cmd` (npm) | `npm install -g <pkg>@latest` |
| OllamaNativeTool without `install_cmd` | `ollama launch <slug>` (Ollama handles it) |
| Ollama itself | `winget upgrade Ollama.Ollama` (Windows) / `curl \| sh` (Unix) |

### `ToolUpdater` (new `src/core/tool_updater.rs`)

```rust
pub struct ToolUpdater;

impl ToolUpdater {
    /// Non-blocking: spawns background thread if TTL expired.
    pub fn spawn_background_check(tools: Vec<...>, state: WzllamaState);

    /// Blocking: updates all installed tools, prints progress.
    pub fn update_all(tools: &[Box<dyn Tool>], state: &WzllamaState, i18n: &I18n) -> Result<()>;

    fn is_update_needed() -> bool;   // last_update.txt > 24h or absent
    fn mark_updated();               // write current timestamp
}
```

**Error handling:** If a tool update fails, log and continue. At the end of `update-all`, show summary: `3 updated, 1 failed, 2 skipped`.

### New API endpoint

```
POST /api/v1/tools/update-all    # Trigger update-all, returns { updated, failed, skipped }
```

---

## CLI Commands

```
wzllama catalog refresh     # Force-refresh catalog from docs.ollama.com, display results
wzllama catalog list        # List all catalog tools grouped by category
wzllama update-all          # Update all installed tools with progress
```

Added to `src/cli.rs`.

---

## Wizard Menu Integration

The tools menu displays catalog tools grouped by category after static tools:

```
Outils installés
  Claude Code ✅  Cline CLI ✅

Coding Agents
  Claude Code | Codex CLI | Cline CLI | Oh My Pi | ...

IDEs & Editors
  VS Code | Roo Code | JetBrains | Zed | ...

Assistants
  OpenClaw | Hermes Agent | Hermes Desktop

Chat/RAG / Automation / Notebooks
  Onyx | n8n | marimo
```

No changes to `InstalledTools` struct — status is always detected dynamically via `is_installed_with_local_bin(slug)`.

---

## API Impact

`GET /api/v1/tools` response adds a `"source"` field per tool:
- `"static"` — hardcoded tool in `src/tools/*.rs`
- `"catalog"` — discovered from ollama integrations catalog

---

## Seed Catalog Content (at spec time)

Tools to include in initial `catalog.json` (new tools not yet in wzllama):

| id | name | slug | category | install_cmd |
|----|------|------|----------|-------------|
| `cline` | Cline CLI | `cline` | coding_agent | `npm install -g cline` |
| `oh-my-pi` | Oh My Pi | `oh-my-pi` | coding_agent | — |
| `hermes-desktop` | Hermes Desktop | `hermes-desktop` | assistant | — |
| `vscode` | VS Code | `vscode` | ide | — |
| `cline-ide` | Cline (IDE) | `cline` | ide | — |
| `roo-code` | Roo Code | `roo-code` | ide | — |
| `jetbrains` | JetBrains | `jetbrains` | ide | — |
| `xcode` | Xcode | `xcode` | ide | — |
| `zed` | Zed | `zed` | ide | — |
| `onyx` | Onyx | `onyx` | chat_rag | — |
| `n8n` | n8n | `n8n` | automation | — |
| `marimo` | marimo | `marimo` | notebook | — |

Existing static tools are also reflected in the catalog for completeness but are skipped at merge time.

---

## Out of Scope

- i18n keys for catalog tools (use `description_fallback` directly; i18n can be added later)
- Install detection beyond `shell::is_installed_with_local_bin(slug)` (complex per-tool logic stays in static tools)
- Modifying `InstalledTools` struct in `state.json` for catalog tools

---

## Success Criteria

1. `cargo check --all-targets` passes with 0 errors
2. `wzllama catalog list` displays all tools from docs.ollama.com/integrations
3. `wzllama catalog refresh` fetches and updates the cache
4. New catalog tools appear in the wizard tool menu, launchable via `ollama launch <slug>`
5. Existing static tools are unaffected
6. Offline startup works (uses embedded catalog.json)
7. `wzllama update-all` updates all installed tools and shows a summary
8. Background auto-update runs silently at startup if last update > 24h
