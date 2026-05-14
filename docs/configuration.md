# Configuration

## Vue d'ensemble

wzllama utilise un système de configuration à deux niveaux:
1. **config.yaml** - Fichier de configuration principale
2. **state.json** - État persistant de l'application

## Structure config.yaml

```yaml
ollama:
  host: "127.0.0.1:11434"
  origins: "http://localhost:*"
  keep_alive: -1
  no_cloud: true
  num_parallel: 4
  max_loaded_models: 3
  flash_attention: true
  kv_cache_type: "q8_0"
  context_length: 16384

providers:
  openai:
    api_key: "ollama"
    base_url: "http://localhost:11434/v1"
  anthropic:
    api_key: "ollama"
    base_url: "http://localhost:11434/v1"

openclaw:
  api_key: "ollama-local"

models:
  code: "qwen2.5-coder:14b"
  book: "qwen2.5:14b"
  agent: "qwen2.5:3b"
  chat: "qwen2.5:7b"
```

## Champs de configuration Ollama

| Champ | Type | Défaut | Description |
|-------|------|--------|-------------|
| `host` | String | "127.0.0.1:11434" | Adresse du serveur Ollama |
| `origins` | String | "http://localhost:*" | CORS origins autorisés |
| `keep_alive` | i32 | -1 | -1 = infini, 0 = unload, N = minutes |
| `no_cloud` | bool | true | Forcer l'utilisation locale |
| `num_parallel` | u32 | 4 | Requêtes parallèles |
| `max_loaded_models` | u32 | 3 | Modèles en RAM max |
| `flash_attention` | bool | true | Optimisation VRAM |
| `kv_cache_type` | String | "q8_0" | Type cache KV |
| `context_length` | u32 | 16384 | Longueur contexte tokens |

## Génération du fichier env

Le fichier `~/.wzllama/env` est généré automatiquement:

```bash
# Généré par EnvConfig::generate_env_file()
export OLLAMA_HOST='127.0.0.1:11434'
export OLLAMA_ORIGINS='http://localhost:*'
export OLLAMA_KEEP_ALIVE=-1
export OLLAMA_NO_CLOUD=1
export OLLAMA_NUM_PARALLEL=4
export OLLAMA_MAX_LOADED_MODELS=3
export OLLAMA_FLASH_ATTENTION=1
export OLLAMA_KV_CACHE_TYPE=q8_0
export OLLAMA_CONTEXT_LENGTH=16384
export OPENAI_API_KEY='ollama'
export OPENAI_BASE_URL='http://localhost:11434/v1'
export ANTHROPIC_API_KEY='ollama'
export ANTHROPIC_BASE_URL='http://localhost:11434/v1'
export OLLAMA_API_KEY='ollama-local'
export WZLLAMA_HOME='...'
export WZLLAMA_LANG='fr'
export WZLLAMA_MODEL_CODE='qwen2.5-coder:14b'
export WZLLAMA_MODEL_BOOK='qwen2.5:14b'
export WZLLAMA_MODEL_AGENT='qwen2.5:3b'
export WZLLAMA_MODEL_CHAT='qwen2.5:7b'
```

## État persistant (state.json)

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

## Configuration par hardware

### Détection hardware

```rust
pub struct HardwareInfo {
    pub os: String,
    pub ram_gb: f64,
    pub gpus: Vec<GpuInfo>,
    pub total_vram_mb: u64,
}

pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
}
```

### Configuration adaptative

```rust
impl EnvConfig {
    pub fn default_for_hardware(hw: &HardwareInfo) -> Self {
        // Sélectionne les meilleurs modèles selon:
        // - RAM disponible
        // - VRAM GPU
        // - Architecture CPU/GPU
    }
}
```

## Menu Configuration

### Accès

```
wzllama
→ ⚙ Configuration
```

### Options

1. **Modèles par usage** (`config.models`)
   - Code, Livre, Agent, Chat
   - Modèles sélectionnés selon hardware

2. **Performance** (`config.performance`)
   - Contexte (4K-64K tokens)
   - Cache KV type (f16/q8_0/q4_0)
   - Flash Attention toggle
   - Cloud models toggle

3. **Shells** (`config.shells`)
   - Install completions
   - Uninstall completions

4. **Regénérer env** (`config.regenerate_env`)
   - Recrée `~/.wzllama/env`

5. **Désinstaller wzllama** (`config.uninstall_wzllama`)
   - Supprime configuration et modèles

## Variables d'environnement supportées

| Variable | Source | Description |
|----------|--------|-------------|
| `OLLAMA_HOST` | config.yaml | Adresse Ollama |
| `OLLAMA_ORIGINS` | config.yaml | CORS |
| `OLLAMA_KEEP_ALIVE` | config.yaml | TTL modèles |
| `OLLAMA_NO_CLOUD` | config.yaml | Force local |
| `OLLAMA_FLASH_ATTENTION` | config.yaml | Optimisation VRAM |
| `OLLAMA_KV_CACHE_TYPE` | config.yaml | Type cache |
| `OLLAMA_CONTEXT_LENGTH` | config.yaml | Contexte tokens |
| `OPENAI_API_KEY` | config.yaml | Pour providers |
| `OPENAI_BASE_URL` | config.yaml | URL Ollama OAI compat |
| `WZLLAMA_LANG` | state.json | Langue UI |
| `WZLLAMA_MODEL_*` | config.yaml | Modèles par usage |

## Chemins de configuration

```rust
pub fn config_path() -> PathBuf {
    paths::config_dir().join("config.yaml")
}

pub fn env_path() -> PathBuf {
    paths::wzllama_dir().join("env")
}

pub fn state_path() -> PathBuf {
    paths::wzllama_dir().join("state.json")
}
```

## Réinitialisation

```bash
# Sauvegarder
cp ~/.wzllama ~/.wzllama.backup

# Réinitialiser complètement
rm -rf ~/.wzllama
wzllama
```