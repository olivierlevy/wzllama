# Gestion des modèles

## Vue d'ensemble

wzllama gère les modèles IA via l'API Ollama avec un système de ranking basé sur l'usage et le hardware.

## API Ollama (src/core/ollama_api.rs)

### Fonctions principales

```rust
// Détecter l'URL Ollama
pub fn detect_url() -> Option<String>

// Lister modèles locaux
pub fn list_local_models() -> Result<Vec<OllamaModel>>

// Lister modèles distants (catalogue)
pub fn fetch_remote_catalog() -> Result<Vec<OllamaModel>>

// Fusionner et dédupliquer
pub fn merge_models(local: &[OllamaModel], remote: &[OllamaModel]) -> Vec<(OllamaModel, bool)>

// Pull un modèle
pub fn pull_model(name: &str) -> Result<()>

// Supprimer un modèle
pub fn delete_model(name: &str) -> Result<()>

// Créer un modèle personnalisé
pub fn create_model(name: &str, modelfile: &str) -> Result<()>

// Lister modèles en cours d'exécution
pub fn get_running_models() -> Vec<String>
```

## Structure OllamaModel

```rust
pub struct OllamaModel {
    pub name: String,
    pub size: Option<u64>,      // Bytes
    pub description: String,
    pub family: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}
```

## Ranking des modèles

### Usage types

```rust
pub enum TaskType {
    Code,      // Programmation
    Book,      // Texte long
    Agents,    // Agents IA
    Mixed,     // Usage général
}
```

### Critères de ranking

```rust
pub fn rank_models(
    models: &[OllamaModel],
    usage: &str,
    hw: &HardwareInfo,
    limit: usize
) -> Vec<(OllamaModel, f32)>
```

**Critères:**
1. **Taille modèle vs RAM disponible**
2. **VRAM GPU disponible** (si applicable)
3. **Quantification** (q4_0, q8_0, f16)
4. **Famille** adaptée à l'usage

### Exemples de classements

| Usage | Préférence |
|-------|-----------|
| Code | qwen2.5-coder, codestral, deepseek-coder |
| Book | qwen2.5, llama3.1, mistral |
| Fast Agents | qwen2.5:3b, llama3.2:3b, gemma2:9b |
| Mixed | qwen2.5:7b, llama3.1:8b, gemma2:27b |

## Modèles par défaut

```rust
impl Default for ModelsEnv {
    fn default() -> Self {
        Self {
            code: "qwen2.5-coder:14b".into(),
            book: "qwen2.5:14b".into(),
            agent: "qwen2.5:3b".into(),
            chat: "qwen2.5:7b".into(),
        }
    }
}
```

## Menu Modèles (menu_models.rs)

### Structure

```
🤖 Choisir un modèle IA
├── Modèles locaux
│   ├── qwen2.5-coder:14b (4.6GB) ✅
│   ├── llama3.1:8b (4.9GB)
│   └── ...
├── Télécharger depuis le catalogue
│   ├── qwen2.5-coder:32b (20GB)
│   ├── llama3.1:70b (43GB)
│   └── ...
└── Retour
```

### Actions disponibles

1. **Sélectionner** - Définir comme modèle actif
2. **Télécharger** - Pull depuis le registry
3. **Supprimer** - Delete du système

## Configuration de modèle (configurator.rs)

### Avant création/lancement

```
⚙ Configuration recommandée
   📐 Contexte : 16384 tokens
   💾 Cache KV : q8_0
   ⚡ Flash Attention : activé
   🌡 Température : 0.7

📄 Modelfile :
   FROM qwen2.5-coder:14b
   PARAMETER num_ctx 16384
   PARAMETER temperature 0.7

Action :
  💬 Lancer le chat
  📦 Créer un modèle personnalisé
  🚀 Créer une flotte d'agents
  ↩ Retour
```

### Modelfile généré

```
FROM {model_name}
PARAMETER num_ctx {context_length}
PARAMETER kv_cache_type {kv_cache_type}
PARAMETER flash_attention {true/false}
SYSTEM {system_prompt}
```

## Setup initial (setup_models.rs)

### Premier lancement

```rust
pub fn ensure_first_models(i18n: &I18n, hw: &HardwareInfo, state: &mut WzllamaState) -> Result<()> {
    let running = get_running_models();
    if running.is_empty() {
        // Proposer installation de modèles initiaux
        let (heavy, light) = recommend_models(hw);
        // Prompt installation
    }
}
```

### Modèles recommandés

| Hardware | Heavy (qualité) | Light (rapide) |
|----------|-----------------|----------------|
| 8GB RAM | qwen2.5:7b | qwen2.5:3b |
| 16GB RAM | qwen2.5:14b | qwen2.5:7b |
| 32GB+ RAM | qwen2.5:32b | qwen2.5-coder:14b |

## Nettoyage des modèles (cleanup_models.rs)

### Menu de suppression

```
🧹 Nettoyage > 🤖 Supprimer des modèles
├── 🗑️ qwen2.5-coder:14b
├── 🗑️ qwen2.5:7b
├── 🗑️ Supprimer tous les modèles wzllama
└── ↩ Retour
```

### Filtrage des modèles wzllama

```rust
pub fn list_wzllama_models() -> Vec<String> {
    // Retourne uniquement les modèles créés/utilisés par wzllama
    // Ex: wzllama-code, wzllama-chat, etc.
}
```

## Estimation de ressources (estimator.rs)

### Calcul tokens estimés

```
📊 50000 tokens estimés
⏱ 2 à 5 minutes
```

### Facteurs de calcul

- **Pages texte** → ~500 tokens/page
- **Lignes de code** → ~10 tokens/ligne
- **Avec contexte** → +30% tokens

## Détection modèle actif

```rust
pub fn detect_active_model(state: &WzllamaState) -> Option<String> {
    state.last_model.clone()
}

pub fn set_active_model(state: &mut WzllamaState, model: &str) {
    state.last_model = Some(model.to_string());
    state.save().ok();
}
```