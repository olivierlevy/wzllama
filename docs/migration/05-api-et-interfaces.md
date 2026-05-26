# API et Interfaces

## 1. CLI Interface - Point d'Entrée Principal

### Structure CLI
```rust
Cli {
    dry_run: bool,    // --dry-run flag
    command: Option<Command>  // Sous-commande
}

enum Command {
    Wizard,           // Menu interactif principal
    Validate,         // Valider templates
    Bench,            // Benchmark modèles
    ResetTemplates,    // Réinitialiser templates
    CheckI18n,        // Vérifier traductions
    Uninstall,        // Désinstaller wzllama
    Serve,            // Démarrer serveur API
    InstallWebui,     // Installer Open WebUI
    LaunchWebui,      // Lancer Open WebUI
}
```

### Méthodes CLI
| Signature | Description | Return |
|-----------|-------------|--------|
| `parse_args() -> Self` | Parse les arguments ligne de commande | Cli |
| `execute(&self) -> Result<()>` | Exécute la commande demandée | Result<()> |

### Exemples d'appel
```bash
wzllama                        # Wizard par défaut
wzllama wizard                 # Menu interactif
wzllama --dry-run wizard       # Mode simulation
wzllama serve                  # API serveur sur port 1133
wzllama install-webui          # Installer Open WebUI
```

---

## 2. API HTTP - Endpoints REST (Port 1133)

### Base URL
```
http://localhost:1133
```

### Menu Endpoints

#### GET `/api/v1/menu`
- **Description**: Retourne la structure complète du menu
- **Response**: `application/json`
```json
{
  "id": "main",
  "label": "Menu principal",
  "type": "menu",
  "items": [
    {
      "id": "wizard",
      "label": "🧙 Wizard",
      "type": "submenu",
      "children": [...]
    }
  ]
}
```

#### GET `/api/v1/menu/{id}`
- **Description**: Retourne un sous-menu spécifique
- **Path Params**: `id` (string) - identifiant du menu
- **Response**: MenuItem avec enfants

#### POST `/api/v1/menu/{id}/select`
- **Description**: Exécute une action menu
- **Response**: 
```json
{
  "action": "submenu|install_tool|launch_tool|select_model|quit",
  "target": "string",      // pour submenu
  "tool_id": "string",    // pour tool actions
  "model": "string"       // pour select_model
}
```

### Tool Endpoints

#### GET `/api/v1/tools`
- **Description**: Liste tous les outils disponibles
- **Response**:
```json
[
  {
    "id": "ollama",
    "name": "Ollama",
    "description": "LLM local...",
    "installed": true,
    "status": "installed",
    "supports_agentic": false,
    "requires_docker": false
  }
]
```

#### GET `/api/v1/tools/{id}`
- **Description**: Détails d'un outil spécifique
- **Response**: ToolInfo

#### POST `/api/v1/tools/{id}/install`
- **Description**: Installe l'outil
- **Response**: ActionResponse

#### POST `/api/v1/tools/{id}/update`
- **Description**: Met à jour l'outil

#### POST `/api/v1/tools/{id}/uninstall`
- **Description**: Désinstalle l'outil

#### GET `/api/v1/tools/{id}/status`
- **Description**: Vérifie le statut d'installation
- **Response**:
```json
{
  "id": "string",
  "installed": true,
  "status": "installed|not_installed"
}
```

### Model Endpoints

#### GET `/api/v1/models`
- **Description**: Liste les modèles installés localement
- **Response**: Menu avec items modèles

#### POST `/api/v1/models/{name}/pull`
- **Description**: Télécharge un modèle

#### DELETE `/api/v1/models/{name}/delete`
- **Description**: Supprime un modèle

### System Endpoints

#### GET `/api/v1/status`
- **Description**: État système global
- **Response**:
```json
{
  "status": "ok",
  "ollama": "running|stopped"
}
```

#### GET `/api/v1/hardware`
- **Description**: Informations matérielles
- **Response**:
```json
{
  "ram_gb": 32.0,
  "has_gpu": true,
  "gpus": [
    {"name": "NVIDIA RTX 4090", "vram_mb": 24576}
  ]
}
```

#### GET `/health`
- **Description**: Health check simple
- **Response**: `"OK"` (text/plain)

---

## 3. Trait Tool - Interface Outils

### Signature complète
```rust
pub trait Tool: Send + Sync {
    // Identification
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    
    // Internationalisation
    fn description(&self, i18n: &I18n) -> String;
    
    // Statut
    fn status(&self, state: &WzllamaState) -> ToolStatus;
    fn status_message(&self, i18n: &I18n) -> String;
    
    // Installation
    fn install(&self, i18n: &I18n) -> Result<()>;
    fn update(&self, i18n: &I18n) -> Result<()>;
    fn uninstall(&self, i18n: &I18n) -> Result<()>;
    
    // Exécution
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()>;
    
    // Properties (default implementations)
    fn requires_docker(&self) -> bool { false }
    fn supports_agentic(&self) -> bool { false }
}
```

### Valeurs de retour
| Type | Description |
|------|-------------|
| `Result<()>` | Ok(()) ou anyhow::Error |

### Implémentations disponibles
- `OllamaTool`
- `OpenWebUITool`
- `OpenClawTool`
- `ClaudeCodeTool`
- `HermesTool`
- `OpenCodeTool`
- `CodexTool`
- `CopilotCliTool`
- `DroidTool`
- `PiTool`
- `PoolTool`
- `ObsidianTool`
- `GooseTool`
- `LLMFitTool`

---

## 4. Trait ToolAction - Actions Menu

### Signature
```rust
pub trait ToolAction: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn execute(&self, ctx: &ActionContext) -> Result<ActionResult>;
    fn validate(&self, ctx: &ActionContext) -> Result<()> { Ok(()) }
    fn requires_confirmation(&self) -> bool { false }
    fn confirmation_message(&self) -> Option<&str> { None }
}
```

---

## 5. MenuTree Methods

### Signature complète
```rust
impl MenuTree {
    pub fn new(root_label: &str) -> Self;
    pub fn with_title(root_label: &str, title: &str) -> Self;
    pub fn with_root(self, root: MenuItem) -> Self;
    pub fn with_metadata(self, metadata: MenuMetadata) -> Self;
    
    pub fn find_by_path(&self, path: &str) -> Option<&MenuItem>;
    pub fn get_leaf_items(&self) -> Vec<&MenuItem>;
    pub fn get_flat_items(&self) -> Vec<(String, &MenuItem)>;
}
```

---

## 6. MenuItem Methods

### Signature complète
```rust
impl MenuItem {
    pub fn leaf(label: &str) -> Self;
    pub fn branch(label: &str) -> Self;
    pub fn with_action(self, action_id: &str) -> Self;
    pub fn with_action_string(self, action_id: String) -> Self;
    pub fn add_submenu(self, item: MenuItem) -> Self;
    pub fn add_submenus(self, items: Vec<MenuItem>) -> Self;
    
    pub fn is_leaf(&self) -> bool;
    pub fn has_action(&self) -> bool;
    pub fn formatted_label(&self) -> String;
}
```

---

## 7. ActionDispatcher Methods

### Signature complète
```rust
impl ActionDispatcher {
    pub fn new() -> Self;
    pub fn register(&mut self, action: Box<dyn ToolAction>);
    pub fn get(&self, id: &str) -> Option<&dyn ToolAction>;
    pub fn execute(&self, id: &str, ctx: &ActionContext) -> Result<ActionResult>;
    pub fn list_ids(&self) -> Vec<&str>;
}
```

---

## 8. Function APIs Principales

### Core - ollama_api
```rust
pub fn get_models() -> Vec<OllamaModel>;
pub fn fetch_local_models(base_url: &str) -> Result<Vec<OllamaModel>>;
pub fn detect_url() -> Option<String>;
pub fn pull_model(model: &str) -> Result<()>;
pub fn delete_model(name: &str) -> Result<()>;
pub fn show_model(model_name: &str) -> Result<ModelShowResponse>;
```

### Config - state
```rust
pub fn load() -> WzllamaState;
pub fn save(state: &WzllamaState) -> Result<()>;
pub fn mark_installed(tool: &str, state: &mut WzllamaState);
pub fn set_language(lang: &str, state: &mut WzllamaState);
pub fn set_last_model(model: &str, state: &mut WzllamaState);
pub fn set_last_tool(tool: &str, state: &mut WzllamaState);
```

### Config - i18n
```rust
pub fn t(&self, key: &str) -> String;
pub fn t_with_vars(&self, key: &str, vars: &[(&str, &str)]) -> String;
pub fn get_available_languages() -> Vec<LanguageMeta>;
pub fn detect_system_language() -> String;
pub fn load(lang_code: &str) -> Result<I18n>;
pub fn check_integrity() -> Result<()>;
```

### Core - hardware
```rust
pub fn detect() -> HardwareInfo;
pub fn get_available_disk_space_gb(path: &str) -> f64;
```

### Tools
```rust
pub fn get_all_tools() -> Vec<Box<dyn Tool>>;
pub fn get_tool(id: &str) -> Option<Box<dyn Tool>>;
pub fn get_available_tools(state: &WzllamaState, i18n: &I18n) -> Vec<ToolInfo>;
```

---

## 9. Types de données API

### ActionResponse
```rust
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}
```

### SystemStatus
```rust
pub struct SystemStatus {
    pub status: String,
    pub ollama: String,  // "running" or "stopped"
}
```