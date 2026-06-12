# wzllama – Copilot Instructions

wzllama is a Rust CLI wizard for managing a local AI stack (Ollama, Open WebUI, OpenClaw, and other AI tools). It provides an interactive TUI wizard, direct CLI commands, and an embedded HTTP API server.

## Build, Test, and Lint

```bash
# Build (release)
cargo build --release

# Run all tests
cargo test

# Run a single test file or test by name
cargo test --test state_tests
cargo test my_test_name

# Library unit tests only
cargo test --lib

# Lint (CI enforces -D warnings)
cargo clippy -- -D warnings

# Format
cargo fmt
```

The CI pipeline runs `cargo check --all-targets`, `cargo clippy -- -D warnings`, then `cargo test`.

## Migration Goal (Active Direction)

**All wizard/menu functionality is being migrated to be callable from the HTTP API service layer.** The `menu_api` module is the target architecture; the `wizard/` dialoguer menus are the legacy source.

Migration pattern:
1. Each wizard menu action becomes a `ToolAction` implementation registered with `ActionDispatcher`
2. `menu_api/api_service.rs` exposes the actions via REST endpoints (port 1133)
3. The `wizard/` menus become thin wrappers that call the same `menu_api` logic
4. Helper functions in `core/` and `tools/` must be service-callable (no stdin/stdout assumptions)

Target API surface (port 1133):
```
GET  /api/v1/menu              → Menu tree structure (JSON)
GET  /api/v1/tools             → All registered tools
GET  /api/v1/models            → Ollama model list
GET  /api/v1/hardware          → Hardware info
POST /api/v1/tools/{id}/install
POST /api/v1/tools/{id}/launch
GET  /api/menu/state           → Current WzllamaState
POST /api/menu/action          → Execute any menu action by ID
GET  /api/menu/i18n            → i18n string map
```

When adding or refactoring functionality: implement it in `menu_api` (or `core/`/`tools/`) first, then wire the wizard menu to call it — not the other way around.

## Architecture

```
main.rs → cli.rs (clap) → execute()
                          ├── Wizard mode  → wizard/ (dialoguer TUI)
                          │                  └── menu_main → menu_models / menu_tools / menu_fleets / menu_cleanup / menu_config
                          ├── Serve mode   → api_server.rs (axum, port 1133)
                          └── Other cmds   → core/ helpers
```

In **wizard mode**, the app also auto-starts the API server in a background thread (once per session via `OnceLock`). The API shuts down gracefully via `request_shutdown()` when the wizard exits.

**Key modules:**

| Module | Responsibility |
|---|---|
| `src/config/` | Persistent state (`WzllamaState`), i18n (`I18n`), env config, paths (`~/.wzllama/`), shell completions |
| `src/core/` | Ollama HTTP API, hardware detection (CPU/RAM/GPU), model ranking, diagnostics |
| `src/tools/` | Tool plugin registry; each tool implements the `Tool` trait |
| `src/wizard/` | Interactive dialoguer menus (legacy, wizard mode) |
| `src/menu_api/` | New hierarchical `MenuTree`/`MenuItem`/`MenuHandler` system with `ToolAction` trait and HTTP service layer |
| `src/api_server.rs` | Axum HTTP server exposing `/api/menu/*` endpoints |
| `src/display.rs` | Terminal-aware output helpers (`menu_max_items` adapts to terminal height) |

## HTTP API Reference (port 1133)

The full OpenAPI spec is in `openapi.yaml`. Key endpoints:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check → `"OK"` |
| GET | `/api/v1/tools` | List all tools (`ToolInfo[]`) |
| GET | `/api/v1/tools/{id}` | Tool details |
| GET | `/api/v1/tools/{id}/status` | Install status |
| POST | `/api/v1/tools/{id}/install` | Install tool |
| POST | `/api/v1/tools/{id}/update` | Update tool |
| POST | `/api/v1/tools/{id}/uninstall` | Uninstall tool |
| POST | `/api/v1/tools/{id}/launch` | Launch tool |
| GET | `/api/v1/models` | Installed Ollama models |
| POST | `/api/v1/models/{name}/pull` | Pull model |
| DELETE | `/api/v1/models/{name}/delete` | Delete model |
| GET | `/api/v1/hardware` | `HardwareInfo` (RAM, GPU, VRAM) |
| GET | `/api/v1/status` | System status + Ollama status |
| GET | `/api/v1/menu` | Menu tree (JSON) |
| GET | `/api/menu/state` | `WzllamaState` |
| POST | `/api/menu/action` | Execute action by ID |
| GET | `/api/menu/i18n` | i18n string map |

`ActionResponse`: `{ success: bool, message: string }`

## Model Recommendation Pipeline

wzllama uses two backends for model recommendations:

**1. llmfit (`src/core/llmfit_api.rs`)** — local service at `127.0.0.1:8787`:
```rust
let client = LLMFitClient::new();
// Get hardware-aware top models for a use_case
client.get_top_models(limit, min_fit, use_case)?
// → Vec<LLMFitModel> with fit_level, score, estimated_tps, best_quant, memory_required_gb
```
- `start_server(port)` / `stop_server()` manage the `llmfit serve` subprocess
- `LLMFitModel` fields to use for ranking: `fit_level`, `score`, `estimated_tps`, `best_quant`, `run_mode`

**2. localmaxxing (`src/core/localmax_models.rs`)** — remote catalog at `https://localmaxxing.com/api`:
- Fetches HuggingFace model IDs and maps them to Ollama names via `hf_to_ollama_name(hf_id)`
- The `hf_to_ollama_name` function handles family detection (qwen/phi/llama/mistral/gemma/deepseek/etc.) and parameter size extraction
- Prefer the llmfit local service when available; fall back to Ollama's own ranking (`rank_models` in `ollama_models.rs`)

**Hardware-based model defaults** (`setup_models.rs`):

| RAM | Heavy | Light |
|-----|-------|-------|
| 8 GB | qwen2.5:7b | qwen2.5:3b |
| 16 GB | qwen2.5:14b | qwen2.5:7b |
| 32+ GB | qwen2.5:32b | qwen2.5-coder:14b |

## OpenClaw Fleet System

Fleets are multi-agent configurations stored in `~/.wzllama/fleets/{project_name}/`.

**Key structures:**
```rust
pub struct FleetConfig {
    pub orchestrator: OrchestratorConfig,   // coordinator model
    pub reflexion_agents: Vec<AgentTemplate>, // analysis/review agents
    pub expert_agents: Vec<AgentTemplate>,   // specialist agents (lint/doc/test)
}
```

**Fleet lifecycle:**
1. `fleet_creator::run()` — interactive wizard: project name → orchestrator config → reflexion agents → expert agents → custom agents → generate YAML files
2. Files written to `~/.wzllama/fleets/{project}/`: `fleet.yaml`, `orchestrator.yaml`, `agents/reflexion-*.yaml`, `agents/experts/*.yaml`
3. Launch: `ollama launch openclaw --project {name}`

**Built-in fleet templates** (`wizard/fleet_templates.rs`):
- `"code"` — orchestrator + reflexion-arch + reflexion-review + experts: lint, doc, test
- `"generic"` — orchestrator + reflexion + expert-fast

**Detection:** `detect_openclaw_fleets()` scans `~/.wzllama/fleets/` and returns `HashMap<name, PathBuf>`.

Only `openclaw` tool has `supports_fleets() → true`. Fleet creation is triggered from `configurator.rs` after model selection.

## Tool Registry

All 11 tools registered in `src/tools/mod.rs` via `get_all_tools()`:

| ID | requires_docker | supports_fleets | Install method |
|----|----------------|----------------|----------------|
| `ollama` | No | No | `curl -fsSL https://ollama.com/install.sh \| sh` |
| `open_webui` | **Yes** | No | `wzllama install-webui` (Docker) |
| `openclaw` | No | **Yes** | `ollama install openclaw` |
| `claude_code` | No | No | `npm install -g @anthropic-ai/claude-code` |
| `opencode` | No | No | `npm install -g @opencode-ai/cli` |
| `codex` | No | No | `ollama install codex` |
| `copilot_cli` | No | No | — |
| `droid` | No | No | — |
| `hermes_agent` | No | No | `npm install -g @hermes-hq/bot` |
| `pi` | No | No | — |
| `pool` | No | No | — |

Installation status is tracked per-tool in `WzllamaState.installed` (one bool field per tool ID).

When adding a new tool: create `src/tools/{id}.rs`, implement `Tool`, add to `get_all_tools()`, add `installed.{id}` to `InstalledTools`, add i18n keys `tool.{id}.description` etc.

## Key Conventions

### Error handling
Use `anyhow::Result` for most functions. `WzllamaError` (thiserror) in `error.rs` for domain errors. Error messages are in French (default locale).

### i18n – all user-visible strings go through `I18n`
```rust
i18n.t("menu.main.title")
i18n.t_with_vars("config.model_not_found", &[("usage", "code"), ("model", &name)])
```
Keys: `menu.<section>.<key>`, `tool.<id>.description`, `config.<key>`. FR is the reference locale. Fallback chain: `~/.wzllama/i18n/{lang}.json` → `config/i18n/{lang}.json` → `fr.json`.

### Tool plugin pattern (Strategy)
```rust
fn id(&self) -> &str;
fn name(&self) -> &str;
fn description(&self, i18n: &I18n) -> String;
fn status(&self) -> ToolStatus;
fn install(&self) -> Result<()>;
fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()>;
fn supports_fleets(&self) -> bool { false }
fn requires_docker(&self) -> bool { false }
```

### Menu patterns
**Wizard menus (legacy):** `dialoguer::Select::interact_opt()` — `None` = Escape/Ctrl-C → return `Ok(())`.

**menu_api (target):** `MenuTree` → `MenuItem::branch/leaf` → `ToolAction` registered with `ActionDispatcher` → `MenuHandler::run()`. "Retour" always at position 0 in sub-menus; "Quitter" always last.

### Persistent user data (`~/.wzllama/`)
- `config.yaml` — Ollama settings, provider API keys, model-per-usage assignments
- `state.json` — `WzllamaState`: language, last_model, `InstalledTools` booleans
- `env` — generated shell env file
- `fleets/{project}/` — fleet YAML configs

### Debugging / logging
```bash
RUST_LOG=debug wzllama
RUST_LOG=wzllama::core::ollama_api wzllama
```
Use `log::{info, debug, warn, error}` internally — not `println!`.

### Commits
Follow Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
