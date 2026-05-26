# Techniques d'Implémentation

## 1. Rust Language (2021 Edition)

### Version
- **Edition**: 2021
- **Version cible**: stable

### Pourquoi utilisé
- Performance système
- Gestion mémoire sans GC
- Sécurité compile-time
- Interopérabilité avec bibliothèques C (libc)

---

## 2. clap (v4.5) - CLI Argument Parser

### Version
`clap = { version = "4.5", features = ["derive"] }`

### Pourquoi utilisé
- Parsing d'arguments moderne et ergonomique
- Génération automatique d'aide
- Support des sous-commandes

### Implémentation
```rust
#[derive(Parser)]
#[command(name = "wzllama", about = "Assistant IA locale", version = "0.3.0")]
pub struct Cli {
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Option<Command>,  // Enum des 8 commandes
}

#[derive(Subcommand)]
pub enum Command {
    Wizard, Validate, Bench, ResetTemplates, CheckI18n, Uninstall, Serve, InstallWebui, LaunchWebui
}
```

### Configuration
- `dry_run`: flag global pour simulation
- `visible_alias`: raccourcis (w, v, b, r, i, u, s)

### Alternatives
| Langage | Bibliothèque |
|---------|-------------|
| Python | Click, argparse |
| Go | Cobra, urfave/cli |
| Node.js | Commander.js, yargs |

---

## 3. dialoguer (v0.12) - Interface Terminal Interactive

### Version
`dialoguer = "0.12"`

### Pourquoi utilisé
- Sélection menus (Select)
- Confirmation (Confirm)
- Input utilisateur
- Fallback terminal propre

### Implémentation
```rust
// Menu sélection
Select::new()
    .with_prompt("Sélectionnez une option")
    .items(&items)
    .default(0)
    .interact_opt()?;

// Confirmation
Confirm::new()
    .with_prompt("Confirmer?")
    .default(true)
    .interact()?;
```

### Alternatives
| Langage | Bibliothèque |
|---------|-------------|
| Python | inquirer, questionary |
| Go | survey, promptui |
| Node.js | inquirer.js, enquirer |

---

## 4. axum (v0.8) - HTTP API Server

### Version
`axum = "0.8"`

### Pourquoi utilisé
- Framework async moderne
- Type-safe route handlers
- Intégration Tokio
- Support CORS (tower-http)

### Implémentation
```rust
// Route GET /api/v1/menu
Router::new()
    .route("/api/v1/menu", get(get_menu_handler))
    .route("/api/v1/tools", get(get_tools_handler))
    .layer(cors_layer)
```

### Configuration serveur
- **Port**: 1133
- **Address**: 0.0.0.0 (accessible réseau)
- **Timeout**: 30s

### Alternatives
| Langage | Bibliothèque |
|---------|-------------|
| Python | FastAPI, Flask |
| Go | Gin, Echo, Fiber |
| Node.js | Express, Fastify |
| Rust | Actix-web, Warp |

---

## 5. tokio (v1.0) - Runtime Asynchrone

### Version
`tokio = { version = "1.0", features = ["full"] }`

### Pourquoi utilisé
- Runtime async pour API serveur
- Non-bloquant pour les futures

### Features activées
- `full`: toutes les fonctionnalités (net, rt, sync, macros, io-util)

---

## 6. serde + serde_json + serde_yaml + toml

### Versions
- `serde = { version = "1.0", features = ["derive"] }`
- `serde_json = "1.0"`
- `serde_yaml = "0.9"`
- `toml = "0.8"`

### Pourquoi utilisés
- Sérialisation state.json
- Parsing fichiers config (TOML/YAML)
- API response JSON

### Implémentation
```rust
#[derive(Serialize, Deserialize)]
pub struct WzllamaState {
    pub language: Option<String>,
    pub installed: InstalledTools,
    pub last_model: Option<String>,
    // ...
}
```

---

## 7. reqwest (v0.12) - HTTP Client

### Version
`reqwest = { version = "0.12", features = ["blocking", "json"] }`

### Pourquoi utilisé
- Appels API Ollama (locale et distante)
- Scraping ollama.com/library
- API LLMFit

### Features
- `blocking`: pour appels synchrones
- `json`: serde integration

---

## 8. sysinfo (v0.39) - Détection Système

### Version
`sysinfo = "0.39"`

### Pourquoi utilisé
- Mémoire RAM totale/disponible
- Informations CPU
- Informations disque

### Implémentation
```rust
let mut sys = sysinfo::System::new_all();
sys.refresh_memory();
let ram_gb = sys.total_memory() as f64 / (1024^3);
```

---

## 9. llmfit-core - Bibliothèque Externe

### Source
`llmfit-core = { git = "https://github.com/AlexsJones/llmfit" }`

### Pourquoi utilisé
- Recommandations modèles basées hardware
- Filtrage par use case (coding, chat, etc.)

### API Client
```rust
let client = LLMFitClient::new();
if client.is_running() {
    let models = client.get_top_models(
        Some(20),    // limit
        None,        // min_memory_gb
        Some("coding") // use case
    );
}
```

---

## 10. scraper (v0.21) - Web Scraping

### Version
`scraper = "0.21"`

### Pourquoi utilisé
- Parsing HTML ollama.com/library
- Extraction liste modèles disponibles

### Implémentation
```rust
let document = Html::parse_document(&html);
let selector = Selector::parse("a[href^='/library/']").unwrap();
for element in document.select(&selector) {
    // Extract model names
}
```

---

## 11. libc (v0.2) - FFI Système Unix

### Version
`libc = "0.2"`

### Pourquoi utilisé
- `statvfs` pour espace disque
- Fork/exec pour sous-processus

---

## 12. dashmap (v6.1) - Concurrence Partagée

### Version
`dashmap = "6.1"`

### Pourquoi utilisé
- Partage state entre serveur API et CLI
- Map concurrente sans lock explicite

---

## 13. colored (v3.1) - Couleurs Terminal

### Version
`colored = "3.1"`

### Pourquoi utilisé
- Messages couleur (rouge=erreur, vert=succès)
- UI améliorée

### Implémentation
```rust
println!("📥 {}...", model.cyan().bold());
println!("   {} installé!", model.green());
```

---

## 14. tower + tower-http - Middleware HTTP

### Versions
- `tower = "0.5"`
- `tower-http = { version = "0.6", features = ["cors"] }`

### Pourquoi utilisé
- CORS pour requêtes cross-origin
- Composabilité middleware