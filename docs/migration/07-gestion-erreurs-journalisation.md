# Gestion des Erreurs et Journalisation

## 1. Stratégie de Gestion d'Exceptions

### Pattern utilisé
- **anyhow::Result<()> pour les fonctions principales**
- **thiserror pour les erreurs spécifiques (optionnel)**
- **No-abort policy**: errors affichés, programme continue quand possible

### Hiérarchie des erreurs
```rust
// Types d'erreurs courantes
anyhow::Error        // Erreurs génériques
std::io::Error       // Erreurs filesystem
reqwest::Error       // Erreurs HTTP API
serde_json::Error    // Erreurs parsing JSON
dialoguer::Error     // Erreurs UI terminal
```

### Gestion contextuelle
```rust
// Exemple: contexte ajouté pour debugging
use anyhow::{Context, Result};

fn fetch_models() -> Result<Vec<OllamaModel>> {
    let url = format!("{}/api/tags", base_url);
    let client = Client::new();
    let resp = client.get(&url)
        .send()
        .context("Ollama API unreachable")?;  // Contexte ajouté
    Ok(resp.json().context("Failed to parse response")?)
}
```

---

## 2. Système de Logging

### Initialisation
```rust
// src/config/logging.rs
pub fn init() -> Result<()> {
    let log_file = paths::log_dir().join("wzllama.log");
    let file = std::fs::create(&log_file)?;
    
    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .filter_level(log::LevelFilter::Debug)  // Debug minimum
        .init();
    
    Ok(())
}
```

### Niveaux de log
| Niveau | Usage | Exemple |
|--------|-------|---------|
| ERROR | Erreurs critiques | `log::error!("Action failed: {}", e)` |
| WARN | Warnings non-fatals | `log::warn!("{}", msg)` |
| INFO | Informations utiles | `display::info()`, `display::success()` |
| DEBUG | Debugging détaillé | Inutilisé explicitement, mais niveau disponible |

### Destination
- **Fichier**: `~/.config/wzllama/wzllama.log` (ou `~/.wzllama/log/`)
- **Format**: texte simple (env_logger default)

### Macros utilisées
```rust
use log::{error, warn, info, debug};

// Dans menu_handler.rs
error!("Action '{}' failed: {}", action_id, e);
warn!("{}", msg);

// Dans display.rs (messages UI)
pub fn success(msg: &str) { println!("✓ {}", msg.green()); }
pub fn warning(msg: &str) { println!("⚠️  {}", msg.yellow()); }
pub fn error(msg: &str) { println!("❌ {}", msg.red()); }
pub fn run(msg: &str) { println!("🚀 {}", msg.cyan()); }
pub fn info(msg: &str) { println!("ℹ️  {}", msg); }
```

---

## 3. Messages d'Erreur Utilisateur

### Fichier i18n (messages en/erreur)
```json
{
  "error.ollama_not_running": "Ollama service is not running",
  "error.model_download_failed": "Failed to download model: {error}",
  "error.tool_install_failed": "Failed to install {tool}: {error}",
  "error.tool_launch_failed": "Failed to launch {tool}: {error}",
  "error.no_models_available": "No models available",
  "error.no_tools_installed": "No tools installed for this use case",
  "warning.docker_required": "Docker is required but not installed",
  "warning.low_disk_space": "Low disk space: {gb}GB available"
}
```

### Messages d'erreur par catégorie

#### Ollama Errors
| Code | Message | Type |
|------|---------|------|
| `ollama.not_installed` | "Ollama n'est pas installé" | FATAL (bloque wizard) |
| `ollama.not_running` | "Ollama n'est pas en cours d'exécution" | RECOVERABLE |
| `ollama.install_failed` | "Échec de l'installation d'Ollama" | ERROR |

#### Tool Errors
| Code | Message | Type |
|------|---------|------|
| `tool.install_failed` | "Échec de l'installation de {tool}" | ERROR |
| `tool.launch_failed` | "Échec du lancement de {tool}" | ERROR |
| `tool.update_failed` | "Mise à jour non supportée pour cet outil" | WARN |

#### Disk/Memory Errors
| Code | Message | Type |
|------|---------|------|
| `warning.low_disk_space` | "Espace disque insuffisant: {gb}GB disponible" | WARN |

---

## 4. Validation Entrées Utilisateur

### Dialoguer Validation
```rust
// Confirmation avant action destructive
Confirm::new()
    .with_prompt("Confirmer la désinstallation?")
    .default(false)
    .interact()?;
```

### Validation ToolAction
```rust
pub trait ToolAction {
    fn validate(&self, ctx: &ActionContext) -> Result<()> {
        // Override pour validation spécifique
        Ok(())
    }
    
    fn requires_confirmation(&self) -> bool {
        // Override pour actions critiques
        false
    }
}
```

---

## 5. Recovery et Fallbacks

### Fallback LLMFit → LocalMax
```rust
// Si LLMFit API indisponible
let api_models = get_models_from_llmfit(use_case);

if api_models.is_empty() {
    // Fallback sur recherche locale
    localmax_models::fetch_models_by_search(search_query, 50)
        .unwrap_or_default()  // Ou liste fallback
}
```

### Fallback Docker → Instructions
```rust
// Si Docker manquant
if let Err(e) = docker::ensure_ready_no_confirm() {
    println!("⚠️  Docker non prêt: {}", e);
    println!("💡 Pour installer Docker: curl -fsSL https://get.docker.com | sh");
    // Pas d'abort, utilisateur peut continuer
}
```

### Fallback State Corruption
```rust
// Si state.json corrompu
let content = read_to_string(&path).unwrap_or_default();
serde_json::from_str(&content).unwrap_or_else(|_| {
    // Backup du fichier corrompu
    let _ = fs::copy(&path, path.with_extension("json.bak"));
    WzllamaState::default()  // Retour état par défaut
})
```

---

## 6. Gestion des Signaux

### Shutdown API Server
```rust
// Static shutdown flag
static API_SHUTDOWN: OnceLock<Arc<AtomicBool>> = OnceLock::new();

pub fn request_shutdown() {
    if let Some(flag) = API_SHUTDOWN.get() {
        flag.store(true, Ordering::SeqCst);
    }
}

// Dans serveur async
axum::serve(listener, app)
    .with_graceful_shutdown(async move {
        while !shutdown_flag.load(Ordering::SeqCst) {
            sleep(100ms).await;
        }
    })
    .await
```

---

## 7. Codes de Retour CLI

| Code | Signification |
|------|---------------|
| 0 | Succès |
| 1 | Erreur générale (anyhow) |
| 2+ | Erreurs spécifiques selon contexte |

### Exemple de main.rs
```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

---

## 8. Logging dans les Modules

### Modules avec logging actif
- `menu_handler.rs`: error/warn pour actions échouées
- `shell.rs`: commandes exécutées avec run_live
- `ollama_api.rs`: erreurs API

### Format log attendu
```
[ERREUR] message d'erreur
[WARN] avertissement
[INFO] information utilisateur
[DEBUG] debug détaillé
```

---

## 9. Reporting d'Erreurs

### Console output
- Messages colorés (vert/jaune/rouge)
- Instructions de récupération quand possible
- Fallback data quand service externe indisponible

### Fichier log
- Toutes les erreurs avec backtrace (si debug)
- Rotation simple (overwrite à chaque session)
- Format texte simple parsable