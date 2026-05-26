# wzllama Documentation

**Version:** 0.3.0  
**Type:** Wizard CLI pour stack IA locale  
**Langue:** Français/English (i18n support)

---

## Table des Matières

1. [Vue d'ensemble](./docs/overview.md) - Présentation du projet
2. [Architecture](./docs/architecture.md) - Diagrammes et flux de données
3. [Guide de démarrage](./docs/getting-started.md) - Installation et premiers pas
4. [Mode CLI (Wizard)](./docs/cli-wizard.md) - Interface ligne de commande
6. [Outils intégrés](./docs/tools.md) - Ollama, OpenClaw, Claude Code, etc.
7. [Configuration](./docs/configuration.md) - Fichiers et variables
8. [Gestion des modèles](./docs/models.md) - Modèles IA et ranking
9. [Flottes OpenClaw](./docs/fleets.md) - Orchestration multi-agents
10. [API et développement](./docs/api-development.md) - Architecture technique
11. [Internationalisation](./docs/i18n.md) - Système de traduction
12. [Structure des fichiers](./docs/file-structure.md) - Arborescence du projet

---

## Résumé rapide

wzllama est un wizard CLI qui simplifie l'installation et l'utilisation d'une stack IA locale complète incluant:

- **Ollama** - Serveur de modèles IA locaux
- **Open WebUI** - Interface web pour les modèles
- **OpenClaw** - Assistant IA avec 100+ skills
- **Claude Code** - Agent de codage d'Anthropic
- **OpenCode, Codex, Droid, Pi** - Autres agents de codage
- **Hermes Agent** - Agent IA auto-améliorant

---

## Commandes principales

```bash
# Lancer le wizard (mode CLI par défaut)
wzllama

# Autres commandes
wzllama wizard     # Alias du wizard
wzllama validate   # Valider les templates
wzllama bench      # Benchmark
wzllama reset-templates  # Réinitialiser
wzllama check-i18n # Vérifier l'i18n
wzllama uninstall  # Désinstaller wzllama
wzllama install-webui  # Installer Open WebUI
wzllama launch-webui  # Lancer Open WebUI
```

---

## Navigation dans le wizard

- **↑/↓** : Naviguer dans les menus
- **Enter** : Valider la sélection
- **Escape** : Retour en arrière (ou quitter si menu principal)
- **Ctrl-C** : Interrompre et quitter

---

## Installation rapide

```bash
# Depuis les sources
git clone https://github.com/yourusername/wzllama.git
cd wzllama
cargo build --release

# Ou avec cargo
cargo install --path .
```

Première exécution:
1. Sélectionnez votre langue
2. wzllama détecte votre hardware
3. Propose l'installation de Ollama si nécessaire
4. Recommande des modèles selon votre machine