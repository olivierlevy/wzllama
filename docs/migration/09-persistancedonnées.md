# Persistance des Données

## 1. État Persistant (State)

### Fichier de sauvegarde
```
~/.config/wzllama/state.json
```

### Structure JSON
```json
{
  "language": "fr",
  "installed": {
    "docker": true,
    "ollama": true,
    "open_webui": false,
    "openclaw": false,
    "claude_code": true,
    "hermes_agent": false,
    "opencode": false,
    "codex": false,
    "copilot_cli": false,
    "droid": false,
    "pi": false,
    "pool": false,
    "obsidian": false,
    "goose": false,
    "llmfit": true
  },
  "last_model": "qwen2.5:7b",
  "last_usage": "2024-01-15T10:30:00",
  "last_tool": "claude_code"
}
```

### Format
- **Type**: JSON
- **Indentation**: 2 espaces (pretty-printed)
- **Encodage**: UTF-8

### Opérations
| Opération | Fonction | Description |
|-----------|----------|-------------|
| Lire | `load()` | Charge depuis fichier ou retourne défaut |
| Écrire | `save(state)` | Sauvegarde avec pretty format |
| Marquer installé | `mark_installed(tool_id, state)` | Met à jour flag installed |
| Définir langue | `set_language(lang, state)` | Stocke préférence langue |

---

## 2. Configuration Environnement

### Fichier
```
~/.config/wzllama/env.json
```

### Structure
```rust
pub struct EnvConfig {
    pub ollama: OllamaEnv,
    pub performance: PerformanceSettings,
}

pub struct OllamaEnv {
    pub origins: String,              // "*" par défaut
    pub keep_alive: String,           // "5m" par défaut
    pub num_parallel: u32,            // 4 par défaut
    pub max_loaded_models: u32,       // 4 par défaut
    pub flash_attention: bool,        // true par défaut
    pub kv_cache_type: String,        // "q8_0" par défaut
    pub context_length: u32,          // 4096 par défaut
    pub max_vram: u32,                // 0 (unlimited) par défaut
    pub cuda_visible_devices: String,   // "" par défaut
}
```

### Génération
- Créé lors de la première exécution
- Utilisé pour override systemd ollama.service

---

## 3. Fichiers Internationalization

### Emplacement
```
~/.config/wzllama/i18n/
  ├── fr.json
  ├── en.json
  └── ... (autres langues)
```

### Structure JSON par fichier
```json
{
  "_language": {
    "code": "fr",
    "name": "Français",
    "name_en": "French",
    "direction": "ltr"
  },
  "menu.main.title": "Menu principal",
  "menu.main.wizard": "Wizard IA",
  "wizard.usecase.coding": "Coding",
  "wizard.usecase.chat": "Chat",
  "tool.installed": "Installé",
  "tool.not_installed": "Non installé"
}
```

### Chemin de fallback
- `~/.config/wzllama/i18n/*.json`
- `config/i18n/*.json` (embarqué dans binaire)
- Retour clé si traduction manquante

---

## 4. Templates de Configuration

### Emplacement
```
config/templates/
```

### Structure
- Fichiers embarqués
- Templates pour différents outils
- Valeurs par défaut personnalisables

---

## 5. Configuration MCP (Model Context Protocol)

### Emplacement
```
~/.config/wzllama/mcp/
config/mcp/
```

### Format
- Configuration serveur MCP
- Endpoints et authentification
- Registry outils externes

---

## 6. Logs

### Emplacement
```
~/.config/wzllama/wzllama.log
~/.wzllama/log/wzllama.log
```

### Rotation
- Pas de rotation automatique
- Overwrite à chaque session (env_logger)

---

## 7. Modèles LLM

### Emplacement
```
/home/ollama/            # Linux par défaut
~/.ollama/               # Alternative
/usr/share/ollama/         # Alternative
```

### Gestion
- **Lecture**: via `ollama_api::get_models()` → API locale
- **Écriture**: `ollama pull {model}` → téléchargement
- **Suppression**: `ollama rm {model}` → suppression

---

## 8. Menus Externes (TOML)

### Emplacement potentiel
```
config/menus/
  ├── main.toml
  ├── wizard.toml
  ├── tools.toml
  └── config.toml
```

### Structure TOML
```toml
version = "1.0"
title = "Menu Principal"

[[items]]
label = "Wizard"
action_id = "wizard"

[[items]]
label = "Tools"
action_id = "tools"

[[items]]
label = "Models"
action_id = "models"
children = [
    { label = "Local", action_id = "list_local_models" },
    { label = "Download", action_id = "list_remote_models" }
]
```

---

## 9. Schéma des Données

```
┌─────────────────────────────────────────────────────────────┐
│                    ~/.config/wzllama/                       │
├─────────────────────────────────────────────────────────────┤
│ state.json          ← État utilisateur (JSON)                │
│ env.json            ← Config environnement (JSON)           │
│ wzllama.log         ← Logs application                     │
│ i18n/               ← Traductions                          │
│   ├── fr.json                                               │
│   └── en.json                                               │
│ mcp/                ← Config Protocol                        │
└─────────────────────────────────────────────────────────────┘
```