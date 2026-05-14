# Outils intégrés

wzllama gère 10 outils IA différents via un système de plugins basé sur le trait `Tool`.

## Architecture des outils

### Trait Tool (src/tools/tool_trait.rs)

```rust
pub trait Tool {
    fn id(&self) -> &str;              // Identifiant unique
    fn name(&self) -> &str;           // Nom affiché
    fn description(&self, i18n: &I18n) -> String;
    fn status(&self) -> ToolStatus;
    fn install(&self) -> Result<()>;
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()>;
    fn supports_fleets(&self) -> bool { false }
    fn requires_docker(&self) -> bool { false }
}
```

### Registry des outils (src/tools/mod.rs)

```rust
pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(ollama::OllamaTool),
        Box::new(open_webui::OpenWebUITool),
        Box::new(openclaw::OpenClawTool),
        Box::new(claude_code::ClaudeCodeTool),
        Box::new(hermes::HermesTool),
        Box::new(opencode::OpenCodeTool),
        Box::new(codex::CodexTool),
        Box::new(copilot_cli::CopilotCliTool),
        Box::new(droid::DroidTool),
        Box::new(pi::PiTool),
        Box::new(pool::PoolTool),
    ]
}
```

## Liste des outils

### 1. Ollama

| Propriété | Valeur |
|-----------|--------|
| ID | `ollama` |
| requires_docker | Non |
| supports_fleets | Non |

**Installation:** `curl -fsSL https://ollama.com/install.sh \| sh`

**Description:** Serveur de modèles IA locaux avec API REST compatible OpenAI.

### 2. Open WebUI

| Propriété | Valeur |
|-----------|--------|
| ID | `open_webui` |
| requires_docker | Oui |
| supports_fleets | Non |

**Installation:** `wzllama install-webui`

**Docker command:**
```bash
docker run -d -p 3000:8080 \
  --add-host=host.docker.internal:host-gateway \
  -v open-webui:/app/backend/data \
  --name open-webui \
  --restart always \
  ghcr.io/open-webui/open-webui:main
```

### 3. OpenClaw

| Propriété | Valeur |
|-----------|--------|
| ID | `openclaw` |
| requires_docker | Non |
| supports_fleets | Oui |

**Installation:** `ollama install openclaw`

**Caractéristiques:**
- Assistant IA personnel avec 100+ skills
- Support des flottes d'agents
- Orchestration multi-agents

### 4. Claude Code

| Propriété | Valeur |
|-----------|--------|
| ID | `claude_code` |
| requires_docker | Non |
| supports_fleets | Non |

**Installation:** `npm install -g @anthropic-ai/claude-code`

**Caractéristiques:**
- Agent de codage d'Anthropic
- Support des sous-agents
- Intégration IDE

### 5. OpenCode

| Propriété | Valeur |
|-----------|--------|
| ID | `opencode` |
| requires_docker | Non |
| supports_fleets | Non |

**Installation:** `npm install -g @opencode-ai/cli`

### 6. Codex (OpenAI)

| Propriété | Valeur |
|-----------|--------|
| ID | `codex` |
| requires_docker | Non |
| supports_fleets | Non |

**Installation:** `ollama install codex`

### 7. Copilot CLI

| Propriété | Valeur |
|-----------|--------|
| ID | `copilot_cli` |
| requires_docker | Non |
| supports_fleets | Non |

### 8. Droid

| Propriété | Valeur |
|-----------|--------|
| ID | `droid` |
| requires_docker | Non |
| supports_fleets | Non |

**Caractéristiques:**
- Agent de codage de Factory
- Terminal + IDE integration

### 9. Hermes Agent

| Propriété | Valeur |
|-----------|--------|
| ID | `hermes_agent` |
| requires_docker | Non |
| supports_fleets | Non |

**Installation:** `npm install -g @hermes-hq/bot`

**Caractéristiques:**
- Agent IA auto-améliorant de Nous Research
- Nécessite configuration au premier lancement (`hermes setup`)

### 10. Pi

| Propriété | Valeur |
|-----------|--------|
| ID | `pi` |
| requires_docker | Non |
| supports_fleets | Non |

**Caractéristiques:**
- Agent IA minimal
- Support plugins

### 11. Pool

| Propriété | Valeur |
|-----------|--------|
| ID | `pool` |
| requires_docker | Non |
| supports_fleets | Non |

**Description:** Agent de codage de Poolside.

## Gestion Docker

### Détection Docker (src/tools/docker.rs)

```rust
pub fn ensure_ready_no_confirm() -> Result<()> {
    // Vérifie si Docker daemon est actif
    // Retourne Ok si prêt, Err sinon
}
```

### Check Docker (utilisé par Open WebUI)

```rust
pub fn check_docker_running() -> bool {
    shell::run("docker ps").is_ok()
}

pub fn check_docker_installed() -> bool {
    shell::run("which docker").is_ok()
}
```

## Commandes par outil

### Installation

```rust
// src/tools/mod.rs
pub fn get_install_command(tool_id: &str) -> Option<String> {
    match tool_id {
        "ollama" => Some("curl -fsSL https://ollama.com/install.sh | sh".into()),
        "open_webui" => Some("wzllama install-webui".into()),
        "openclaw" => Some("ollama install openclaw".into()),
        "claude_code" => Some("npm install -g @anthropic-ai/claude-code".into()),
        // ...
        _ => None,
    }
}
```

### Lancement

```rust
pub fn get_launch_command(tool_id: &str, model: Option<&str>) -> Option<String> {
    match tool_id {
        "openclaw" => Some(format!("ollama launch openclaw{}", 
            model.map(|m| format!(" --model {}", m)).unwrap_or_default())),
        "open_webui" => Some("wzllama launch-webui".into()),
        "ollama" => Some(format!("ollama run {}", model.unwrap_or("llama3"))),
        // ...
        _ => Some(tool_id.to_string()),
    }
}
```

## État d'installation

### Suivi dans WzllamaState

```rust
pub struct InstalledTools {
    pub ollama: bool,
    pub open_webui: bool,
    pub openclaw: bool,
    pub claude_code: bool,
    pub opencode: bool,
    pub codex: bool,
    pub copilot_cli: bool,
    pub droid: bool,
    pub hermes_agent: bool,
    pub pi: bool,
    pub pool: bool,
}
```

### Auto-détection

```rust
// Détecte si un outil est installé
pub fn get_available_tools(state: &WzllamaState, i18n: &I18n) -> Vec<ToolInfo> {
    get_all_tools().iter().map(|t| {
        let installed = match t.id() {
            "ollama" => state.installed.ollama,
            // ... map ID -> state field
            _ => false,
        };
        ToolInfo { id, name, description, installed, supports_fleets }
    }).collect()
}
```