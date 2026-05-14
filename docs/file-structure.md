# Structure des fichiers

## Arborescence du projet

```
wzllama/
├── Cargo.toml                    # Configuration Cargo (dépendances)
├── Cargo.lock                    # Lockfile Cargo
├── README.md                     # Documentation principale (à créer)
├── DOCUMENTATION.md              # Index documentation
├── LICENSE                       # Licence du projet
│
├── src/                          # Code source Rust
│   ├── main.rs                   # Entry point (4 lignes)
│   ├── lib.rs                    # Bibliothèque partagée
│   ├── cli.rs                    # Parsing CLI et routing
│   ├── display.rs                # Utilities affichage
│   ├── error.rs                  # Gestion d'erreurs
│   │
│   ├── config/                   # Configuration et état
│   │   ├── mod.rs                # Exports publics
│   │   ├── env.rs                # EnvConfig (yaml → env)
│   │   ├── i18n.rs               # Internationalisation
│   │   ├── paths.rs              # Gestion chemins (~/.wzllama)
│   │   ├── state.rs              # WzllamaState persistant
│   │   ├── logging.rs            # Setup logging
│   │   ├── templates.rs          # Templates embarqués
│   │   ├── fleets.rs             # Gestion flottes OpenClaw
│   │   └── shells.rs             # Shell completions
│   │
│   ├── core/                     # Logique métier
│   │   ├── mod.rs
│   │   ├── hardware.rs           # Détection CPU/RAM/GPU
│   │   ├── ollama_api.rs         # API Ollama
│   │   ├── ollama_models.rs      # Modèles IA et ranking
│   │   ├── ollama_doctor.rs      # Diagnostics
│   │   ├── estimation.rs         # Estimation ressources
│   │   ├── shell.rs              # Shell utilities
│   │   └── system.rs             # System info (RAM/VRAM)
│   │
│   ├── wizard/                   # Mode CLI wizard
│   │   ├── mod.rs                # Exports
│   │   ├── menu_main.rs          # Menu principal + alternate screen
│   │   ├── menu_models.rs        # Gestion modèles
│   │   ├── menu_tools.rs         # Lancement outils
│   │   ├── menu_fleets.rs          # Gestion flottes
│   │   ├── menu_cleanup.rs       # Nettoyage
│   │   ├── menu_config.rs        # Configuration
│   │   ├── cleanup_fleets.rs     # Suppression flottes
│   │   ├── cleanup_models.rs     # Suppression modèles
│   │   ├── cleanup_tools.rs      # Suppression outils
│   │   ├── configurator.rs       # Config avant création modèle
│   │   ├── fleet_creator.rs      # Création flotte interactive
│   │   ├── fleet_templates.rs    # Templates de flottes
│   │   ├── setup_models.rs       # Installation modèles initiaux
│   │   └── estimator.rs          # Estimation ressources
│   │
│   ├── tui/                      # Terminal UI mode
│   │   ├── mod.rs                # run_tui entry
│   │   ├── app.rs                # State machine
│   │   ├── ui.rs                 # Rendering
│   │   ├── event.rs              # Event handling
│   │   ├── screens.rs            # Screen enum
│   │   ├── widgets.rs            # Custom widgets
│   │   └── terminal.rs           # Terminal setup
│   │
│   └── tools/                    # Outils IA (plugins)
│       ├── mod.rs                # Registry + ToolInfo
│       ├── tool_trait.rs         # Trait Tool
│       ├── docker.rs             # Docker management
│       ├── ollama.rs             # Ollama tool
│       ├── openclaw.rs           # OpenClaw tool
│       ├── open_webui.rs         # Open WebUI tool
│       ├── claude_code.rs
│       ├── codex.rs
│       ├── copilot_cli.rs
│       ├── droid.rs
│       ├── hermes.rs
│       ├── opencode.rs
│       ├── pi.rs
│       └── pool.rs
│
├── config/                       # Fichiers embarqués
│   ├── templates/                # Templates de configuration
│   │   ├── modelfile.code        # Modèle code
│   │   ├── modelfile.book        # Modèle livre
│   │   └── modelfile.agents      # Modèle agents
│   └── i18n/                     # Fichiers de traduction
│       └── fr.json               # Français (318 clés)
│
├── tests/                        # Tests
│   ├── tui_app.rs
│   └── tui_screens.rs
│
├── docs/                         # Documentation (créée)
│   ├── overview.md
│   ├── architecture.md
│   ├── getting-started.md
│   ├── cli-wizard.md
│   ├── tui-mode.md
│   ├── tools.md
│   ├── configuration.md
│   ├── models.md
│   ├── fleets.md
│   ├── api-development.md
│   ├── i18n.md
│   └── file-structure.md
│
└── target/                       # Build artifacts (généré)
    ├── debug/
    └── release/
```

## Fichiers de configuration utilisateur

### ~/.wzllama/

```
~/.wzllama/
├── config.yaml        # Configuration principale (éditeable)
├── state.json         # État persistant (auto-généré)
├── env                # Fichier environnement (auto-généré)
├── i18n/              # Traductions personnalisées (optionnel)
│   └── custom.json
├── fleets/            # Flottes OpenClaw
│   ├── myproject/
│   │   ├── fleet.yaml
│   │   └── agents/
│   └── another-project/
└── completions/       # Shell completions (optionnel)
    ├── wzllama.bash
    └── wzllama.zsh
```

## Fichier config.yaml détaillé

### Sections

```yaml
# Ollama configuration
ollama:
  host: string           # Adresse du serveur
  origins: string        # CORS origins
  keep_alive: i32        # TTL modèles
  no_cloud: bool         # Force local
  num_parallel: u32      # Requêtes parallèles
  max_loaded_models: u32 # Modèles en RAM
  flash_attention: bool  # Optimisation VRAM
  kv_cache_type: string  # q8_0, f16, q4_0
  context_length: u32    # Tokens contexte

# Provider API keys (Ollama-compatible)
providers:
  openai:
    api_key: string
    base_url: string
  anthropic:
    api_key: string
    base_url: string

# OpenClaw configuration
openclaw:
  api_key: string

# Models par usage
models:
  code: string   # Modèle programmation
  book: string   # Modèle texte long
  agent: string  # Modèle agents légers
  chat: string   # Modèle chat général
```

## Fichier state.json

```json
{
  "language": "fr",
  "last_model": "qwen2.5-coder:14b",
  "installed": {
    "ollama": true,
    "open_webui": false,
    "openclaw": true,
    "claude_code": false,
    "opencode": false,
    "codex": false,
    "copilot_cli": false,
    "droid": false,
    "hermes_agent": false,
    "pi": false,
    "pool": false
  }
}
```

## Templates embarqués

### modelfile.code

```
FROM {model}
PARAMETER num_ctx {context}
PARAMETER temperature {temp}
PARAMETER flash_attention true
SYSTEM {system_prompt}
```

## Build artifacts

```
target/
├── debug/wzllama      # Build debug
└── release/wzllama    # Build release (optimisé)
```

## Dépendances principales (Cargo.toml)

```toml
[dependencies]
# CLI
clap = { version = "4.5", features = ["derive"] }

# Async
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Système
dirs = "6.0"
sysinfo = "0.39"

# Erreurs
anyhow = "1.0"
thiserror = "2.0"

# Logging
log = "0.4"
env_logger = "0.11"

# Affichage
colored = "3.1"

# TUI
ratatui = "0.26"
crossterm = "0.27"
unicode-width = "0.1"
strum = "0.26"
strum_macros = "0.26"

# CLI interaction
dialoguer = "0.12"
```