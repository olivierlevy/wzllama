# Documentation de Migration - wzllama

Ce répertoire contient la documentation complète pour reconstruire **wzllama** dans un autre langage de programmation.

## Sommaire

| Fichier | Description |
|---------|-------------|
| [01-architecture-globale.md](01-architecture-globale.md) | Architecture, modules, design patterns, data flow |
| [02-fonctionnalites-completes.md](02-fonctionnalites-completes.md) | Toutes les fonctionnalités avec algorithmes détaillés |
| [03-techniques-implémentation.md](03-techniques-implémentation.md) | Technologies, bibliothèques, versions, implémentations |
| [04-structures-données.md](04-structures-données.md) | Structures de données, champs, relations |
| [05-api-et-interfaces.md](05-api-et-interfaces.md) | API publiques, signatures, endpoints REST |
| [06-logique-métier-détaillée.md](06-logique-métier-détaillée.md) | Algorithmes complexes, business rules |
| [07-gestion-erreurs-journalisation.md](07-gestion-erreurs-journalisation.md) | Gestion des erreurs, logging, messages |
| [08-concurrency-performance.md](08-concurrency-performance.md) | Concurrence, async, performances |
| [09-persistancedonnées.md](09-persistancedonnées.md) | Fichiers, état, sérialisation |
| [10-configuration-environnement.md](10-configuration-environnement.md) | Variables d'env, fichiers config, prérequis |
| [11-tests-qualité.md](11-tests-qualité.md) | Tests existants, couverture, critères qualité |
| [12-guide-migration-langage-nouveau.md](12-guide-migration-langage-nouveau.md) | Guide pas à pas pour réécriture |

## Vue d'Ensemble Rapide

**wzllama** est un CLI d'installation et d'utilisation d'une stack IA locale:

```
wzllama
├── CLI (clap)              → 8 commandes principales
├── Menu interactif (dialoguer) → Navigation hiérarchique
├── API REST (axum)           → Port 1133, interface web embarquée
├── 14 outils IA            → Installation/lancement
└── Ollama intégration      → Modèles LLM locaux
```

## Commandes CLI

```bash
wzllama                    # Menu wizard interactif
wzllama serve              # API serveur (port 1133)
wzllama validate           # Valider templates
wzllama bench              # Benchmark modèles
wzllama reset-templates    # Réinitialiser config
wzllama check-i18n         # Vérifier traductions
wzllama uninstall          # Désinstaller
wzllama install-webui      # Installer Open WebUI (Docker)
wzllama launch-webui       # Lancer Open WebUI
```

## Endpoints API

```
GET  /api/v1/menu         → Structure menu JSON
GET  /api/v1/tools        → Liste outils
GET  /api/v1/models       → Liste modèles
GET  /api/v1/hardware     → Infos système
POST /api/v1/tools/{id}/install  → Installer outil
POST /api/v1/tools/{id}/launch   → Lancer outil
```

## Stack Technologique

| Catégorie | Technologie | Version |
|-----------|-------------|---------|
| Langage | Rust | 2021 Edition |
| CLI | clap | 4.5 |
| TUI | dialoguer | 0.12 |
| HTTP | axum | 0.8 |
| Async | tokio | 1.0 |
| JSON | serde_json | 1.0 |
| YAML | serde_yaml | 0.9 |
| TOML | toml | 0.8 |
| System | sysinfo | 0.39 |
| LLM API | llmfit-core (git) | - |