# Configuration et Environnement

## 1. Variables d'Environnement

### Variables Système
| Variable | Description | Exemple |
|----------|-------------|---------|
| `WZLLAMA_LANG` | Force la langue (contourne state) | `en`, `fr`, `es` |
| `WZLLAMA_HOME` | Répertoire configuratif | `~/.wzllama` |
| `HOME` | Répertoire utilisateur | `/home/user` |

### Variables Ollama (générées)
| Variable | Description | Défaut |
|----------|-------------|--------|
| `OLLAMA_HOST` | URL serveur Ollama | `127.0.0.1:11434` |
| `OLLAMA_MODELS` | Répertoire modèles | `/home/ollama` |
| `OLLAMA_ORIGINS` | CORS origins | `http://localhost:*` |
| `OLLAMA_KEEP_ALIVE` | Keep alive duration | `-1` (infini) |
| `OLLAMA_NO_CLOUD` | Désactiver cloud | `1` |
| `OLLAMA_NUM_PARALLEL` | Requêtes parallèles | `4` |
| `OLLAMA_MAX_LOADED_MODELS` | Modèles en mémoire | `3` |
| `OLLAMA_FLASH_ATTENTION` | Flash attention | `1` |
| `OLLAMA_KV_CACHE_TYPE` | Type cache KV | `q8_0` |
| `OLLAMA_CONTEXT_LENGTH` | Contexte tokens | `16384` |
| `OLLAMA_MAX_VRAM` | VRAM max (bytes) | `0` (auto) |

### Variables Providers
| Variable | Description | Défaut |
|----------|-------------|--------|
| `OPENAI_API_KEY` | Clé API OpenAI | `ollama` (local) |
| `OPENAI_BASE_URL` | URL API OpenAI | `http://localhost:11434/v1` |
| `ANTHROPIC_API_KEY` | Clé API Anthropic | `ollama` (local) |
| `ANTHROPIC_BASE_URL` | URL API Anthropic | `http://localhost:11434/v1` |
| `OLLAMA_API_KEY` | Clé OpenClaw | `ollama-local` |

---

## 2. Fichiers de Configuration

### Structure des Dossiers
```
~/.wzllama/
├── config/
│   └── config.yaml          # EnvConfig sérialisé
├── i18n/
│   ├── fr.json              # Traductions françaises
│   ├── en.json              # Traductions anglaises
│   └── ...                  # Autres langues
├── logs/
│   └── wzllama.log          # Logs application
├── state.json               # État utilisateur
└── env                      # Script shell (.sh)
```

### config.yaml
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
  max_vram: 0
  cuda_visible_devices: ""

providers:
  openai:
    api_key: "ollama"
    base_url: "http://localhost:11434/v1"
  anthropic:
    api_key: "ollama"
    base_url: "http://localhost:11434/v1"

openclaw:
  api_key: "ollama-local"
```

### Fichier `env` (shell script)
```bash
# wzllama - Environnement IA 100% locale
# Généré le 2024-01-15

# ═══ Ollama ═══════════════════════════════
export OLLAMA_HOST='127.0.0.1:11434'
export OLLAMA_ORIGINS='http://localhost:*'
export OLLAMA_KEEP_ALIVE=-1
export OLLAMA_NO_CLOUD=1
export OLLAMA_NUM_PARALLEL=4
# ... etc

# ═══ Providers ════════════════════════════
export OPENAI_API_KEY='ollama'
export OPENAI_BASE_URL='http://localhost:11434/v1'
export ANTHROPIC_API_KEY='ollama'

# ═══ wzllama ══════════════════════════════
export WZLLAMA_HOME='~/.wzllama'
export WZLLAMA_LANG='fr'
```

---

## 3. Prérequis Système

### OS Supportés
- **Linux** (principalement testé)
- **macOS** (partiellement supporté)
- **Windows** (théorique via WSL)

### Dépendances Système
| Dépendance | Usage | Obligatoire |
|------------|-------|-------------|
| `curl` | Installation outils | Pour install auto |
| `docker` | Open WebUI | Pour open_webui |
| `npm` | Certains tools | Pour npm-based tools |
| `pip` | Certains tools | Pour python tools |
| `cargo` | Certains tools | Pour cargo-based tools |
| `systemd` | Service ollama | Pour Linux |

### Bibliothèques Système
- `libc` (pour statvfs, exec)
- `sysinfo` (pour RAM/GPU via sysfs)

---

## 4. Ports Réseau

| Port | Service | Description |
|------|---------|-------------|
| 11434 | Ollama | API LLM locale |
| 1133 | wzllama API | Serveur REST wzllama |
| 8080 | Open WebUI | Interface web (Docker) |

---

## 5. Secrets et Credentials

### Pas de secrets externes requis
- **LLMFit**: Pas d'authentification (service local)
- **Ollama**: Pas d'authentification par défaut
- **API Providers**: Valeurs par défaut vers Ollama local

### Stockage
- Pas de stockage de mots de passe
- API keys "ollama" comme placeholder local
- Configuration world-readable (pas de chmod 600)

---

## 6. Shell Configuration Integration

### Méthode d'installation
```rust
// src/config/shells.rs
pub fn install_all_shells(i18n: &I18n) -> Result<()> {
    for shell in &["bash", "zsh", "fish"] {
        install_for_shell(shell)?;
    }
}
```

### Fichiers concernés
| Shell | Fichier |
|-------|---------|
| bash | `~/.bashrc` |
| zsh | `~/.zshrc` |
| fish | `~/.config/fish/config.fish` |

### Source du fichier env
```bash
# Ajouté à la fin du .bashrc etc
echo "source ~/.wzllama/env" >> ~/.bashrc
```

---

## 7. Configuration Docker

### Variables Docker
```yaml
environment:
  - OLLAMA_BASE_URL=http://127.0.0.1:11434
  - OLLAMA_MODELS=/home/ollama
```

### Volumes
```
open-webui:/app/backend/data  # Persistance données
```

### Network
```
--network=host              # Accès direct au réseau
--add-host=host.docker.internal:host-gateway
```