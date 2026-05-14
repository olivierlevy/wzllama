# Guide de démarrage

## Prérequis

- **OS**: Linux (testé sur Arch, Ubuntu)
- **RAM**: 8GB minimum, 16GB recommandé
- **GPU**: Optionnel mais fortement recommandé (NVIDIA/AMD)
- **Docker**: Pour Open WebUI (optionnel)
- **Rust**: Pour compiler depuis les sources

## Installation

### Depuis les sources

```bash
# Cloner le repository
git clone https://github.com/yourusername/wzllama.git
cd wzllama

# Compiler
cargo build --release

# Installer (optionnel)
cargo install --path .
```

### Version release

```bash
# Télécharger la dernière release
wget https://github.com/yourusername/wzllama/releases/latest/download/wzllama
chmod +x wzllama
sudo mv wzllama /usr/local/bin/
```

## Première exécution

```bash
# Lancer wzllama (mode CLI par défaut)
wzllama

# Ou mode TUI si terminal suffisamment grand
wzllama --tui
```

### Étapes initiales

1. **Détection langue**: Sélectionnez votre langue (FR/EN)
2. **Détection hardware**: CPU/RAM/GPU analysés
3. **Ollama setup**: Installation et démarrage si nécessaire
4. **Modèles initiaux**: Choix de modèles recommandés selon hardware

## Configuration initiale

### Structure des répertoires

```
~/.wzllama/
├── config.yaml      # Configuration principale
├── env              # Fichier d'environnement (.env)
├── state.json       # État persistant
├── i18n/            # Fichiers de traduction
│   └── fr.json
└── fleets/          # Flottes OpenClaw
```

### Configuration YAML (config.yaml)

```yaml
ollama:
  host: "127.0.0.1:11434"
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

## Usage quotidien

### Workflow typique

```bash
# 1. Lancer wzllama
wzllama

# 2. Choisir "🤖 Choisir un modèle IA"
# 3. Sélectionner ou télécharger un modèle
# 4. Retour au menu principal
# 5. Choisir "🛠 Lancer un outil"
# 6. Sélectionner l'outil (ex: claude, openclaw)
```

### Variables d'environnement

Le fichier `~/.wzllama/env` contient toutes les variables nécessaires:

```bash
# Source du fichier
source ~/.wzllama/env

# Variables exportées
export OLLAMA_HOST='127.0.0.1:11434'
export OLLAMA_ORIGINS='http://localhost:*'
export OLLAMA_KEEP_ALIVE=-1
export OLLAMA_NO_CLOUD=1
export OLLAMA_FLASH_ATTENTION=1
export OLLAMA_KV_CACHE_TYPE=q8_0
export OLLAMA_CONTEXT_LENGTH=16384
export OPENAI_API_KEY='ollama'
export OPENAI_BASE_URL='http://localhost:11434/v1'
# ...
```

## Dépannage

### Ollama ne démarre pas

```bash
# Vérifier le service
systemctl status ollama

# Redémarrer manuellement
systemctl restart ollama

# Vérifier le port
ss -tlnp | grep 11434
```

### Docker pour Open WebUI

```bash
# Vérifier Docker
docker ps

# Démarrer Docker si nécessaire
sudo systemctl start docker

# Ajouter utilisateur au groupe docker
sudo usermod -aG docker $USER
# Se déconnecter/reconnecter
```

### Réinitialiser la configuration

```bash
# Sauvegarder
cp ~/.wzllama ~/.wzllama.backup

# Réinitialiser
rm -rf ~/.wzllama
wzllama  # Reconfiguration
```