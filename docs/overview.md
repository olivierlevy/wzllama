# Vue d'ensemble

## Qu'est-ce que wzllama ?

wzllama est un **wizard CLI** (interface ligne de commande interactive) conçu pour simplifier l'installation, la configuration et l'utilisation d'une stack complète d'outils d'Intelligence Artificielle locale.

## Problème résolu

Utiliser une stack IA locale comme Ollama implique souvent:
- L'installation manuelle d'outils (Ollama, Open WebUI, OpenClaw, etc.)
- La configuration complexe des variables d'environnement
- Le choix des modèles adaptés à votre hardware
- La gestion des mises à jour et versions
- La coordination entre plusieurs agents IA

wzllama automatise tout cela via un wizard interactif.

## Fonctionnalités principales

### 🔧 Installation automatisée

| Outil | Description | Installation |
|-------|-------------|--------------|
| **Ollama** | Serveur de modèles IA | Auto-install ou vérification |
| **Open WebUI** | Interface web | Docker géré |
| **OpenClaw** | Assistant IA 100+ skills | Ollama integration |
| **Claude Code** | Agent de codage Anthropic | NPM |
| **OpenCode** | Agent open-source | NPM |
| **Codex** | Agent OpenAI | Ollama |
| **Droid** | Agent Factory | Ollama |
| **Hermes** | Agent auto-améliorant | NPM |

### 🎯 Sélection intelligente des modèles

- **Détection hardware**: CPU, RAM, GPU automatiquement détectés
- **Ranking adaptatif**: Modèles classés selon votre machine
- **Usage ciblé**: Code, texte long, agents, chat
- **Estimation ressources**: Tokens et temps calculés

### ⚙️ Configuration centralisée

- Fichier `config.yaml` unique pour tous les paramètres
- Génération automatique du fichier `env` avec toutes les variables
- Sauvegarde de l'état (dernier modèle, outils installés)

### 🚀 Orchestration multi-agents

- Création de "flottes" d'agents OpenClaw
- Orchestrateur + agents réflexion + experts
- Configuration personnalisable de chaque agent

### 🌍 Internationalisation

- Support FR/EN (extensible)
- Plus de 300 clés de traduction
- Détection automatique de la langue système

## Deux modes d'interface

### Mode CLI (Wizard) - Par défaut

```bash
wzllama
```

- Interface en ligne de commande avec `dialoguer`
- Menus navigables ↑↓ et Enter
- Escape pour retourner en arrière
- Compatible tout terminal (min 40x10)

### Mode TUI - Optionnel

```bash
wzllama --tui
```

- Interface terminal riche avec `ratatui`
- Widgets temps réel (barres de ressources)
- Navigation fluide entre écrans
- Nécessite un terminal plus grand (recommendé 80x25)

## Architecture technique

```
┌─────────────────────────────────────────────┐
│                  CLI                        │
└─────────────────────────────────────────────┘
                        │
        ┌───────────────┴───────────────┐
        ▼                               ▼
┌───────────────┐              ┌───────────────┐
│   Wizard      │              │     TUI       │
│   (dialoguer) │              │   (ratatui)   │
└───────────────┘              └───────────────┘
        │                               │
        └───────────────┬───────────────┘
                        ▼
┌─────────────────────────────────────────────┐
│              Business Logic                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │  config  │ │   core   │ │  tools   │   │
│  │  i18n    │ │ ollama   │ │ Tool trait│   │
│  │  state   │ │ hardware │ │ impls    │   │
│  └──────────┘ └──────────┘ └──────────┘   │
└─────────────────────────────────────────────┘
```

## Use cases typiques

### Développeur solo

1. Installation: `wzllama` → tout est installé
2. Choix: "Outils" → "Claude Code" → coding assistant
3. Itération: "Modèles" → changer de modèle selon besoin

### Équipe projet

1. Installation stack complète
2. Configuration modèles équipe (dans config.yaml)
3. Partage du fichier `env` pour cohérence
4. Utilisation d'OpenClaw fleet pour tâches complexes

### Ressources limitées

1. Hardware détecté (ex: 8GB RAM)
2. wzllama propose des modèles adaptés (3-7B)
3. Pas de surcharge mémoire

## Stack technique

- **Langage**: Rust (edition 2021)
- **CLI parsing**: clap 4.5
- **TUI**: ratatui 0.26 + crossterm 0.27
- **CLI interaction**: dialoguer 0.12
- **Serialization**: serde/serde_yaml/serde_json
- **i18n**: JSON avec HashMap

## Licence

À définir - MIT ou Apache-2.0 recommandé