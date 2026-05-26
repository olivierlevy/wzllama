# Structures de Données

## 1. MenuTree - Arbre de Menus

### Définition
```rust
pub struct MenuTree {
    pub root: MenuItem,                          // Racine du menu
    pub metadata: MenuMetadata,                  // Métadonnées
}

pub struct MenuMetadata {
    pub title: Option<String>,                 // Titre menu
    pub description: Option<String>,             // Description
    pub version: Option<String>,                 // Version config
}
```

### MenuConfig - Configuration externe
```rust
pub struct MenuConfig {
    pub version: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub items: Vec<MenuConfigItem>,              // Items enfants
}

pub struct MenuConfigItem {
    pub label: String,                           // Texte affiché
    pub action_id: Option<String>,               // ID action associée
    pub children: Option<Vec<MenuConfigItem>>,   // Sous-menus
    pub condition: Option<String>,              // Condition d'affichage
}
```

### MenuItem - Élément de Menu
```rust
pub struct MenuItem {
    pub label: String,                           // Libellé affichage
    pub action_id: Option<String>,               // Action à exécuter
    pub submenus: Vec<MenuItem>,                 // Enfants (vide si leaf)
    pub label_vars: HashMap<String, String>,     // Variables interpolation
}
```

### NavigationState - État Navigation
```rust
pub struct NavigationState {
    pub history: Vec<usize>,                     // Historique positions
    pub current_index: usize,                    // Index actuel
}
```

---

## 2. WzllamaState - État Persistant

### Définition
```rust
pub struct WzllamaState {
    pub language: Option<String>,                // Langue UI (fr/en/etc)
    pub installed: InstalledTools,              // Statut outils
    pub last_model: Option<String>,              // Dernier modèle utilisé
    pub last_usage: Option<String>,              // Dernière utilisation
    pub last_tool: Option<String>,               // Dernier outil utilisé
}
```

### InstalledTools
```rust
pub struct InstalledTools {
    pub docker: bool,
    pub ollama: bool,
    pub open_webui: bool,
    pub openclaw: bool,
    pub claude_code: bool,
    pub hermes_agent: bool,
    pub opencode: bool,
    pub codex: bool,
    pub copilot_cli: bool,
    pub droid: bool,
    pub pi: bool,
    pub pool: bool,
    pub obsidian: bool,
    pub goose: bool,
    pub llmfit: bool,
}
```

### Stockage
- **Fichier**: `~/.config/wzllama/state.json`
- **Format**: JSON (pretty-printed)

---

## 3. HardwareInfo - Informations Système

### Définition
```rust
pub struct HardwareInfo {
    pub os: String,                              // "linux x86_64"
    pub ram_gb: f64,                          // RAM totale en GB
    pub total_vram_mb: u64,                   // VRAM totale en MB
    pub gpus: Vec<GpuInfo>,                   // Liste GPU détectés
    pub available_disk_gb: f64,                // Espace disque disponible
}

pub struct GpuInfo {
    pub name: String,                          // "NVIDIA RTX 4090"
    pub vram_mb: u64,                        // VRAM GPU
}
```

---

## 4. ToolInfo - Informations Outil

### Définition
```rust
pub struct ToolInfo {
    pub id: String,                            // Identifiant unique
    pub name: String,                          // Nom affichage
    pub description: String,                     // Description i18n
    pub installed: bool,                       // Statut installation
}
```

---

## 5. OllamaModel - Modèle LLM

### Définition
```rust
pub struct OllamaModel {
    pub name: String,                          // Nom complet (ex: "qwen2.5:7b")
    pub model: String,                         
    pub modified_at: Option<String>,            
    pub size: Option<u64>,                      // Taille en bytes
    pub details: Option<ModelDetails>,           // Détails modèle
}

pub struct ModelDetails {
    pub family: Option<String>,                  // "qwen2", "llama"
    pub parameter_size: Option<String>,          // "7B", "24B"
    pub quantization_level: Option<String>,      // "Q4_K_M"
}
```

---

## 6. ToolStatus - Enum Statut

```rust
pub enum ToolStatus {
    Installed,
    NotInstalled,
}

impl ToolStatus {
    pub fn from_installed(installed: bool) -> Self {
        if installed { Installed } else { NotInstalled }
    }
}
```

---

## 7. UseCase - Enum Catégories

```rust
pub enum UseCase {
    General,
    Coding,
    Reasoning,
    Chat,
    Multimodal,
    Embedding,
}

impl UseCase {
    pub fn as_str(&self) -> &'static str {
        match self {
            General => "general",
            Coding => "coding",
            Reasoning => "reasoning",
            Chat => "chat",
            Multimodal => "multimodal",
            Embedding => "embedding",
        }
    }
}
```

---

## 8. ActionContext - Contexte d'Action

```rust
pub struct ActionContext {
    pub params: HashMap<String, String>,         // Paramètres
    pub state: Option<serde_json::Value>,         // État serialisé
}
```

---

## 9. ActionResult - Résultat d'Action

```rust
pub struct ActionResult {
    pub success: bool,
    pub message: Option<String>,
}
```

---

## 10. EnvConfig - Configuration Environnement

```rust
pub struct EnvConfig {
    pub ollama: OllamaEnv,
    pub performance: PerformanceSettings,
}

pub struct OllamaEnv {
    pub origins: String,
    pub keep_alive: String,
    pub num_parallel: u32,
    pub max_loaded_models: u32,
    pub flash_attention: bool,
    pub kv_cache_type: String,
    pub context_length: u32,
    pub max_vram: u32,
    pub cuda_visible_devices: String,
}

pub struct PerformanceSettings {
    pub cpu_threads: u32,
}
```

---

## 11. LLMFitModel - Modèle Recommandé

```rust
pub struct LLMFitModel {
    pub name: String,
    pub memory_required_gb: f64,
    pub use_cases: Vec<String>,
    pub strength: f64,
    pub speed: f64,
}
```

---

## 12. ScientificCategory - Catégories Scientifiques

```rust
pub struct ScientificCategory {
    pub name_key: String,          // "scientific.chemistry"
    pub icon: &'static str,        // "🧪"
    pub tools: Vec<&'static str>,   // ["moleculer", "chemskill"]
    pub skills: Vec<&'static str>,  // Skills MCP
}
```

---

## 13. AgenticToolInfo - Outils Agentiques

```rust
pub struct AgenticToolInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
}
```

---

## 14. LanguageMeta - Métadonnées Langue

```rust
pub struct LanguageMeta {
    pub code: String,              // "fr", "en"
    pub name: String,              // "Français", "English"
}
```

---

## 15. Relations entre structures

```
WzllamaState 1 ── * InstalledTools
WzllamaState 1 ── 0..1 MenuItem (last_model/last_tool)

MenuTree 1 ── 1 MenuItem (root)
MenuItem 1 ── 0..* MenuItem (submenus)
MenuItem 0..1 ── String (action_id)

ActionContext * ── HashMap params
ActionDispatcher 1 ── * ToolAction
ActionResult 1 ── bool + message

HardwareInfo 1 ── * GpuInfo
OllamaModel 0..1 ── ModelDetails
```