# Flottes OpenClaw

## Vue d'ensemble

Les flottes OpenClaw sont des groupes d'agents IA spécialisés qui travaillent ensemble via l'orchestrateur OpenClaw.

## Concepts clés

### FleetConfig

```rust
pub struct FleetConfig {
    pub orchestrator: OrchestratorConfig,
    pub reflexion_agents: Vec<AgentTemplate>,
    pub expert_agents: Vec<AgentTemplate>,
}

pub struct OrchestratorConfig {
    pub model: String,
    pub num_ctx: u32,
    pub system_prompt: String,
}

pub struct AgentTemplate {
    pub name: String,
    pub role: String,
    pub model: String,
    pub num_ctx: u32,
    pub temperature: f32,
    pub system_prompt: String,
    pub enabled: bool,
}
```

## Templates de flottes (fleet_templates.rs)

### Code fleet

```rust
fleet_templates::get("code", wizard_model, i18n) -> FleetConfig {
    FleetConfig {
        orchestrator: OrchestratorConfig {
            model: "qwen2.5:7b".into(),
            num_ctx: 32768,
            system_prompt: "Tu es un architecte logiciel en chef.",
        },
        reflexion_agents: [
            AgentTemplate { name: "wzllama-reflexion-arch", role: "Architecte logiciel", ... },
            AgentTemplate { name: "wzllama-reflexion-review", role: "Réviseur de code", ... },
        ],
        expert_agents: [
            AgentTemplate { name: "wzllama-expert-lint", role: "Linter", ... },
            AgentTemplate { name: "wzllama-expert-doc", role: "Documentaliste", ... },
            AgentTemplate { name: "wzllama-expert-test", role: "Testeur", ... },
        ],
    }
}
```

### Generic fleet

```rust
fleet_templates::get("generic", wizard_model, i18n) -> FleetConfig {
    FleetConfig {
        orchestrator: OrchestratorConfig {
            model: "qwen2.5:7b".into(),
            num_ctx: 32768,
            system_prompt: "Tu es un coordinateur de projet.",
        },
        reflexion_agents: [
            AgentTemplate { name: "wzllama-reflexion", role: "Analyste", ... },
        ],
        expert_agents: [
            AgentTemplate { name: "wzllama-expert-fast", role: "Assistant rapide", ... },
        ],
    }
}
```

## Création de flotte (fleet_creator.rs)

### Étapes de création

1. **Nom du projet** - Validation (sans espaces, chars alphanumériques)
2. **Orchestrateur** - Configuration du modèle orchestrateur
3. **Agents réflexion** - Ajout/modification des agents réflexion
4. **Experts** - Ajout/modification des agents experts
5. **Agents personnalisés** - Optionnel, via prompt interactif
6. **Validation** - Génération des fichiers de configuration

### Flux interactif

```rust
pub fn run(
    i18n: &I18n,
    state: &mut WzllamaState,
    hw: &HardwareInfo,
    model: &OllamaModel,
    model_name: &str,
    usage_type: &str
) -> Result<String>  // returns project_name
```

## Menu Flottes (menu_fleets.rs)

### Structure

```
🚀 Flottes OpenClaw
├── Créer une nouvelle flotte
├── Lancer une flotte existante
│   ├── mon-projet-1
│   ├── mon-projet-2
│   └── ...
└── Retour
```

### Détection des flottes

```rust
pub fn detect_openclaw_fleets() -> HashMap<String, PathBuf> {
    // Cherche dans ~/.wzllama/fleets/
    // Retourne nom -> chemin
}
```

## Lancement de flotte

### Processus

1. **Sélection du projet**
2. **Vérification gateway OpenClaw**
3. **Installation si nécessaire**
4. **Configuration orchestrateur**
5. **Activation des agents**
6. **Lancement via `ollama launch openclaw`**

### Commande générée

```bash
ollama launch openclaw --project mon-projet
```

## Édition d'agent (fleet_templates.rs)

### Menu d'édition

```rust
pub fn edit_agent(
    i18n: &I18n,
    agent: &mut AgentTemplate,
    index: usize,
    agent_type: &str
) -> Result<()> {
    let items = [
        "Activer/Désactiver",
        "Modifier le rôle",
        "Modifier le prompt",
        "Garder tel quel",
    ];
    
    // Navigation avec Select
    // Modification via Input
}
```

## Nettoyage des flottes (cleanup_fleets.rs)

### Menu de suppression

```
🧹 Nettoyage > 📂 Supprimer des flottes
├── 🗑️ ma-flotte-1
├── 🗑️ ma-flotte-2
├── 🗑️ Supprimer toutes les flottes
└── ↩ Retour
```

## Configuration OpenClaw

### Fichiers générés

```
~/.wzllama/fleets/{project_name}/
├── orchestrator.yaml
├── agents/
│   ├── reflexion-arch.yaml
│   ├── reflexion-review.yaml
│   └── experts/
│       ├── lint.yaml
│       ├── doc.yaml
│       └── test.yaml
└── fleet.yaml
```

### Exemple orchestrator.yaml

```yaml
model: qwen2.5:7b
num_ctx: 32768
system_prompt: |
  Tu es un architecte logiciel en chef.
  Coordonne les agents experts pour accomplir les tâches.
```

### Exemple agent.yaml

```yaml
name: wzllama-reflexion-arch
role: Architecte logiciel
model: wizard-model
num_ctx: 8192
temperature: 0.3
system_prompt: |
  Analyse la structure du code et propose des améliorations.
enabled: true
```