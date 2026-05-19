# wzllama 🦙

**wzllama** is an interactive CLI wizard that simplifies installing, configuring and running a complete local AI stack — Ollama, Open WebUI, OpenClaw, coding agents and more — all from a single terminal interface.

```
wzllama
```

![Rust](https://img.shields.io/badge/Rust-edition%202021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey)

---

## Why wzllama?

Running a local AI stack usually means manually installing Ollama, hunting for compatible models, juggling environment variables, and wiring up multiple tools. wzllama automates all of it:

- **One command** to install and configure the entire stack
- **Hardware-aware** model recommendations (CPU, RAM, GPU auto-detected)
- **Centralized config** — one `config.yaml`, one generated `env` file
- **Multi-agent orchestration** via OpenClaw fleets
- **FR / EN** support out of the box (system language auto-detected)

---

## Features

| | |
|---|---|
| 🔧 **Automated install** | Ollama, Open WebUI, Claude Code, OpenCode, Codex, Droid, Hermes, Pi, Pool, Copilot CLI |
| 🎯 **Smart model ranking** | Ranked by usage type (code, long text, agents, chat) and your hardware |
| ⚙️ **Centralized config** | `config.yaml` → auto-generated `~/.wzllama/env` |
| 🚀 **Agent fleets** | Create orchestrator + reflexion + expert agent groups for OpenClaw |
| 🌍 **i18n** | FR / EN, extensible to any language |
| 💻 **CLI wizard** | Interactive terminal interface with hardware-aware recommendations |

---

## Quick Start

### Prerequisites

- **OS**: Linux (tested on Arch, Ubuntu)
- **RAM**: 8 GB minimum, 16 GB recommended
- **GPU**: Optional but strongly recommended (NVIDIA / AMD)
- **Rust**: To build from source

### Install from remote (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/olivierlevy/wzllama/main/install.sh | sh
```

This script automatically downloads, compiles, and installs wzllama.

### Install locally (for developers)

```bash
git clone https://github.com/olivierlevy/wzllama.git
cd wzllama
./deploy.sh
```

The `deploy.sh` script builds and installs wzllama locally for development purposes.

### First run

```bash
wzllama          # CLI wizard (default)
wzllama --tui    # Rich TUI mode (beta)
```

wzllama will detect your hardware, offer to install Ollama, and suggest models suited to your machine.

→ Full setup guide: [docs/getting-started.md](docs/getting-started.md)

---

## Supported Tools

| Tool | Description | Transport |
|------|-------------|-----------|
| **Ollama** | Local AI model server | Native |
| **LLMFit** | LLM training and evaluation tool with MCP support | Native / MCP |
| **Open WebUI** | Web interface for AI models | Docker |
| **OpenClaw** | Personal AI assistant (100+ skills, fleet support) | Ollama |
| **Claude Code** | Anthropic coding agent with sub-agents | NPM |
| **OpenCode** | Open-source coding agent by Anomaly | NPM |
| **Codex** | OpenAI coding agent | Ollama |
| **Copilot CLI** | GitHub AI agent for the terminal | Native |
| **Droid** | Factory coding agent (terminal + IDE) | Ollama |
| **Hermes Agent** | Self-improving AI agent by Nous Research | NPM |
| **Pi** | Minimal AI agent with plugin support | Native |
| **Pool** | Poolside coding agent | Native |

→ Full tool reference: [docs/tools.md](docs/tools.md)

### LLMFit MCP Integration

LLMFit can be used as an MCP (Model Context Protocol) server, making it available to AI agents like Claude Desktop:

```bash
# Install llmfit (done automatically by wzllama)
uv tool install -U llmfit

# MCP configuration for Claude Desktop (claude_desktop_config.json):
{
  "mcpServers": {
    "llmfit": {
      "command": "llmfit",
      "args": ["serve", "--mcp"]
    }
  }
}
```

Available MCP tools: `get_system_specs`, `recommend_models`, `search_models`, `plan_hardware`, `get_runtimes`, `get_installed_models`.

→ MCP documentation: [config/mcp/README.md](config/mcp/README.md)

---

## Interface Modes

### CLI Wizard (default)

```
┌─ wzllama ──────────────────────────────────┐
│  RAM: 32GB  │  GPU: RTX 4090 (24GB VRAM)   │
├─────────────────────────────────────────────┤
│  Menu Principal                              │
│                                              │
│  > 🤖 Choose an AI model                    │
│    🛠  Launch a tool                         │
│    🚀 OpenClaw Fleets                        │
│    🧹 Cleanup                                │
│    ⚙️  Configuration                         │
│    🌍 Change language                        │
│    ❌ Quit                                   │
└─────────────────────────────────────────────┘
```

Navigate with **↑ ↓ Enter** — **Escape** goes back. Works in any terminal ≥ 40×10.

→ Full wizard reference: [docs/cli-wizard.md](docs/cli-wizard.md)

### TUI Mode (`--tui`)

Richer terminal interface with real-time resource widgets (RAM/VRAM bars). Recommended terminal size: 80×25.

→ TUI documentation: [docs/tui-mode.md](docs/tui-mode.md)

---

## Configuration

wzllama stores everything under `~/.wzllama/`:

```
~/.wzllama/
├── config.yaml      # Main configuration (editable)
├── env              # Auto-generated environment file
├── state.json       # Persistent state
└── fleets/          # OpenClaw agent fleets
```

Example `config.yaml`:

```yaml
ollama:
  host: "127.0.0.1:11434"
  flash_attention: true
  kv_cache_type: "q8_0"
  context_length: 16384

models:
  code: "qwen2.5-coder:14b"
  book: "qwen2.5:14b"
  agent: "qwen2.5:3b"
  chat: "qwen2.5:7b"
```

→ Full configuration reference: [docs/configuration.md](docs/configuration.md)

---

## OpenClaw Fleets

Fleets are groups of specialized agents (orchestrator + reflexion + experts) that collaborate on complex tasks.

```
🚀 Fleet: my-project
├── Orchestrator   qwen2.5:7b   "Chief software architect"
├── Reflexion      ── Software architect
├── Reflexion      ── Code reviewer
└── Experts        ── Linter / Documentarian / Tester
```

→ Fleet documentation: [docs/fleets.md](docs/fleets.md)

---

## Documentation

| Document | Description |
|----------|-------------|
| [Overview](docs/overview.md) | What wzllama is and how it works |
| [Getting Started](docs/getting-started.md) | Installation and first run |
| [CLI Wizard](docs/cli-wizard.md) | Wizard mode reference |
| [TUI Mode](docs/tui-mode.md) | TUI mode reference |
| [Tools](docs/tools.md) | All supported tools |
| [Models](docs/models.md) | Model management and ranking |
| [Fleets](docs/fleets.md) | OpenClaw agent fleets |
| [Configuration](docs/configuration.md) | config.yaml reference |
| [i18n](docs/i18n.md) | Adding a new language |
| [Architecture](docs/architecture.md) | Code architecture and module map |
| [API & Development](docs/api-development.md) | Extending wzllama |
| [File Structure](docs/file-structure.md) | Full project file tree |

---

## Tech Stack

- **Language**: Rust (edition 2021)
- **CLI parsing**: clap 4.5
- **TUI**: ratatui 0.26 + crossterm 0.27
- **CLI interaction**: dialoguer 0.12
- **Serialization**: serde / serde_yaml / serde_json
- **i18n**: JSON with HashMap

---

## Contributing

Adding a new tool is straightforward — implement the `Tool` trait, register it in the tool registry, add i18n keys. See [docs/api-development.md](docs/api-development.md) for a step-by-step guide.

---

## License

MIT — see [LICENSE](LICENSE)