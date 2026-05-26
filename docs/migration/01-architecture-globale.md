# Architecture Générale de wzllama

## 1. Vue d'ensemble du programme

```
wzllama
├── Interface CLI (clap)                ← Point d'entrée principal
├── Menu Système                        ← Navigation hiérarchique
│   ├── menu_main.rs                    ← Menu principal
│   ├── menu_wizard.rs                  ← Wizard IA (Use Cases)
│   ├── menu_tools.rs                   ← Installation/Outils
│   ├── menu_models.rs                  ← Sélection modèles
│   ├── menu_scientific.rs              ← Outils scientifiques
│   ├── menu_cleanup.rs                 ← Nettoyage
│   └── menu_config.rs                  ← Configuration
├── API HTTP (axum)                     ← Serveur API REST
│   └── api_server.rs                   ← Routes et endpoints
├── Core (Fonctionnalités)              ← Logique métier
│   ├── ollama_api.rs                   ← API Ollama locale
│   ├── hardware.rs                     ← Détection matériel
│   ├── ollama_models.rs                ← Gestion modèles
│   ├── llmfit_api.rs                   ← Recommandations modèles
│   └── system.rs                       ← Commandes système
├── Outils (Tools)                      ← Outils IA installables
│   ├── ollama.rs                       ← LLM local (moteur principal)
│   ├── open_webui.rs                   ← Interface web
│   ├── claude_code.rs                  ← Agent IA
│   ├── openclaw.rs                     ← Recherche web
│   ├── hermes.rs                       ← Agent IA
│   ├── opencode.rs                     ← Agent IA
│   ├── codex.rs                        ← Agent IA
│   ├── copilot_cli.rs                  ← Agent IA
│   ├── droid.rs                        ← Agent IA
│   ├── pi.rs                           ← Agent IA
│   ├── pool.rs                         ← Pool d'outils
│   ├── obsidian.rs                     ← Wiki personnel
│   └── goose.rs                        ← Agent IA
├── Configuration
│   ├── state.rs                        ← État persistant (JSON)
│   ├── i18n.rs                         ← Internationalisation
│   ├── env.rs                          ← Variables d'environnement
│   ├── paths.rs                        ← Chemins système
│   └── logging.rs                      ← Système de logs
└── menu_api                            ← Abstraction menu hiérarchique
    ├── menu_tree.rs                    ← Structure arbre de menus
    ├── menu_item.rs                    ← Élément de menu
    ├── menu_handler.rs                 ← Gestionnaire de navigation
    └── api_first.rs                    ← API-first menu structure
```

## 2. Diagramme des relations composants

```
┌─────────────────────────────────────────────────────────────────┐
│                         main.rs (point d'entrée)                  │
│                               │                                    │
│            ┌──────────────────┴──────────────────┐              │
│            ▼                                     ▼              │
│    ┌───────────────┐                    ┌───────────────┐       │
│    │   CLI (clap)  │                    │   API Server  │       │
│    │   (wizard)    │                    │   (axum)      │       │
│    └───────────────┘                    └───────────────┘       │
│            │                                     │              │
│            ▼                                     ▼              │
│    ┌───────────────┐                    ┌───────────────┐       │
│    │ MenuHandler   │◄──────────────────►│ /api/v1/menu  │       │
│    │ (dialoguer)   │                    │ /api/v1/tools │       │
│    └───────────────┘                    └───────────────┘       │
│            │                                     │              │
│            ▼                                     ▼              │
│    ┌───────────────┐                    ┌───────────────┐       │
│    │   wizard/     │                    │  menu_api/    │       │
│    │   mod.rs      │───────────────────►│ api_first.rs  │       │
│    └───────────────┘                    └───────────────┘       │
│            │                                     │              │
│            ▼                                     ▼              │
│    ┌───────────────┐                    ┌───────────────┐       │
│    │   tools/      │                    │  core/        │       │
│    │   mod.rs      │───────────────────►│               │       │
│    └───────────────┘                    └───────────────┘       │
│            │                               �            │              │
│            ▼                               �            ▼              │
│    ┌───────────────┐                       �    ┌───────────────┐      │
│    │  Tool trait   │                       �    │ ollama_api    │      │
│    │ (dyn install/ │                       �    │ hardware      │      │
│    │  launch/...)   │                       �    │ system        │      │
│    └───────────────┘                       �    └───────────────┘      │
│            │                               �            │              │
│            ▼                               �            ▼              │
│    ┌───────────────┐                    ┌───────────────┐       │
│    │15 outils      │                    │ Config/state  │       │
│    │ implémentés    │                    │ i18n          │       │
│    └───────────────┘                    │ paths         │       │
│                                         └───────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

## 3. Patterns de conception utilisés

### 3.1 Trait Object Pattern (Tool)
```rust
// Trait définissant l'interface des outils
pub trait Tool {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self, i18n: &I18n) -> String;
    fn install(&self, i18n: &I18n) -> anyhow::Result<()>;
    fn launch(&self, model: Option<&str>) -> anyhow::Result<()>;
    fn is_installed(&self) -> bool;
    fn supports_agentic(&self) -> bool { false }
}
```

### 3.2 Menu Tree Pattern (Composite)
```
MenuItem (abstract)
├── Menu (peut contenir des enfants)
└── Leaf (action terminale)
```

### 3.3 State Pattern (WzllamaState)
- Persistance JSON de l'état utilisateur
- Stockage : `~/.config/wzllama/state.json`

### 3.4 Builder Pattern (MenuTree)
```rust
MenuTree::new("root")
    .with_root(MenuItem::branch("menu"))
    .with_metadata(MenuMetadata { title: Some("...") })
```

## 4. Flux de données principal

### 4.1 Démarrage CLI → Menu principal
```
main() 
  └─► Cli::execute()
      └─► wizard::run()
          └─► MainMenuRunner::run()
              ├─► Affichage header matériel
              ├─► Sélection menu (dialoguer::Select)
              └─► Dispatch vers menu concerné
```

### 4.2 Menu Wizard → Outils
```
WizardMenuRunner::run()
  └─► Sélection UseCase (Coding/Chat/Reasoning/Embedding/Multimodal)
      └─► Tool selection via get_tools_for_usecase()
          └─► Launch or install tool
```

### 4.3 API Server (background)
```
api_server::start_server()
  └─► Routes axum montées
      ├─► GET /api/v1/menu          → MenuTree JSON
      ├─► GET /api/v1/tools         → Liste outils
      ├─► GET /api/v1/models        → Liste modèles
      ├─► GET /api/v1/hardware      → Info matériel
      └─► POST /api/v1/tools/{id}/{action}
```

## 5. Structure des dossiers

| Dossier | Description |
|---------|-------------|
| `src/` | Code source principal |
| `src/tools/` | Implémentations des outils IA |
| `src/wizard/` | Logique des menus CLI |
| `src/menu_api/` | Abstraction menu pour API |
| `src/core/` | Fonctionnalités système |
| `src/config/` | Configuration et état |
| `config/` | Fichiers de configuration externe |
| `config/menus/` | Menus au format TOML |
| `config/i18n/` | Fichiers de traduction JSON |
| `config/mcp/` | Configuration MCP (Model Context Protocol) |
| `tests/` | Tests unitaires et d'intégration |