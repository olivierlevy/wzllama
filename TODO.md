Prompt pour la transformation du code :
Je souhaite réorganiser mon code Rust CLI (écrit en vibe coding) en une API locale gérant :
1. Arbre de menus dynamique :
- Structure hiérarchique de menus/submenus (ex : "Installer" → "Logiciel A", "Logiciel B")
- Externaliser la définition des menus dans une structure de données (ex : MenuTree contenant MenuItems)
- Supporter des sous-menus imbriqués et des commandes terminales
2. Gestion des commandes via le trait Tool :
- Externaliser la logique des commandes (install/lancer/désinstaller) dans un système de dispatch
- Associer chaque menu item à un Tool::execute avec des paramètres dynamiques
- Gérer les erreurs de commande et les validations d'entrée utilisateur
3. UI/UX avancé :
- Affichage des menus en mode texte interactif (ex : clap ou crossterm)
- Navigation via clavier (flèches, entrée, retour arrière)
- Gestion des menus imbriqués avec retour automatique
4. API de base :
- Créer un module menu_api contenant :
- MenuTree : structure hiérarchique
- MenuItem : contient le texte, la commande (fonction closure), et les sous-menus
- MenuHandler : gestionnaire principal de l'arbre
- Exporter des fonctions pour :
- Construire l'arbre à partir de configurations
- Lancer l'interpréteur de menus
- Exécuter des commandes via le Tool::execute
5. Externalisation maximale :
- Séparer les menus de la logique métier
- Permettre des configurations externes (ex : fichier TOML/JSON pour les menus)
- Garantir la compatibilité avec différents Tool implémentant des actions spécifiques
6. Exemples de structuration :
// Exemple de MenuTree
pub struct MenuTree {
    root: MenuItem,
}
pub struct MenuItem {
    label: String,
    action: Box<dyn Fn(&str) -> Result<(), String>>,
    submenus: Vec<MenuItem>,
}
Objectifs secondaires :
- Générer un README.md avec le schéma de l'API
- Créer un test unitaire pour le MenuHandler
- Ajouter un système de logs minimal pour les erreurs
Résultat attendu :
- Code modulaire avec un arbre de menus géré via MenuTree
- Une API menu_api indépendante des commandes spécifiques
- Externalisation de la logique d'interaction utilisateur
- Possibilité d'étendre les menus sans modifier le cœur de l'application

***

1. Déplacer la logique des fichiers wizard/*.rs vers menu_api/, en utilisant:                                                                                                                                                                                                   
     - MenuTree pour la structure                                                                                                                                                                                                                                                 
     - MenuHandler pour la navigation                                                                                                                                                                                                                                             
     - ToolAction/ActionDispatcher pour les actions                                                                                                                                                                                                                               
  2. Exemple d'évolution du models_menu_adapter.rs:                                                                                                                                                                                                                               
                                                                                                                                                                                                                                                                                  
     // Actuel (wrapper vide):                                                                                                                                                                                        ```                                                         
     pub fn run(&mut self) -> Result<()> {                                                                                                                                                                                                                                        
         crate::wizard::menu_models::run(self.i18n, self.state, self.hw)                                                                                                                                                                                                          
     }                                                                                                                                                                                                                                                                            
                                                                                                                                                                                                                                                                                  
     // À faire (vraie implémentation):                                                                                                                                                                                                                                           
     pub fn run(&mut self) -> Result<()> {                                                                                                                                                                                                                                        
         let tree = build_models_menu_tree(self.i18n, self.state, self.hw)?;                                                                                                                                                                                                      
         let mut handler = MenuHandler::new(tree, build_dispatcher(self.i18n, self.state, self.hw)?);                                                                                                                                                                             
         handler.run()                                                                                                                                                                                                                                                            
     }                                                                                                                                                                                        ```                                                                          
                                                                                                                                                                                                                                                                                  
  Pour l'instant:                                                                                                                                                                                                                                                                 
                                                                                                                                                                                                                                                                                  
   - Les wizard/*.rs ne peuvent pas être supprimés                                                                                                                                                                                                                                
   - Ils contiennent le code fonctionnel                                                                                                                                                                                                                                          
   - Les adaptateurs dans menu_api/ sont prêts pour recevoir la vraie implémentation

***

# Rappel des menus actuels

```
Menu Principal
   💾 49.8/61.9 Go | 🎮 9.9/12.0 Go

> ▶ Reprendre OpenCode (qwen2.5:7b)
  🧙 Que voulez-vous faire aujourd'hui ?
  🤖 Choisir un modèle IA
  🔬 Scientific Agent Skills
  🛠️  Lancer un outil
  🧹 Nettoyage
  ⚙️  Configuration
  🌍 Changer de langue
  ❌ Quitter
```

## ▶ Reprendre OpenCode (qwen2.5:7b)
- Check: Vérifie dans le state si last_model et last_tool sont définis
- Action: Tool::launch avec le tool et le model

## Notes
- Pour chaque prochain menu et sous-menu on a "↩️  Retour" en premier menu item qui retourne au menu parent
- Pagination selon la taille du terminal

## 🧙 Que voulez-vous faire aujourd'hui ?

```
🧙 Que voulez-vous faire aujourd'hui ?
   💾 80% 49.8/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 82% 9.8/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

  ↩️  Retour
  Usage général
> Programmation
  Raisonnement
  Chat
  Multimodal
  Embedding
```

- Action: Ouvrir un sous menu avec la liste des modèles compatibles

## Programmation
- Notes:
* idem pour les autres cas d'usage du menu "🧙 Que voulez-vous faire aujourd'hui ?" mais avec un filtre sur les modèles lié au cas d'usage
* le choix d'un modèle met à jour last_model et le menu_header puis ouvre la liste des outils relatifs

```
🧙 Que voulez-vous faire aujourd'hui ? Programmation
   💾 80% 49.8/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 82% 9.8/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 phi4:latest

  ↩️  Retour
  hf.co/my-ai-stack/Stack-3.0-Omni-Nexus:Q8_0
> phi4:latest
  deepseek-coder:latest
  qwen2.5:7b
  qwen2.5-coder:14b
  qwen3:latest
  llama3.1:latest
  llama3:latest
  gemma:latest
  qwen2.5:3b
  qwen2.5:1.5b
  ministral-3:3b
```

### 🚀 Lancer avec le modèle actuel : phi4:latest
- Ouvre ce sous-menu avec la liste des outils installés
- Action: Tool::launch avec le tool et le model

```
🧙 Que voulez-vous faire aujourd'hui ? Programmation
   💾 81% 50.0/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 82% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 phi4:latest

  ↩️  Retour
> 🔧 Claude Code - Outil de codage d'Anthropic avec sous-agents
  🔧 OpenCode - Agent de codage open-source d'Anomaly
  🔧 Droid - Agent de codage de Factory (terminal + IDE)
  🔧 Codex - Agent de codage open-source d'OpenAI
  🔧 Ollama - Chat avec un modèle IA local
```

## 🤖 Choisir un modèle IA
```
🤖 Choisir un modèle IA
   💾 80% 49.6/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 83% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 phi4:latest

  ↩️  Retour
> ✅ hf.co/my-ai-stack/Stack-3.0-Omni-Nexus:Q8_0 [8b] local (installed)
  ✅ phi4:latest [3b] microsoft (installed)
  ✅ deepseek-coder:latest [7b] deepseek-ai (installed)
  ✅ qwen2.5-coder:14b [7b] NousResearch (installed)
  ✅ llama3.1:latest [8b] meta-llama (installed)
  ✅ llama3:latest [8b] meta-llama (installed)
  ✅ gemma:latest [3b] google (installed)
  ✅ qwen2.5:3b [7b] local (installed)
  ✅ qwen2.5:1.5b [7b] local (installed)
  ✅ ministral-3:3b [3b] mistralai (installed)
  ✅ qwen2.5:7b [7b] GestaltLabs (installed)
  ✅ qwen3:latest [8b] Qwen (installed)
  ─── Parcourir par organisation ───
  🏢 Qwen (16 modèle(s))
  🏢 openai (2 modèle(s))
  🏢 google (7 modèle(s))
  🏢 deepseek-ai (3 modèle(s))
  🏢 mistralai (6 modèle(s))
  🏢 moonshotai (1 modèle(s))
  🏢 nvidia (3 modèle(s))
  🏢 ggml-org (1 modèle(s))
  🏢 kai-os (2 modèle(s))
  🏢 mlx-community (3 modèle(s))
  🏢 pastapaul (1 modèle(s))
  🏢 zai-org (1 modèle(s))
  🏢 meta-llama (1 modèle(s))
  🏢 bartowski (1 modèle(s))
  🏢 TurbulenceDeterministe (1 modèle(s))
  🏢 FINAL-Bench (1 modèle(s))
```

- Notes:
* La première partie montre les modèles installés, c.f. OllamaApi::get_models()
* Action: Sur un modèle, sauve le state last_model et met à jour le menu_header
* Parcourir par organisation, utilise l'API llmfit en prenant la liste des organisation
* Action: Sur une organisation, ouvre un nouveau sous-menu listant les modèles de l'organisation (toujours API llmfit)

### 🏢 Qwen (16 modèle(s))
- Notes:
* idem pour les autres organisations
* le choix d'un modèle l'installe
* Il y a une colorisation des modèles pour montrer la compatibilité avec le hardware

```
🤖 Choisir un modèle IA. Organisation: 🏢 Qwen (16 modèle(s))
   💾 80% 49.6/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 83% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 phi4:latest

  ↩️  Retour
> 📥 Qwen2.5-14B-Instruct [14b] → qwen2.5:latest
  📥 Qwen2.5-1.5B-Instruct-GGUF [2b] → qwen2.5:latest
  📥 Qwen3-14B [14b] → qwen3:14b
  📥 Qwen2.5-7B-Instruct [7b] → qwen2.5:latest
  📥 Qwen3-8B [8b] → qwen3:8b
  📥 Qwen2.5-7B [8b] → qwen2.5:latest
  📥 Qwen3-VL-8B-Instruct [9b] → qwen3:8b
  📥 Qwen3-8B-GGUF [8b] → qwen3:8b
  📥 Qwen3-14B-Base [15b] → qwen3:14b
  📥 Qwen2.5-1.5B [2b] → qwen2.5:latest
  📥 Qwen2.5-1.5B-Instruct [2b] → qwen2.5:latest
  📥 Qwen2.5-14B [15b] → qwen2.5:latest
  📥 Qwen3-8B-Base [8b] → qwen3:8b
  📥 Qwen3-32B [32b] → qwen3:30b
  📥 Qwen2.5-32B [33b] → qwen2.5:latest
  📥 Qwen2.5-72B [73b] → qwen2.5:latest
```

## 🔬 Scientific Agent Skills

```
🔬 Scientific Agent Skills
   💾 81% 49.9/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 82% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

  ↩️  Retour
> 🧬 Bioinformatique & Génomique
  🧪 Chéminformatique & Découverte de médicaments
  🔬 Protéomique & Spectrométrie de masse
  🏥 Recherche clinique & Médecine personnalisée
  🧬 Génomique
  🤖 Machine Learning & IA
  🤖 Outils Agentiques pour la Recherche
```


### 🧬 Bioinformatique & Génomique

- Notes:
* Si des compétentences ne sont pas encore installées, propose de le faire
```
🧬 Bioinformatique & Génomique
   💾 81% 49.9/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 82% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b
Compétences disponibles
> ↩️  Retour
  📦 biopython
  📦 bioservices
  ✅ gget
  ✅ scanpy
  ✅ anndata
  ✅ cellxgene-census

Sélectionnez un outil pour lancer les skills
> ↩️  Retour
  🤖 Claude Code
  🤖 OpenCode
  🤖 Droid
  🤖 Codex
```


## 🛠️  Lancer un outil
- Notes:
* Installe ou lance un outils

```
🛠️  Lancer un outil
   💾 81% 49.9/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 83% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

  ↩️  Retour
> ✅ Ollama - Chat avec un modèle IA local
  📦 Open WebUI - Interface web pour vos modèles IA
  📦 OpenClaw - Assistant IA personnel avec 100+ skills
  🤖 Claude Code - Outil de codage d'Anthropic avec sous-agents [agentic]
  📦 Hermes Agent - Agent IA auto-améliorant de Nous Research
  🤖 OpenCode - Agent de codage open-source d'Anomaly [agentic]
  🤖 Codex - Agent de codage open-source d'OpenAI [agentic]
  🤖 Copilot CLI - Agent de codage IA de GitHub pour le terminal [agentic]
  🤖 Droid - Agent de codage de Factory (terminal + IDE) [agentic]
  🤖 Pi - Agent IA minimal avec support plugins [agentic]
  🤖 Pool - Agent de codage de Poolside (https://github.com/poolsideai/pool) [agentic]
  ✅ Obsidian - Base de connaissances locale et application de prise de notes
  🤖 Goose - Agent IA par Block pour les tâches de codage [agentic]
  📦 LLMFit - Outil d'entraînement et d'évaluation de modèles LLM
```


## 🧹 Nettoyage
- Notes:
* Ouvre un sous-menu

```
🧹 Nettoyage
   💾 81% 49.9/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 83% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

  ↩️  Retour
> 🛠️  Désinstaller des outils
  🤖 Supprimer des modèles
```

### 🛠️  Désinstaller des outils
- Action: Tool::uninstall

```
🛠️  Désinstaller des outils
   💾 81% 49.9/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 83% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

  ↩️  Retour
> 🗑️  Ollama
  🗑️  Claude Code
  🗑️  OpenCode
  🗑️  Codex
  🗑️  Droid
  🗑️  Pool
```

### 🤖 Supprimer des modèles
- Liste: OllamaApi::get_models
- Action: OllamaApi::delete_model

```
🤖  Supprimer des modèles
   💾 81% 49.9/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 83% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

  ↩️  Retour
> 🗑️ hf.co/my-ai-stack/Stack-3.0-Omni-Nexus:Q8_0 [8b] local
  🗑️ phi4:latest [3b] microsoft
  🗑️ deepseek-coder:latest [7b] deepseek-ai
  🗑️ qwen2.5-coder:14b [7b] NousResearch
  🗑️ llama3.1:latest [8b] meta-llama
  🗑️ llama3:latest [8b] meta-llama
  🗑️ gemma:latest [3b] google
  🗑️ qwen2.5:3b [7b] local
  🗑️ qwen2.5:1.5b [7b] local
  🗑️ ministral-3:3b [3b] mistralai
  🗑️ qwen2.5:7b [7b] GestaltLabs
  🗑️ qwen3:latest [8b] Qwen
```

## ⚙️  Configuration

```
⚙️  Configuration
   💾 81% 49.9/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 83% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

  ↩️  Retour
> ⚡ Performance
  🔧 Paramètres Ollama
  🔑 Fournisseurs API
  🦞 Paramètres OpenClaw
  📂 Shells
  📄 Regénérer ~/.wzllama/env
  🗑️  Désinstaller wzllama
```

- Notes pour les sous-menu:
* Si une valeur, met à jour
* Si un binaire, switch true/false

### ⚡ Performance

```
⚡ Performance
   💾 81% 49.9/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 83% 9.9/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

  ↩️  Retour
> 📐 16384 tokens
  💾 Cache KV: q8_0
  ⚡ Flash Attention: ✅
  ☁️  Cloud: ❌ Bloqué
```

### 🔧 Paramètres Ollama
```
🔧 Paramètres Ollama
   💾 80% 49.7/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 82% 9.8/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

> ↩️  Retour
  📍 Host: 127.0.0.1:11434
  🌐 Origins: http://localhost:*
  ⏱️  Keep alive: -1s
  🔀 Parallel: 4
  📦 Max loaded: 3
  💾 Max VRAM: auto MB
  🎮 CUDA: all
```



### 🔑 Fournisseurs API
```
🔑 Fournisseurs API
   💾 80% 49.7/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 82% 9.8/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

> ↩️  Retour
  🔓 OpenAI: not set
  🔓 Anthropic: not set
  🔗 OpenAI URL: http://localhost:11434/v1
  🔗 Anthropic URL: http://localhost:11434/v1
```

### 🦞 Paramètres OpenClaw
```
🦞 Paramètres OpenClaw
   💾 80% 49.7/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 82% 9.8/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

  🔑 API Key: default (ollama-local)
> ↩️  Retour
```

### 📂 Shells
```
📂 Shells
   💾 80% 49.7/61.9 Go ███░░░░░░░░░░░░░░░░░
   🎮 82% 9.8/12.0 Go ███░░░░░░░░░░░░░░░░░
   🤖 qwen2.5:7b

   ✅ bash
   ✅ zsh
   ✅ fish
Action:
  ↩️  Retour
> 📂 Installer dans tous les shells
  📂 Retirer de tous les shells
```

### 📄 Regénérer ~/.wzllama/env
- Action: Régénère directement le fichier env

### 🗑️  Désinstaller wzllama
Action: Désinstalle avec confirmation wzllama

## 🌍 Changer de langue
- Action: met à jour le state de la langue
```
🌍 Changer de langue
   💾 49.8/61.9 Go | 🎮 9.8/12.0 Go

> ↩️  Retour
  English (en)
  Français (fr)
```


***

# Mission : Créer une documentation complète en Markdown pour recréer un programme wzllama

## Contexte
Je possède l'intégralité du code source d'un programme wzllama. Je souhaite le réécrire dans un autre langage de programmation. Tu dois analyser mon code et générer une série de fichiers Markdown **ultra-détaillés** qui me permettront de reconstruire ce programme **fonctionnalité par fonctionnalité**, sans avoir besoin de relire le code source original.

## Ce que je veux obtenir

Génère **plusieurs fichiers Markdown** structurés comme suit :

### 1. `01-architecture-globale.md`
- Architecture générale du programme (diagramme textuel ou description claire)
- Structure des dossiers et fichiers
- Relations entre les composants/modules
- Patrons de conception utilisés (design patterns)
- Flux de données principal (data flow)

### 2. `02-fonctionnalités-completes.md`
Pour **CHAQUE fonctionnalité** du programme :
- Nom de la fonctionnalité
- Description détaillée de ce qu'elle fait
- Entrées (inputs) : types, formats, contraintes
- Sorties (outputs) : types, formats, comportements
- Logique métier complète (algorithme détaillé)
- Cas limites gérés (edge cases)
- Exemples concrets d'utilisation

### 3. `03-techniques-implémentation.md`
Pour **CHAQUE technique/technologie** utilisée :
- Nom de la technique/bibliothèque/framework
- Version utilisée (si identifiable)
- Pourquoi elle est utilisée
- Comment elle est implémentée dans le code
- Configuration nécessaire
- Dépendances et prérequis
- Alternatives possibles pour le nouveau langage

### 4. `04-structures-données.md`
- Toutes les structures de données (classes, structs, types, interfaces)
- Champs/attributs avec types et descriptions
- Relations entre structures (héritage, composition, agrégation)
- Constantes et valeurs fixes
- Énumerations (enums)

### 5. `05-api-et-interfaces.md` (si applicable)
- Toutes les fonctions/publiques méthodes avec :
  - Signature complète
  - Paramètres (nom, type, description, valeur par défaut, obligatoire/optionnel)
  - Valeur de retour (type, description)
  - Exceptions levées
  - Exemples d'appel
- API externes consommées (endpoints, authentification, formats de données)

### 6. `06-logique-métier-détaillée.md`
- Algorithmes complexes expliqués étape par étape
- Règles de métier (business rules)
- Formules de calcul (avec équations si nécessaire)
- Workflows et processus métier
- Estados machines (si applicable)

### 7. `07-gestion-erreurs-journalisation.md`
- Types d'erreurs gérés
- Codes d'erreur et messages
- Stratégie de gestion d'exceptions
- Système de logging (niveaux, formats, destinations)
- Messages d'erreur utilisateur

### 8. `08-concurrency-performance.md` (si applicable)
- Gestion du multithreading/multiprocessing
- Synchronisation (locks, mutex, sémaphores)
- Optimisations de performance implémentées
- Goulots d'étranglement connus
- Stratégies de caching

### 9. `09-persistancedonnées.md` (si applicable)
- Type de stockage (base de données, fichiers, cache)
- Schéma de base de données (tables, colonnes, types, relations)
- Requêtes SQL/NoSQL utilisées
- Formats de fichiers (JSON, XML, binaire, etc.)
- Sérialisation/désérialisation

### 10. `10-configuration-environnement.md`
- Variables d'environnement nécessaires
- Fichiers de configuration (formats, paramètres)
- Prérequis système (OS, bibliothèques système)
- Ports réseau utilisés
- Secrets et gestion des credentials

### 11. `11-tests-qualité.md`
- Types de tests implémentés (unitaires, intégration, E2E)
- Couverture de code
- Données de test
- Critères de qualité

### 12. `12-guide-migration-langage-nouveau.md`
- Tableau de correspondance : ancien langage → nouveau langage
  - Types de données
  - Fonctions/bibliothèques équivalentes
  - Syntaxe critique
  - Particularités à retenir
- Structure recommandée pour le nouveau projet
- Checklist de migration étape par étape
- Pièges courants à éviter lors de la réécriture

## Exigences de qualité

Pour chaque fichier Markdown :

✅ **Exhaustif** : rien d'important ne doit être omis  
✅ **Détaillé** : chaque concept doit être expliqué suffisamment pour être réimplémenté sans le code original  
✅ **Clair** : langage simple, éviter le jargon non expliqué  
✅ **Structuré** : utiliser titres, sous-titres, listes, tableaux, blocs de code  
✅ **Avec exemples** : inclure des exemples de code pseudo ou du nouveau langage cible quand c'est pertinent  
✅ **Avec références** : citer quelles parties du code source original correspondent à chaque section  

## Format de sortie

- Génère chaque section comme un **fichier Markdown séparé**
- Utilise la syntaxe Markdown complète :
  - `#` pour les titres hiérarchisés
  - Tableaux pour les comparaisons
  - Blocs de code avec spécification du langage
  - Listes numérotées et à puces
  - Gras/italique pour mettre en évidence
  - Liens internes entre fichiers (si pertinent)
- Ajoute un `README.md` principal avec la table des matières et les liens vers tous les fichiers

## Instructions d'analyse du code

1. Analyse **tous les fichiers** de code source que je vais te fournir
2. Identifie **toutes les fonctionnalités** même celles qui semblent mineures
3. Repère **toutes les dépendances** internes et externes
4. Documente **tous les comportements** y compris les comportements implicites
5. Note **toutes les hypothèses** faites par le code original