# Fonctionnalités Completes de wzllama

## 1. Wizard - Sélection par Use Case

### Description
Le wizard guide l'utilisateur à travers un processus en 2 étapes pour sélectionner un modèle et un outil approprié selon son besoin.

### Inputs
| Type | Format | Contraintes |
|------|--------|-------------|
| `UseCase` enum | `General`, `Coding`, `Reasoning`, `Chat`, `Multimodal`, `Embedding` | 6 valeurs possibles |
| `HardwareInfo` | Détecté automatiquement au démarrage | RAM/VRAM requis |

### Outputs
| Type | Comportement |
|------|-------------|
| `Result<()>` | Retourne Ok(()) ou Erreur |
| Side effect | Modifie `state.last_model` et `state.last_tool` |

### Algorithme détaillé

```
FONCTION run(i18n, state, hw):
    BOUCLE:
        1. Afficher header avec ressources système (RAM/VRAM)
        2. Présenter les Use Cases disponibles:
           - Général
           - Coding
           - Reasoning
           - Chat
           - Multimodal
           - Embedding
        3. Si sélection "Retour" (position 0) → retourner Ok(())
        4. Sinon appeler handle_usecase_selection(use_case)
        
FONCTION handle_usecase_selection(i18n, state, hw, use_case):
    1. Récupérer modèles locaux via ollama_api::get_models()
    2. Récupérer modèles recommandés via llmfit_api::get_top_models(use_case)
    3. Si llmfit vide → fallback localmax_models::fetch_models_by_search()
    4. Construire liste des choix:
       - Retour (position 0)
       - Modèles installés (nom affiché)
       - Modèles disponibles (préfixe 📥 download)
       - Option "Lancer avec modèle actuel"
    5. Présenter menu Select à l'utilisateur
    6. Si modèle sélectionné:
       - Si installé: définir comme last_model
       - Si non installé: télécharger via ollama_api::pull_model()
    7. Appeler launch_tool_for_usecase(state, use_case, model)
    
FONCTION launch_tool_for_usecase(i18n, state, use_case, model):
    1. Appeler get_priority_tools_for_usecase(use_case, state)
    2. Filtrer par outils installés uniquement
    3. Si un seul outil disponible → le lancer directement
    4. Sinon présenter menu de sélection d'outil
    5. Lancer l'outil avec model en paramètre
```

### Cas limites
- Aucun modèle installé → message d'erreur
- Aucun outil installé pour le use case → message warning
- Erreur téléchargement modèle → affichage warning sans interruption

### Exemple d'utilisation
```
$ wzllama wizard
┌─ wzllama ─────────────────────────────┐
│ RAM: 32GB | VRAM: 8GB               │
└───────────────────────────────────────┘

? Sélectionnez un use case:
  ↩️ Retour
  📋 Général
  💻 Coding ← sélectionné
  🧠 Reasoning
  💬 Chat
  ...

? Sélectionnez un modèle:
  ↩️ Retour
  📥 qwen2.5-coder-7b (download)
  📥 deepseek-coder-33b (download)
  🤖 claude_code - Anthropic CLI
  ...

→ Installation du modèle + lancement de l'outil
```

---

## 2. Outils (Tools) - Gestion des IA

### Description
14 outils IA supportés avec installation, lancement et suivi de statut.

### Liste des outils
| ID | Name | Type | Agentic |
|----|------|------|---------|
| `ollama` | Ollama | LLM local | Non |
| `open_webui` | Open WebUI | Interface web | Non |
| `openclaw` | OpenClaw | Agent IA | Oui |
| `claude_code` | Claude Code | Agent IA | Oui |
| `hermes_agent` | Hermes | Agent IA | Oui |
| `opencode` | OpenCode | Agent IA | Oui |
| `codex` | Codex | Agent IA | Oui |
| `copilot_cli` | Copilot CLI | Agent IA | Oui |
| `droid` | Droid | Agent IA | Oui |
| `pi` | Pi | Agent IA | Oui |
| `pool` | Pool | Agent IA | Oui |
| `obsidian` | Obsidian | Wiki | Non |
| `goose` | Goose | Agent IA | Oui |
| `llmfit` | LLMFit | Recommandation | Non |

### Trait Tool (Interface)
```rust
pub trait Tool {
    fn id() -> &str;
    fn name() -> &str;
    fn description(i18n: &I18n) -> String;
    
    // Installation
    fn install(i18n: &I18n) -> Result<()>;
    fn update(i18n: &I18n) -> Result<()>;
    fn uninstall(i18n: &I18n) -> Result<()>;
    
    // Exécution
    fn launch(i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()>;
    
    // Properties
    fn is_installed() -> bool;
    fn requires_docker() -> bool { false }
    fn supports_agentic() -> bool { false }
}
```

### Algorithm d'installation
```
FONCTION install(i18n):
    1. Vérifier si déjà installé
    2. Afficher message "Installation..."
    3. Selon outil:
       - ollama: curl -fsSL https://ollama.com/install.sh | sh
       - docker: vérifier installation, sinon message d'instructions
       - open_webui: docker run ...
       - Autres (npm/pip/cargo): commandes spécifiques
    4. Marquer comme installé dans state.installed.{id} = true
    5. Sauvegarder state
```

---

## 3. Serveur API REST (Port 1133)

### Description
Serveur HTTP en arrière-plan fournissant l'API pour l'interface web ou TUI.

### Endpoints disponibles

| Méthode | Path | Description | Response |
|---------|------|-------------|----------|
| GET | `/api/v1/menu` | Structure menu complète | JSON MenuItem[] |
| GET | `/api/v1/tools` | Liste des outils | JSON ToolInfo[] |
| GET | `/api/v1/models` | Modèles locaux | JSON OllamaModel[] |
| GET | `/api/v1/hardware` | Ressources système | JSON |
| POST | `/api/v1/tools/{id}/install` | Installer outil | JSON Result |
| POST | `/api/v1/tools/{id}/launch` | Lancer outil | JSON Result |
| POST | `/api/v1/tools/{id}/uninstall` | Désinstaller | JSON Result |

### Exemple response `/api/v1/menu`
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
    },
    {
      "id": "models",
      "label": "🤖 Models",
      "type": "submenu",
      "children": [...]
    }
  ]
}
```

---

## 4. Menu Principal Interactif

### Description
Navigation hiérarchique avec les flèches, entrée et retour arrière.

### Structure du menu
```
Menu Principal
├── ▶ Reprendre (si last_tool/last_model définis)
├── 🧙 Wizard
│   ├── Usage: Coding
│   │   ├── 📦 claude_code
│   │   ├── 📦 opencode
│   │   └── ✅ ollama (toujours disponible)
│   ├── Usage: Chat
│   ├── Usage: Reasoning
│   ├── Usage: Multimodal
│   └── Usage: Embedding
├── 🤖 Models
│   ├── qwen2.5-coder-7b (default)
│   ├── deepseek-coder-33b
│   └── ...
├── 🔬 Scientific
│   ├── Chemistry
│   ├── Biology
│   └── Physics
├── 🛠️ Tools
│   ├── ✅ ollama - LLM local
│   ├── ✅ claude_code - Agent IA
│   └── ...
├── 🧹 Cleanup
│   ├── 🗑️ Nettoyer outils
│   └── 🗑️ Nettoyer modèles
└── ⚙️ Config
    ├── Performance settings
    ├── Ollama settings
    └── ...
```

### Navigation
- Flèches haut/bas pour sélectionner
- Entrée pour valider
- Position 0 = Retour (dans sous-menus)
- Dernière position = Quitter

---

## 5. Benchmark

### Description
Exécute un benchmark des modèles LLM locaux pour mesurer les performances.

### Inputs
| Type | Description |
|------|-------------|
| `dry_run` flag | Skip actual benchmark |

### Outputs
| Type | Format |
|------|--------|
| Résultats | Affichage console avec temps d'exécution |

### Algorithme
```
FONCTION run_benchmark():
    1. Récupérer liste des modèles installés
    2. Pour chaque modèle:
       - Exécuter prompt standard: "Why is the sky blue?"
       - Mesurer temps de réponse
       - Calculer tokens/second
    3. Afficher classement par performance
```

---

## 6. Validation Templates

### Description
Valide les templates de configuration embarqués.

### Commande
```
$ wzllama validate
```

### Algorithme
1. Parcourir les templates dans `config/templates/`
2. Parser chaque template
3. Vérifier la cohérence des références i18n
4. Afficher résultats

---

## 7. Reset Templates

### Description
Réinitialise les templates de configuration à leurs valeurs par défaut.

---

## 8. Check I18n

### Description
Vérifie l'intégrité des fichiers de traduction.

### Fichiers i18n
- `config/i18n/fr.json`
- `config/i18n/en.json`

---

## 9. Uninstall

### Description
Désinstalle wzllama CLI et nettoie les fichiers de configuration.

---

## 10. Gestion des Modèles

### Description
Interface avec l'API Ollama locale pour gérer les modèles.

### Fonctions clés
| Fonction | Description |
|----------|-------------|
| `get_models()` | Liste des modèles installés localement |
| `pull_model(name)` | Télécharge un nouveau modèle |
| `run_benchmark()` | Bench des performances |

---

## 11. Détection Matériel

### Description
Collecte les informations système (RAM, VRAM, CPU).

### Données collectées
- RAM disponible (GB)
- VRAM totale et disponible (MB)
- CPU model
- GPU model (nvidia/amd/intel)

### Utilisation
- Affichage header menu
- Recommandations modèles (llmfit)