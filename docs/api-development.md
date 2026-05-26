# API et développement

## Architecture du code

### Pattern MVC simplifié

```
View: wizard/ menus (dialoguer) ou TUI (ratatui)
Model: config/, core/ (état, données)
Controller: wizard/ logique métier
```

## Points d'extension

### Ajouter un nouvel outil

1. Créer le fichier `src/tools/mon_outil.rs`:

```rust
use crate::config::{I18n, WzllamaState};
use crate::tools::tool_trait::Tool;
use anyhow::Result;

pub struct MonOutilTool;

impl Tool for MonOutilTool {
    fn id(&self) -> &str { "mon_outil" }
    
    fn name(&self) -> &str { "Mon Outil" }
    
    fn description(&self, _i18n: &I18n) -> String {
        "Description de l'outil".into()
    }
    
    fn status(&self) -> ToolStatus { ToolStatus::NotInstalled }
    
    fn install(&self) -> Result<()> {
        // Logique d'installation
        Ok(())
    }
    
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        // Logique de lancement
        Ok(())
    }
    
    fn supports_fleets(&self) -> bool { false }
    fn requires_docker(&self) -> bool { false }
}
```

2. Ajouter dans `src/tools/mod.rs`:

```rust
pub mod mon_outil;

pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    vec![
        // ... autres outils
        Box::new(mon_outil::MonOutilTool),
    ]
}
```

3. Ajouter dans `get_install_command` et `get_launch_command`:

```rust
match tool_id {
    "mon_outil" => Some("commande_install".into()),
    // ...
}

match tool_id {
    "mon_outil" => Some("commande_lancement".into()),
    // ...
}
```

4. Ajouter les traductions dans `config/i18n/fr.json`:

```json
{
  "tool.mon_outil.description": "Description de l'outil",
  "tool.mon_outil.run_model": "Lancement avec {model}",
  "tool.mon_outil.uninstall_confirm": "Désinstaller ?",
  "tool.mon_outil.uninstalled": "Désinstallé"
}
```

## API publiques principales

### Core API

```rust
// src/core/ollama_api.rs
pub fn list_local_models() -> Result<Vec<OllamaModel>>
pub fn pull_model(name: &str) -> Result<()>
pub fn delete_model(name: &str) -> Result<()>
pub fn create_model(name: &str, modelfile: &str) -> Result<()>
pub fn get_running_models() -> Vec<String>

// src/core/hardware.rs
pub fn detect() -> HardwareInfo
pub fn get_cpu_info() -> String
pub fn get_gpu_info() -> Vec<GpuInfo>

// src/core/ollama_models.rs
pub fn rank_models(models: &[OllamaModel], usage: &str, hw: &HardwareInfo, limit: usize) -> Vec<(OllamaModel, f32)>
```

### Config API

```rust
// src/config/env.rs
pub fn EnvConfig::load() -> Self
pub fn EnvConfig::save(&self) -> Result<()>
pub fn EnvConfig::generate_env_file(&self) -> Result<()>

// src/config/state.rs
pub fn WzllamaState::load() -> Self
pub fn WzllamaState::save(&self) -> Result<()>
pub fn set_language(lang: &str, state: &mut WzllamaState)
pub fn set_last_model(model: &str, state: &mut WzllamaState)

// src/config/i18n.rs
pub fn I18n::t(&self, key: &str) -> String
pub fn I18n::t_with_vars(&self, key: &str, vars: &[(&str, &str)]) -> String
pub fn load(lang_code: &str) -> Result<I18n>
pub fn get_available_languages() -> Vec<LanguageMeta>
```

### Wizard API

```rust
// src/wizard/menu_main.rs
pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()>
pub fn select_language(state: &mut WzllamaState) -> Result<I18n>
pub fn change_language(state: &mut WzllamaState) -> Result<I18n>
```

## Tests

### Tests existants

```
tests/
├── tui_app.rs      # Tests TUI state machine
└── tui_screens.rs  # Tests navigation écrans
```

### Exécuter les tests

```bash
cargo test
```

## Build et release

### Build debug

```bash
cargo build
./target/debug/wzllama
```

### Build release

```bash
cargo build --release
./target/release/wzllama
```

### Optimisation release

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

## Debugging

### Activation du logging

```bash
# Verbose output
RUST_LOG=debug wzllama

# Spécifique au module
RUST_LOG=wzllama::core::ollama_api wzllama
```

### Logging interne

```rust
use log::{info, debug, warn, error};

info!("wzllama v0.3.0 started");
debug!("Hardware detected: {:?}", hw);
warn!("Docker not ready: {}", e);
```

## Internationalisation (i18n)

### Ajouter une langue

1. Créer `config/i18n/{lang}.json`:

```json
{
  "_language": {
    "code": "es",
    "name": "Español",
    "name_en": "Spanish",
    "direction": "ltr"
  },
  "menu.main.title": "Menú Principal",
  // ... toutes les clés
}
```

2. Lancer la vérification:

```bash
wzllama check-i18n
```

### Clés de traduction

Format: `section.subsection.key`

Exemples:
- `menu.main.title` - Titre du menu principal
- `config.performance` - Label menu performance
- `tool.ollama.description` - Description de l'outil

## Conventions de code

### Nommage

- Fonctions: `snake_case`
- Variables: `snake_case`
- Types: `PascalCase`
- Constantes: `SCREAMING_SNAKE_CASE`

### Documentation

```rust
/// Court description
///
/// Description détaillée si nécessaire.
///
/// # Arguments
/// * `param` - Description
///
/// # Returns
/// Description de la valeur de retour
pub fn ma_fonction(param: Type) -> Result<Type> {
    // ...
}
```

### Gestion des erreurs

```rust
// Utiliser anyhow pour la chaîne d'erreurs
use anyhow::{Result, Context};

fn operation() -> Result<()> {
    risky_operation()
        .context("Contexte d'erreur")?;
    Ok(())
}
```

### Pattern Result

Toutes les fonctions qui peuvent échouer retournent `Result<()>` ou `Result<T>`.