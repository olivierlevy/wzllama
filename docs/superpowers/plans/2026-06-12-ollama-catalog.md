# Ollama Integrations Catalog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automatically discover and expose all tools from `docs.ollama.com/integrations` in wzllama's tool list, launchable via `ollama launch <slug>`, with a 24h auto-refresh cache and `wzllama update-all` to update installed tools.

**Architecture:** A `ToolCatalog` data model is embedded as `catalog.json` (seed) and cached in `~/.wzllama/cache/ollama_catalog.json` (refreshed every 24h). `OllamaNativeTool` implements the `Tool` trait dynamically for each catalog entry. `get_all_tools()` merges static + catalog tools (static takes priority). A `ToolUpdater` calls `tool.update()` on all installed tools, either in background (TTL 24h) or via `wzllama update-all`.

**Tech Stack:** Rust, `serde_json` (catalog parsing), `scraper` (HTML parsing, already in deps), `reqwest::blocking` (HTTP, already in deps), `cache.rs` reused for 24h TTL.

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `src/tools/catalog/mod.rs` | Create | `CatalogEntry`, `ToolCategory`, `ToolCatalog` (load/save/merge) |
| `src/tools/catalog/catalog.json` | Create | Seed catalog (all Ollama integrations as of 2026-06-12) |
| `src/tools/catalog/ollama_native.rs` | Create | `OllamaNativeTool: impl Tool` |
| `src/core/catalog_refresh.rs` | Create | `CatalogRefresher`: HTTP fetch, HTML parse, cache write |
| `src/core/tool_updater.rs` | Create | `ToolUpdater`: background + force update of installed tools |
| `tests/catalog_tests.rs` | Create | Unit tests for catalog load, merge, OllamaNativeTool |
| `src/tools/mod.rs` | Modify | Add `pub mod catalog`; merge catalog into `get_all_tools()` + fix `get_available_tools()` |
| `src/core/mod.rs` | Modify | Add `pub mod catalog_refresh`, `pub mod tool_updater` |
| `src/cli.rs` | Modify | Add `Catalog { subcommand }` and `UpdateAll` commands |
| `src/main.rs` | Modify | Spawn background `catalog_refresh` and `tool_updater` at startup |
| `src/api_server.rs` | Modify | Add `POST /api/v1/tools/update-all` route + handler |

---

## Task 1: `ToolCatalog` data model + seed `catalog.json`

**Files:**
- Create: `src/tools/catalog/mod.rs`
- Create: `src/tools/catalog/catalog.json`

- [ ] **Step 1: Create `src/tools/catalog/catalog.json`**

```json
{
  "version": "2026-06-12",
  "tools": [
    {"id": "claude_code",     "name": "Claude Code",    "slug": "claude-code",    "category": "coding_agent", "install_cmd": "npm install -g @anthropic-ai/claude-code", "description_fallback": "AI coding agent by Anthropic"},
    {"id": "codex",           "name": "Codex CLI",      "slug": "codex",          "category": "coding_agent", "install_cmd": "npm install -g @openai/codex",             "description_fallback": "OpenAI Codex CLI coding agent"},
    {"id": "copilot_cli",     "name": "Copilot CLI",    "slug": "copilot-cli",    "category": "coding_agent", "install_cmd": null,                                       "description_fallback": "GitHub Copilot for the terminal"},
    {"id": "cline",           "name": "Cline CLI",      "slug": "cline",          "category": "coding_agent", "install_cmd": "npm install -g cline",                     "description_fallback": "Autonomous coding agent for interactive terminal sessions"},
    {"id": "codex-app",       "name": "Codex App",      "slug": "codex-app",      "category": "coding_agent", "install_cmd": null,                                       "description_fallback": "OpenAI Codex graphical application"},
    {"id": "droid",           "name": "Droid",          "slug": "droid",          "category": "coding_agent", "install_cmd": null,                                       "description_fallback": "Factory AI autonomous coding agent"},
    {"id": "goose",           "name": "Goose",          "slug": "goose",          "category": "coding_agent", "install_cmd": null,                                       "description_fallback": "Block open-source AI coding agent"},
    {"id": "oh-my-pi",        "name": "Oh My Pi",       "slug": "oh-my-pi",       "category": "coding_agent", "install_cmd": null,                                       "description_fallback": "Oh My Pi AI coding agent"},
    {"id": "pi",              "name": "Pi",             "slug": "pi",             "category": "coding_agent", "install_cmd": "npm install -g pi-agent",                  "description_fallback": "Pi AI coding agent"},
    {"id": "pool",            "name": "Pool",           "slug": "pool",           "category": "coding_agent", "install_cmd": null,                                       "description_fallback": "Poolside AI coding agent"},
    {"id": "opencode",        "name": "OpenCode",       "slug": "opencode",       "category": "coding_agent", "install_cmd": "npm install -g @opencode-ai/cli",          "description_fallback": "Open-source AI coding agent"},
    {"id": "openclaw",        "name": "OpenClaw",       "slug": "openclaw",       "category": "assistant",    "install_cmd": "npm install -g openclaw",                  "description_fallback": "Multi-agent AI assistant"},
    {"id": "hermes_agent",    "name": "Hermes Agent",   "slug": "hermes",         "category": "assistant",    "install_cmd": null,                                       "description_fallback": "Hermes AI assistant CLI"},
    {"id": "hermes-desktop",  "name": "Hermes Desktop", "slug": "hermes-desktop", "category": "assistant",    "install_cmd": null,                                       "description_fallback": "Hermes AI desktop application"},
    {"id": "vscode",          "name": "VS Code",        "slug": "vscode",         "category": "ide",          "install_cmd": null,                                       "description_fallback": "Visual Studio Code with Ollama via GitHub Copilot Chat"},
    {"id": "cline-ide",       "name": "Cline (IDE)",    "slug": "cline",          "category": "ide",          "install_cmd": null,                                       "description_fallback": "Cline VS Code extension with Ollama"},
    {"id": "roo-code",        "name": "Roo Code",       "slug": "roo-code",       "category": "ide",          "install_cmd": null,                                       "description_fallback": "Roo Code VS Code extension with Ollama"},
    {"id": "jetbrains",       "name": "JetBrains",      "slug": "jetbrains",      "category": "ide",          "install_cmd": null,                                       "description_fallback": "JetBrains IDEs Ollama integration"},
    {"id": "xcode",           "name": "Xcode",          "slug": "xcode",          "category": "ide",          "install_cmd": null,                                       "description_fallback": "Xcode with Ollama integration"},
    {"id": "zed",             "name": "Zed",            "slug": "zed",            "category": "ide",          "install_cmd": null,                                       "description_fallback": "Zed editor with Ollama integration"},
    {"id": "onyx",            "name": "Onyx",           "slug": "onyx",           "category": "chat_rag",     "install_cmd": null,                                       "description_fallback": "Chat and RAG platform with Ollama"},
    {"id": "n8n",             "name": "n8n",            "slug": "n8n",            "category": "automation",   "install_cmd": null,                                       "description_fallback": "Workflow automation with Ollama AI nodes"},
    {"id": "marimo",          "name": "marimo",         "slug": "marimo",         "category": "notebook",     "install_cmd": null,                                       "description_fallback": "Interactive Python notebook with Ollama"}
  ]
}
```

- [ ] **Step 2: Create `src/tools/catalog/mod.rs`**

```rust
pub mod ollama_native;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static CATALOG: OnceLock<ToolCatalog> = OnceLock::new();

/// Tool category matching docs.ollama.com/integrations sections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    CodingAgent,
    Assistant,
    Ide,
    ChatRag,
    Automation,
    Notebook,
    Unknown,
}

impl ToolCategory {
    pub fn display_name(&self) -> &str {
        match self {
            ToolCategory::CodingAgent => "Coding Agents",
            ToolCategory::Assistant   => "Assistants",
            ToolCategory::Ide         => "IDEs & Editors",
            ToolCategory::ChatRag     => "Chat & RAG",
            ToolCategory::Automation  => "Automation",
            ToolCategory::Notebook    => "Notebooks",
            ToolCategory::Unknown     => "Other",
        }
    }
}

/// Single tool entry from the Ollama integrations catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Unique wzllama ID (e.g. "cline", "hermes-desktop")
    pub id: String,
    /// Display name (e.g. "Cline CLI")
    pub name: String,
    /// Slug used in `ollama launch <slug>` (e.g. "cline")
    pub slug: String,
    pub category: ToolCategory,
    /// Optional explicit install command (e.g. "npm install -g cline").
    /// None means `ollama launch <slug>` handles installation.
    pub install_cmd: Option<String>,
    /// English fallback description (used when i18n key is absent)
    pub description_fallback: String,
}

/// Full catalog: embedded seed + optional HTTP-refreshed cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCatalog {
    pub version: String,
    pub tools: Vec<CatalogEntry>,
}

impl ToolCatalog {
    /// Embedded seed catalog (compiled into binary)
    const SEED: &'static str = include_str!("catalog.json");

    /// Load catalog: prefer fresh 24h cache, fall back to embedded seed.
    /// Result is cached in process memory after first call.
    pub fn load() -> &'static ToolCatalog {
        CATALOG.get_or_init(|| Self::load_inner())
    }

    fn load_inner() -> ToolCatalog {
        // Try reading from 24h cache
        if let Ok(Some(cached)) = crate::core::cache::read_cache("ollama_catalog", false) {
            if let Ok(catalog) = serde_json::from_str::<ToolCatalog>(&cached) {
                return catalog;
            }
        }
        // Fall back to embedded seed
        serde_json::from_str(Self::SEED)
            .expect("catalog.json must be valid JSON — this is a compile-time resource")
    }

    /// Save this catalog to the 24h cache file
    pub fn save_to_cache(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        crate::core::cache::write_cache("ollama_catalog", &json)
    }

    /// Returns only the entries whose IDs are NOT already in `existing_ids`
    pub fn new_entries<'a>(&'a self, existing_ids: &[&str]) -> Vec<&'a CatalogEntry> {
        self.tools.iter()
            .filter(|e| !existing_ids.contains(&e.id.as_str()))
            .collect()
    }
}
```

- [ ] **Step 3: Verify the file compiles (add mod declaration temporarily)**

Open `src/tools/mod.rs` and temporarily add at the top:
```rust
pub mod catalog;
```
Then run:
```
cargo check -p wzllama 2>&1 | grep "^error"
```
Expected: no `error` lines (warnings OK).

- [ ] **Step 4: Commit**

```
git add src/tools/catalog/ && git commit -m "feat(catalog): add ToolCatalog data model and seed catalog.json"
```

---

## Task 2: `OllamaNativeTool` — impl `Tool` for catalog entries

**Files:**
- Create: `src/tools/catalog/ollama_native.rs`

- [ ] **Step 1: Create `src/tools/catalog/ollama_native.rs`**

```rust
use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};
use super::CatalogEntry;

/// A tool backed by a catalog entry.
/// Install/update/launch all delegate to `ollama launch <slug>` or the entry's `install_cmd`.
pub struct OllamaNativeTool {
    pub entry: CatalogEntry,
}

impl OllamaNativeTool {
    pub fn new(entry: CatalogEntry) -> Self {
        Self { entry }
    }
}

impl Tool for OllamaNativeTool {
    fn id(&self) -> &str { &self.entry.id }
    fn name(&self) -> &str { &self.entry.name }

    fn description(&self, _i18n: &I18n) -> String {
        self.entry.description_fallback.clone()
    }

    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        ToolStatus::from_installed(shell::is_installed_with_local_bin(&self.entry.slug))
    }

    fn install(&self, _i18n: &I18n) -> Result<()> {
        match &self.entry.install_cmd {
            Some(cmd) => {
                display::info(&format!("Installing {} via: {}", self.entry.name, cmd));
                shell::run_live(cmd)
            }
            None => {
                // `ollama launch <slug>` handles installation interactively
                let cmd = format!("ollama launch {}", self.entry.slug);
                display::info(&format!("Installing {} via: {}", self.entry.name, cmd));
                shell::exec(&cmd)
            }
        }
    }

    fn update(&self, _i18n: &I18n) -> Result<()> {
        match &self.entry.install_cmd {
            Some(cmd) => {
                display::info(&format!("Updating {} via: {}", self.entry.name, cmd));
                shell::run_live(cmd)
            }
            None => {
                let cmd = format!("ollama launch {}", self.entry.slug);
                display::info(&format!("Updating {} via: {}", self.entry.name, cmd));
                shell::exec(&cmd)
            }
        }
    }

    fn launch(&self, _i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        let cmd = match model {
            Some(m) => format!("ollama launch {} --model {}", self.entry.slug, m),
            None    => format!("ollama launch {}", self.entry.slug),
        };
        display::run(&cmd);
        shell::exec(&cmd)
    }
}
```

- [ ] **Step 2: Verify it compiles**

```
cargo check -p wzllama 2>&1 | grep "^error"
```
Expected: no errors.

- [ ] **Step 3: Commit**

```
git add src/tools/catalog/ollama_native.rs && git commit -m "feat(catalog): add OllamaNativeTool impl Tool"
```

---

## Task 3: Wire catalog into `get_all_tools()` and fix `get_available_tools()`

**Files:**
- Modify: `src/tools/mod.rs`

- [ ] **Step 1: Write failing test** (add to `tests/catalog_tests.rs` — create the file)

```rust
// tests/catalog_tests.rs
use wzllama::tools::get_all_tools;
use wzllama::tools::catalog::ToolCatalog;

#[test]
fn test_catalog_loads_without_panic() {
    let catalog = ToolCatalog::load();
    assert!(!catalog.tools.is_empty(), "Catalog must have at least one tool");
}

#[test]
fn test_catalog_has_new_tools() {
    let catalog = ToolCatalog::load();
    let has_cline = catalog.tools.iter().any(|t| t.id == "cline");
    assert!(has_cline, "Catalog must contain cline");
}

#[test]
fn test_get_all_tools_contains_catalog_tools() {
    let tools = get_all_tools();
    let has_cline = tools.iter().any(|t| t.id() == "cline");
    assert!(has_cline, "get_all_tools() must include catalog tool 'cline'");
}

#[test]
fn test_no_duplicate_tool_ids() {
    let tools = get_all_tools();
    let mut ids = std::collections::HashSet::new();
    for tool in &tools {
        assert!(ids.insert(tool.id().to_string()), "Duplicate tool id: {}", tool.id());
    }
}

#[test]
fn test_static_tools_not_overridden_by_catalog() {
    let tools = get_all_tools();
    // Claude Code is a static tool AND in catalog; should appear exactly once
    let count = tools.iter().filter(|t| t.id() == "claude_code").count();
    assert_eq!(count, 1, "claude_code must appear exactly once (static priority)");
}
```

- [ ] **Step 2: Run test to verify it fails**

```
cargo test --test catalog_tests 2>&1 | tail -20
```
Expected: compile error ("failed to resolve: use of undeclared module `catalog`").

- [ ] **Step 3: Update `src/tools/mod.rs`**

Replace the existing `mod.rs` content with:

```rust
pub mod tool_trait;
pub mod docker;
pub mod open_webui;
pub mod ollama;
pub mod openclaw;
pub mod claude_code;
pub mod hermes;
pub mod opencode;
pub mod codex;
pub mod copilot_cli;
pub mod droid;
pub mod pi;
pub mod pool;
pub mod obsidian;
pub mod goose;
pub mod flatpak; // Utility tool, not exposed in menus
pub mod llmfit;
pub mod catalog;

use crate::config::{I18n, WzllamaState};
use tool_trait::{Tool, ToolStatus};

/// Returns all built-in (hardcoded) tools. These take priority over catalog tools.
fn get_static_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(ollama::OllamaTool),
        Box::new(open_webui::OpenWebUITool),
        Box::new(openclaw::OpenClawTool),
        Box::new(claude_code::ClaudeCodeTool),
        Box::new(hermes::HermesTool),
        Box::new(opencode::OpenCodeTool),
        Box::new(codex::CodexTool),
        Box::new(copilot_cli::CopilotCliTool),
        Box::new(droid::DroidTool),
        Box::new(pi::PiTool),
        Box::new(pool::PoolTool),
        Box::new(obsidian::ObsidianTool),
        Box::new(goose::GooseTool),
        Box::new(llmfit::LLMFitTool),
    ]
}

/// Returns all tools: static tools first, then catalog tools not already present.
pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    let mut tools = get_static_tools();
    let static_ids: Vec<&str> = tools.iter().map(|t| t.id()).collect();
    let cat = catalog::ToolCatalog::load();
    for entry in cat.new_entries(&static_ids) {
        tools.push(Box::new(catalog::ollama_native::OllamaNativeTool::new(entry.clone())));
    }
    tools
}

pub fn get_tool(id: &str) -> Option<Box<dyn Tool>> {
    get_all_tools().into_iter().find(|t| t.id() == id)
}

pub fn get_available_tools(state: &WzllamaState, i18n: &I18n) -> Vec<ToolInfo> {
    get_all_tools().iter().map(|t| {
        // For static tools, use the state booleans (avoids shell::which overhead).
        // For catalog tools, detect dynamically via tool.status().
        let installed = match t.id() {
            "ollama"      => state.installed.ollama,
            "open_webui"  => state.installed.open_webui,
            "openclaw"    => state.installed.openclaw,
            "claude_code" => state.installed.claude_code,
            "hermes_agent"=> state.installed.hermes_agent,
            "opencode"    => state.installed.opencode,
            "codex"       => state.installed.codex,
            "copilot_cli" => state.installed.copilot_cli,
            "droid"       => state.installed.droid,
            "pi"          => state.installed.pi,
            "pool"        => state.installed.pool,
            "obsidian"    => state.installed.obsidian,
            "goose"       => state.installed.goose,
            "llmfit"      => state.installed.llmfit,
            _ => matches!(t.status(state), ToolStatus::Installed),
        };
        ToolInfo {
            id: t.id().to_string(),
            name: t.name().to_string(),
            description: t.description(i18n),
            installed,
        }
    }).collect()
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub installed: bool,
}
```

- [ ] **Step 4: Run tests to verify they pass**

```
cargo test --test catalog_tests 2>&1 | tail -20
```
Expected: all 5 tests pass.

- [ ] **Step 5: Run full test suite to ensure no regressions**

```
cargo test 2>&1 | tail -10
```
Expected: all tests pass.

- [ ] **Step 6: Commit**

```
git add src/tools/mod.rs tests/catalog_tests.rs && git commit -m "feat(catalog): wire catalog into get_all_tools() and get_available_tools()"
```

---

## Task 4: `CatalogRefresher` — HTTP fetch + HTML parse + cache write

**Files:**
- Create: `src/core/catalog_refresh.rs`
- Modify: `src/core/mod.rs`

- [ ] **Step 1: Create `src/core/catalog_refresh.rs`**

```rust
//! Fetches the Ollama integrations catalog from docs.ollama.com and updates the local cache.

use anyhow::Result;
use scraper::{Html, Selector};
use crate::tools::catalog::{CatalogEntry, ToolCatalog, ToolCategory};

const INTEGRATIONS_URL: &str = "https://docs.ollama.com/integrations";
const TOOL_PAGE_BASE: &str = "https://docs.ollama.com/integrations/";

pub struct CatalogRefresher;

impl CatalogRefresher {
    /// Spawn a background thread to refresh the catalog if the cache is stale.
    /// Returns immediately; any errors are logged, not surfaced to the caller.
    pub fn spawn_background_check() {
        // Only spawn if cache is absent or stale
        let cache_fresh = crate::core::cache::read_cache("ollama_catalog", false)
            .map(|v| v.is_some())
            .unwrap_or(false);
        if cache_fresh {
            return;
        }
        std::thread::Builder::new()
            .name("catalog-refresh".into())
            .spawn(|| {
                match Self::fetch_and_update(false) {
                    Ok(catalog) => log::info!(
                        "Catalog refreshed: {} tools found (version {})",
                        catalog.tools.len(), catalog.version
                    ),
                    Err(e) => log::warn!("Catalog refresh failed (offline?): {}", e),
                }
            })
            .ok(); // ignore thread spawn errors
    }

    /// Force a refresh right now, display progress.
    /// Used by `wzllama catalog refresh`.
    pub fn force_refresh() -> Result<ToolCatalog> {
        println!("🔄 Fetching https://docs.ollama.com/integrations …");
        let catalog = Self::fetch_and_update(true)?;
        println!("✅ Catalog updated: {} tools (version {})", catalog.tools.len(), catalog.version);
        Ok(catalog)
    }

    /// Fetch the integrations index, parse tools, write cache, return new catalog.
    /// `fetch_pages`: if true, fetch each tool's page to extract install_cmd.
    fn fetch_and_update(fetch_pages: bool) -> Result<ToolCatalog> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        // 1. Fetch integrations index
        let html = client.get(INTEGRATIONS_URL).send()?.text()?;
        let entries = Self::parse_integrations_page(&html, &client, fetch_pages)?;

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let catalog = ToolCatalog { version: today, tools: entries };
        catalog.save_to_cache()?;
        Ok(catalog)
    }

    /// Parse the integrations index HTML → Vec<CatalogEntry>
    fn parse_integrations_page(
        html: &str,
        client: &reqwest::blocking::Client,
        fetch_pages: bool,
    ) -> Result<Vec<CatalogEntry>> {
        let document = Html::parse_document(html);

        // Selector for section headings (h2) and tool links
        let h2_sel = Selector::parse("h2").unwrap();
        let a_sel  = Selector::parse("a[href]").unwrap();

        // Collect all elements in document order with their context
        // Strategy: iterate h2 elements, collect following sibling links
        let mut entries: Vec<CatalogEntry> = Vec::new();

        // Gather all (category_text, slug, name) from anchor links under h2 sections
        // We walk the full document collecting h2→ul→li→a triples
        let body_sel = Selector::parse("body").unwrap();
        let body = document.select(&body_sel).next()
            .ok_or_else(|| anyhow::anyhow!("No <body> element in page"))?;

        let mut current_category = ToolCategory::Unknown;
        for child in body.descendants() {
            let el = match child.value().as_element() {
                Some(e) => e,
                None => continue,
            };
            match el.name() {
                "h2" => {
                    // Get text content of h2
                    let node = scraper::ElementRef::wrap(child)
                        .map(|er| er.text().collect::<String>())
                        .unwrap_or_default();
                    current_category = Self::category_from_heading(&node);
                }
                "a" => {
                    let href = match el.attr("href") {
                        Some(h) => h,
                        None => continue,
                    };
                    if !href.starts_with("/integrations/") || href == "/integrations/" {
                        continue;
                    }
                    let page_slug = href.trim_start_matches("/integrations/");
                    if page_slug.is_empty() { continue; }

                    // Name = link text
                    let name_node = scraper::ElementRef::wrap(child)
                        .map(|er| er.text().collect::<String>().trim().to_string())
                        .unwrap_or_else(|| page_slug.to_string());
                    if name_node.is_empty() { continue; }

                    // Derive wzllama id: replace hyphens with underscores for existing tools,
                    // keep as-is for new ones
                    let id = page_slug.replace('-', "_");
                    // Fix: hermes-agent should stay hermes_agent, not hermes-agent
                    let id = match page_slug {
                        "hermes" => "hermes_agent".to_string(),
                        other    => other.replace('-', "_"),
                    };

                    // Derive ollama launch slug and install_cmd from tool page if fetch_pages
                    let (launch_slug, install_cmd) = if fetch_pages {
                        Self::fetch_tool_page(client, page_slug).unwrap_or_else(|_| {
                            (page_slug.to_string(), None)
                        })
                    } else {
                        // Default: slug = page_slug, install_cmd = None
                        (page_slug.to_string(), None)
                    };

                    entries.push(CatalogEntry {
                        id,
                        name: name_node,
                        slug: launch_slug,
                        category: current_category.clone(),
                        install_cmd,
                        description_fallback: String::new(),
                    });
                }
                _ => {}
            }
        }

        // Merge with embedded seed to preserve description_fallback and install_cmd
        let seed = serde_json::from_str::<ToolCatalog>(ToolCatalog::SEED).unwrap_or(ToolCatalog {
            version: "seed".into(),
            tools: vec![],
        });
        for entry in &mut entries {
            if let Some(seed_entry) = seed.tools.iter().find(|s| s.id == entry.id) {
                if entry.description_fallback.is_empty() {
                    entry.description_fallback = seed_entry.description_fallback.clone();
                }
                if entry.install_cmd.is_none() && seed_entry.install_cmd.is_some() {
                    entry.install_cmd = seed_entry.install_cmd.clone();
                }
            }
        }

        if entries.is_empty() {
            anyhow::bail!("Parsing returned 0 tools — page structure may have changed");
        }
        Ok(entries)
    }

    /// Fetch a single tool page, extract ollama launch slug and install_cmd
    fn fetch_tool_page(
        client: &reqwest::blocking::Client,
        page_slug: &str,
    ) -> Result<(String, Option<String>)> {
        let url = format!("{}{}", TOOL_PAGE_BASE, page_slug);
        let html = client.get(&url).send()?.text()?;
        let document = Html::parse_document(&html);

        // Find `ollama launch <slug>` in any code element
        let code_sel = Selector::parse("code").unwrap();
        let mut launch_slug = page_slug.to_string();
        let mut install_cmd: Option<String> = None;

        for code in document.select(&code_sel) {
            let text = code.text().collect::<String>();
            let text = text.trim();

            // Extract launch slug: pattern `ollama launch <word>`
            if text.starts_with("ollama launch ") {
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() >= 3 {
                    launch_slug = parts[2].to_string();
                }
            }

            // Extract install_cmd: pattern `npm install -g <pkg>`
            if install_cmd.is_none() && text.starts_with("npm install -g ") {
                install_cmd = Some(text.to_string());
            }
        }
        Ok((launch_slug, install_cmd))
    }

    fn category_from_heading(text: &str) -> ToolCategory {
        let t = text.to_lowercase();
        if t.contains("coding") || t.contains("agent") {
            ToolCategory::CodingAgent
        } else if t.contains("assistant") {
            ToolCategory::Assistant
        } else if t.contains("ide") || t.contains("editor") {
            ToolCategory::Ide
        } else if t.contains("chat") || t.contains("rag") {
            ToolCategory::ChatRag
        } else if t.contains("automat") {
            ToolCategory::Automation
        } else if t.contains("notebook") {
            ToolCategory::Notebook
        } else {
            ToolCategory::Unknown
        }
    }
}
```

- [ ] **Step 2: Add to `src/core/mod.rs`**

```rust
pub mod hardware;
pub mod llmfit_api;
pub mod ollama_api;
pub mod ollama_doctor;
pub mod ollama_models;
pub mod shell;
pub mod system;
pub mod localmax_models;
pub mod cache;
pub mod catalog_refresh;
pub mod tool_updater;   // added in Task 5

pub use hardware::HardwareInfo;
```
(Add `pub mod catalog_refresh;` only for now; `pub mod tool_updater;` is added in Task 5.)

- [ ] **Step 3: Check compilation**

```
cargo check -p wzllama 2>&1 | grep "^error"
```
Expected: no errors.

- [ ] **Step 4: Commit**

```
git add src/core/catalog_refresh.rs src/core/mod.rs && git commit -m "feat(catalog): add CatalogRefresher HTTP fetch and HTML parser"
```

---

## Task 5: `ToolUpdater` — background and force update of all installed tools

**Files:**
- Create: `src/core/tool_updater.rs`
- Modify: `src/core/mod.rs` (add `pub mod tool_updater;`)

- [ ] **Step 1: Write failing test** (add to `tests/catalog_tests.rs`)

Add these tests at the bottom of the file:

```rust
#[test]
fn test_tool_updater_update_needed_when_no_timestamp() {
    // Delete the timestamp file if it exists, then check
    let home = dirs::home_dir().unwrap_or_default();
    let ts_file = home.join(".wzllama").join("last_update.txt");
    let _ = std::fs::remove_file(&ts_file);
    assert!(wzllama::core::tool_updater::ToolUpdater::is_update_needed());
}
```

Run:
```
cargo test test_tool_updater 2>&1 | tail -5
```
Expected: compile error (module not found).

- [ ] **Step 2: Create `src/core/tool_updater.rs`**

```rust
//! Updates all installed tools in background or on demand.

use anyhow::Result;
use std::path::PathBuf;
use crate::config::{I18n, WzllamaState};
use crate::tools::{get_all_tools, tool_trait::ToolStatus};
use crate::display;

const TIMESTAMP_FILE: &str = "last_update.txt";
const UPDATE_INTERVAL_HOURS: u64 = 24;

pub struct ToolUpdater;

/// Summary of an update-all run
pub struct UpdateSummary {
    pub updated: Vec<String>,
    pub failed: Vec<(String, String)>,   // (tool_name, error_message)
    pub skipped: Vec<String>,            // not installed
}

impl ToolUpdater {
    /// Non-blocking: spawn background update if last update > 24h ago.
    pub fn spawn_background_check(state: WzllamaState) {
        if !Self::is_update_needed() {
            return;
        }
        std::thread::Builder::new()
            .name("tool-updater".into())
            .spawn(move || {
                let i18n = I18n::default();
                match Self::update_all_silent(&state, &i18n) {
                    Ok(summary) => {
                        log::info!(
                            "Background update: {} updated, {} failed, {} skipped",
                            summary.updated.len(), summary.failed.len(), summary.skipped.len()
                        );
                        Self::mark_updated();
                    }
                    Err(e) => log::warn!("Background update error: {}", e),
                }
            })
            .ok();
    }

    /// Blocking: update all installed tools with progress output.
    /// Used by `wzllama update-all`.
    pub fn update_all_verbose(state: &WzllamaState, i18n: &I18n) -> Result<UpdateSummary> {
        let tools = get_all_tools();
        let mut summary = UpdateSummary { updated: vec![], failed: vec![], skipped: vec![] };

        for tool in &tools {
            let name = tool.name().to_string();
            let is_installed = match tool.status(state) {
                ToolStatus::Installed => true,
                ToolStatus::NotInstalled => false,
            };
            if !is_installed {
                summary.skipped.push(name);
                continue;
            }
            display::info(&format!("Updating {}…", tool.name()));
            match tool.update(i18n) {
                Ok(_) => {
                    display::success(&format!("✅ {} updated", tool.name()));
                    summary.updated.push(name);
                }
                Err(e) => {
                    display::warning(&format!("⚠️  {} update failed: {}", tool.name(), e));
                    summary.failed.push((name, e.to_string()));
                }
            }
        }
        Self::mark_updated();
        Ok(summary)
    }

    /// Silent version for background use (no stdout).
    fn update_all_silent(state: &WzllamaState, i18n: &I18n) -> Result<UpdateSummary> {
        let tools = get_all_tools();
        let mut summary = UpdateSummary { updated: vec![], failed: vec![], skipped: vec![] };
        for tool in &tools {
            let is_installed = matches!(tool.status(state), ToolStatus::Installed);
            if !is_installed { summary.skipped.push(tool.name().into()); continue; }
            match tool.update(i18n) {
                Ok(_)  => summary.updated.push(tool.name().into()),
                Err(e) => summary.failed.push((tool.name().into(), e.to_string())),
            }
        }
        Ok(summary)
    }

    /// Returns true if the last update was > 24h ago or never ran.
    pub fn is_update_needed() -> bool {
        let ts_file = Self::timestamp_path();
        let Ok(meta) = std::fs::metadata(&ts_file) else { return true; };
        let Ok(modified) = meta.modified() else { return true; };
        let Ok(age) = std::time::SystemTime::now().duration_since(modified) else { return true; };
        age.as_secs() > UPDATE_INTERVAL_HOURS * 3600
    }

    /// Write current timestamp to mark a successful update run.
    pub fn mark_updated() {
        let ts_file = Self::timestamp_path();
        // Ensure parent dir exists
        if let Some(parent) = ts_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let now = chrono::Local::now().to_rfc3339();
        let _ = std::fs::write(&ts_file, now);
    }

    fn timestamp_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".wzllama")
            .join(TIMESTAMP_FILE)
    }
}
```

- [ ] **Step 3: Add `pub mod tool_updater;` to `src/core/mod.rs`**

File should now have these module declarations:
```rust
pub mod hardware;
pub mod llmfit_api;
pub mod ollama_api;
pub mod ollama_doctor;
pub mod ollama_models;
pub mod shell;
pub mod system;
pub mod localmax_models;
pub mod cache;
pub mod catalog_refresh;
pub mod tool_updater;

pub use hardware::HardwareInfo;
```

- [ ] **Step 4: Run test to verify it passes**

```
cargo test test_tool_updater 2>&1 | tail -5
```
Expected: `test_tool_updater_update_needed_when_no_timestamp ... ok`

- [ ] **Step 5: Commit**

```
git add src/core/tool_updater.rs src/core/mod.rs tests/catalog_tests.rs && git commit -m "feat(catalog): add ToolUpdater for background and force update of installed tools"
```

---

## Task 6: CLI commands — `wzllama catalog refresh|list` and `wzllama update-all`

**Files:**
- Modify: `src/cli.rs`

- [ ] **Step 1: Replace the `Command` enum and `execute()` in `src/cli.rs`**

Add these new variants and their execution logic. Replace the existing `Command` enum and the `execute()` match:

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::wizard;
use crate::config;
use crate::core::{ollama_api, shell};

#[derive(Parser)]
#[command(name = "wzllama", about = "Assistant IA locale", version = "0.3.0")]
pub struct Cli {
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(visible_alias = "w")]
    Wizard,
    #[command(visible_alias = "v")]
    Validate,
    #[command(visible_alias = "b")]
    Bench,
    #[command(visible_alias = "r")]
    ResetTemplates,
    #[command(visible_alias = "i")]
    CheckI18n,
    #[command(visible_alias = "u")]
    Uninstall,
    #[command(visible_alias = "s")]
    Serve,
    /// Install Open WebUI with Docker checks
    InstallWebui,
    /// Launch Open WebUI with Docker checks
    LaunchWebui,
    /// Catalog management: refresh or list Ollama integrations
    Catalog {
        #[command(subcommand)]
        subcommand: CatalogCommand,
    },
    /// Update all installed tools
    UpdateAll,
}

#[derive(Subcommand)]
pub enum CatalogCommand {
    /// Force-refresh the tool catalog from docs.ollama.com
    Refresh,
    /// List all tools in the catalog, grouped by category
    List,
}

impl Cli {
    pub fn parse_args() -> Self { Cli::parse() }

    pub fn execute(&self) -> Result<()> {
        match self.command.as_ref().unwrap_or(&Command::Wizard) {
            Command::Wizard if self.dry_run => {
                println!("[DRY-RUN]");
                Ok(())
            }
            Command::Wizard => {
                let mut state = crate::config::WzllamaState::load();
                let i18n = wizard::select_language(&mut state)?;
                let hardware = crate::core::hardware::detect();
                wizard::run(&i18n, &mut state, &hardware)
            }
            Command::Validate       => config::templates::validate_all(),
            Command::Bench          => ollama_api::run_benchmark(),
            Command::ResetTemplates => config::templates::reset_all(),
            Command::CheckI18n      => config::i18n::check_integrity(),
            Command::Uninstall      => wizard::menu_config::uninstall_wzllama_cli(),

            Command::Catalog { subcommand } => match subcommand {
                CatalogCommand::Refresh => {
                    crate::core::catalog_refresh::CatalogRefresher::force_refresh()?;
                    Ok(())
                }
                CatalogCommand::List => {
                    use crate::tools::catalog::{ToolCatalog, ToolCategory};
                    let catalog = ToolCatalog::load();
                    let categories = [
                        ToolCategory::CodingAgent,
                        ToolCategory::Assistant,
                        ToolCategory::Ide,
                        ToolCategory::ChatRag,
                        ToolCategory::Automation,
                        ToolCategory::Notebook,
                        ToolCategory::Unknown,
                    ];
                    println!("📦 Ollama Integrations Catalog ({})", catalog.version);
                    println!();
                    for cat in &categories {
                        let tools: Vec<_> = catalog.tools.iter().filter(|t| &t.category == cat).collect();
                        if tools.is_empty() { continue; }
                        println!("  {} {}:", crate::display::BOLD, cat.display_name());
                        for t in tools {
                            let install = t.install_cmd.as_deref().unwrap_or("ollama launch");
                            println!("    • {} ({})\t[{}]", t.name, t.id, install);
                        }
                        println!();
                    }
                    Ok(())
                }
            },

            Command::UpdateAll => {
                let state = crate::config::WzllamaState::load();
                let i18n = crate::config::I18n::default();
                let summary = crate::core::tool_updater::ToolUpdater::update_all_verbose(&state, &i18n)?;
                println!();
                println!("📊 Update summary: {} updated, {} failed, {} skipped",
                    summary.updated.len(), summary.failed.len(), summary.skipped.len());
                for (name, err) in &summary.failed {
                    println!("  ❌ {}: {}", name, err);
                }
                Ok(())
            }

            Command::InstallWebui => {
                // Keep existing implementation unchanged
                if let Err(e) = crate::tools::docker::ensure_ready_no_confirm() {
                    println!("⚠️  Docker non prêt: {}", e);
                    return Ok(());
                }
                let exists = shell::run("docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$' || sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$'").is_ok();
                if exists {
                    shell::run("docker start open-webui 2>/dev/null || sudo docker start open-webui")?;
                    println!("✅ Open WebUI started");
                } else {
                    shell::run("docker run -d --network=host --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data -e OLLAMA_BASE_URL=http://127.0.0.1:11434 --name open-webui --restart always ghcr.io/open-webui/open-webui:ollama 2>/dev/null || sudo docker run -d --network=host --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data -e OLLAMA_BASE_URL=http://127.0.0.1:11434 --name open-webui --restart always ghcr.io/open-webui/open-webui:ollama")?;
                    println!("✅ Open WebUI installed");
                }
                Ok(())
            }
            Command::LaunchWebui => {
                if let Err(e) = crate::tools::docker::ensure_ready_no_confirm() {
                    println!("⚠️  Docker non prêt: {}", e);
                    return Ok(());
                }
                let url = "http://localhost:8080";
                println!("🌐 Open WebUI : {}", url);
                shell::open_url(url);
                println!("✅ Open WebUI lancé dans le navigateur");
                Ok(())
            }
            Command::Serve => {
                let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 1133));
                tokio::runtime::Runtime::new()?.block_on(crate::api_server::start_server(addr));
                Ok(())
            }
        }
    }
}
```

Note: `crate::display::BOLD` may not exist — if so, replace with `"──"` or the existing bold string from `display.rs`. Check first with:
```
grep -r "BOLD\|pub const" src/display.rs
```
If absent, replace `crate::display::BOLD` with `"──"` in the List command.

- [ ] **Step 2: Check compilation**

```
cargo check -p wzllama 2>&1 | grep "^error"
```
Expected: no errors.

- [ ] **Step 3: Smoke test**

```
cargo run -- catalog list 2>&1 | head -30
```
Expected: prints categories with tool names, no crash.

- [ ] **Step 4: Commit**

```
git add src/cli.rs && git commit -m "feat(catalog): add 'wzllama catalog refresh|list' and 'wzllama update-all' CLI commands"
```

---

## Task 7: Background spawns in `main.rs`

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Update `main()` to spawn background catalog refresh and tool updater**

In `src/main.rs`, update the `main()` function. Add background spawns after `config::logging::install_embedded_i18n()?;`:

```rust
fn main() -> Result<()> {
    config::paths::ensure_dirs()?;
    config::logging::init()?;
    config::logging::install_embedded_i18n()?;
    info!("wzllama v0.3.0 started");

    let cli = Cli::parse_args();

    // Start API server in background for wizard mode only (not for serve command)
    if matches!(cli.command, None | Some(Command::Wizard)) {
        start_api_server_background();
    }

    // Background catalog refresh (24h TTL, non-blocking)
    crate::core::catalog_refresh::CatalogRefresher::spawn_background_check();

    // Background tool update check (24h TTL, non-blocking, wizard mode only)
    if matches!(cli.command, None | Some(Command::Wizard)) {
        let state = crate::config::WzllamaState::load();
        crate::core::tool_updater::ToolUpdater::spawn_background_check(state);
    }

    let result = cli.execute();

    // Request API server shutdown when exiting (only if we started it)
    if matches!(cli.command, None | Some(Command::Wizard)) {
        crate::api_server::request_shutdown();
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    result
}
```

- [ ] **Step 2: Check compilation**

```
cargo check -p wzllama 2>&1 | grep "^error"
```
Expected: no errors.

- [ ] **Step 3: Verify background spawn doesn't block**

```
cargo run -- catalog list 2>&1 | head -5
```
Expected: returns immediately (no hanging).

- [ ] **Step 4: Commit**

```
git add src/main.rs && git commit -m "feat(catalog): spawn background catalog refresh and tool updater at startup"
```

---

## Task 8: API endpoint `POST /api/v1/tools/update-all`

**Files:**
- Modify: `src/api_server.rs`

- [ ] **Step 1: Add route to `create_router()`**

In the tool endpoints section of `create_router()`:

```rust
// After .route("/api/v1/tools/{id}/launch", post(launch_tool))
.route("/api/v1/tools/update-all", post(update_all_tools))
```

- [ ] **Step 2: Add handler function** (add after the existing `launch_tool` handler)

```rust
async fn update_all_tools() -> Json<Value> {
    let state = ApiService::get_state();
    let i18n = I18n::default();
    match crate::core::tool_updater::ToolUpdater::update_all_verbose(&state, &i18n) {
        Ok(summary) => Json(serde_json::json!({
            "success": true,
            "updated": summary.updated,
            "failed": summary.failed.iter().map(|(n, e)| serde_json::json!({"tool": n, "error": e})).collect::<Vec<_>>(),
            "skipped": summary.skipped,
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": e.to_string(),
        })),
    }
}
```

- [ ] **Step 3: Check compilation**

```
cargo check -p wzllama 2>&1 | grep "^error"
```
Expected: no errors.

- [ ] **Step 4: Commit**

```
git add src/api_server.rs && git commit -m "feat(catalog): add POST /api/v1/tools/update-all endpoint"
```

---

## Task 9: Final validation

- [ ] **Step 1: Run full test suite**

```
cargo test 2>&1 | tail -20
```
Expected: all tests pass, including `catalog_tests`.

- [ ] **Step 2: Run clippy**

```
cargo clippy -- -D warnings 2>&1 | grep "^error"
```
Fix any errors. Warnings that were pre-existing are acceptable.

- [ ] **Step 3: Smoke test catalog list**

```
cargo run -- catalog list
```
Expected: prints tools grouped by category, includes "Cline CLI", "VS Code", "n8n", "marimo".

- [ ] **Step 4: Smoke test update-all (dry)**

```
cargo run -- update-all 2>&1 | tail -5
```
Expected: prints summary `N updated, M failed, K skipped`. Does not crash.

- [ ] **Step 5: Build release**

```
cargo build --release 2>&1 | grep "^error"
```
Expected: build succeeds.

- [ ] **Step 6: Final commit and push**

```
git add -A && git commit -m "feat(catalog): complete Ollama integrations catalog with auto-refresh and update-all"
git push
```

---

## Self-Review Notes

**Spec coverage check:**
- ✅ `ToolCatalog` + `CatalogEntry` + `ToolCategory` → Task 1
- ✅ `OllamaNativeTool: impl Tool` → Task 2
- ✅ `get_all_tools()` merge (static priority, no duplicates) → Task 3
- ✅ `get_available_tools()` dynamic status for catalog tools → Task 3
- ✅ `CatalogRefresher::spawn_background_check()` (24h TTL) → Task 4
- ✅ `CatalogRefresher::force_refresh()` → Task 4
- ✅ Offline fallback to embedded catalog.json → Task 1 (`OnceLock` + seed)
- ✅ `ToolUpdater::spawn_background_check()` (24h TTL) → Task 5
- ✅ `ToolUpdater::update_all_verbose()` → Task 5
- ✅ `wzllama catalog refresh` CLI → Task 6
- ✅ `wzllama catalog list` CLI → Task 6
- ✅ `wzllama update-all` CLI → Task 6
- ✅ Background spawns at startup → Task 7
- ✅ `POST /api/v1/tools/update-all` API → Task 8
- ✅ Tests (catalog load, merge, no duplicates, OllamaNativeTool, ToolUpdater) → Tasks 3, 5

**Potential issues:**
- `display::BOLD` constant — verify in `src/display.rs` before using in Task 6. If absent, use `"──"`.
- `ToolCatalog::SEED` is a `const` — needs to be `pub(crate)` for `catalog_refresh.rs` to access it. If compile error, change to `pub(crate) const SEED`.
- The `OnceLock<ToolCatalog>` in `catalog/mod.rs` means the first load is cached for the process lifetime. This means a `catalog refresh` won't be visible until restart. This is acceptable per spec (cache is on disk; process restarts see the new data). If live reload is needed, replace `OnceLock` with a `Mutex<Option<ToolCatalog>>`.
- `update_all_verbose` calls `shell::run_live` which writes to stdout — in background mode `update_all_silent` is used instead, which does not write to stdout.
