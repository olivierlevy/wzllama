pub mod tool_trait;
pub mod docker;
pub mod open_webui;
pub mod ollama;
pub mod openclaw;
pub mod claude_code;
pub mod hermes;
pub mod opencode;
pub mod codex;
pub mod copilot_cli;
pub mod droid;
pub mod pi;
pub mod pool;
pub mod obsidian;
pub mod goose;

use crate::config::{I18n, WzllamaState};
use tool_trait::Tool;

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
        Box::new(obsidian::ObsidianTool),
        Box::new(goose::GooseTool),
    ]
}

pub fn get_tool(id: &str) -> Option<Box<dyn Tool>> {
    get_all_tools().into_iter().find(|t| t.id() == id)
}

pub fn get_available_tools(state: &WzllamaState, i18n: &I18n) -> Vec<ToolInfo> {
    get_all_tools().iter().map(|t| {
        let installed = match t.id() {
            "ollama" => state.installed.ollama,
            "open_webui" => state.installed.open_webui,
            "openclaw" => state.installed.openclaw,
            "claude_code" => state.installed.claude_code,
            "hermes_agent" => state.installed.hermes_agent,
            "opencode" => state.installed.opencode,
            "codex" => state.installed.codex,
            "copilot_cli" => state.installed.copilot_cli,
            "droid" => state.installed.droid,
            "pi" => state.installed.pi,
            "pool" => state.installed.pool,
            "obsidian" => state.installed.obsidian,
            "goose" => state.installed.goose,
            _ => false,
        };
        ToolInfo {
            id: t.id().to_string(),
            name: t.name().to_string(),
            description: t.description(i18n),
            installed,
            supports_fleets: t.supports_fleets(),
        }
    }).collect()
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub installed: bool,
    #[allow(dead_code)]
    pub supports_fleets: bool,
}

/// Reruns la commande d'installation pour un outil (via son ID)
pub fn get_install_command(tool_id: &str) -> Option<String> {
    match tool_id {
        "ollama" => Some("curl -fsSL https://ollama.com/install.sh | sh".to_string()),
        // Open WebUI nécessite Docker - on utilise un wrapper qui vérifie Docker d'abord
        "open_webui" => Some("wzllama install-webui".to_string()),
        "openclaw" => Some("ollama install openclaw".to_string()),
        "claude_code" => Some("npm install -g @anthropic-ai/claude-code".to_string()),
        "opencode" => Some("npm install -g @opencode-ai/cli".to_string()),
        "hermes_agent" => Some("npm install -g @hermes-hq/bot".to_string()),
        "codex" => Some("ollama install codex".to_string()),
        "droid" => Some("ollama install droid".to_string()),
        "pi" => Some("ollama install pi".to_string()),
        "obsidian" => Some("flatpak install flathub md.obsidian.Obsidian -y".to_string()),
        "goose" => Some("curl -fsSL https://github.com/aaif-goose/goose/releases/download/stable/download_cli.sh | bash".to_string()),
        _ => None,
    }
}

/// Reruns la commande de lancement pour un outil (via son ID) avec modèle optionnel
pub fn get_launch_command(tool_id: &str, model: Option<&str>) -> Option<String> {
    match tool_id {
        "openclaw" => Some(format!("ollama launch openclaw{}", model.map(|m| format!(" --model {}", m)).unwrap_or_default())),
        // Open WebUI nécessite Docker - use un wrapper
        "open_webui" => Some("wzllama launch-webui".to_string()),
        "claude_code" => Some("claude".to_string()),
        "opencode" => Some("opencode".to_string()),
        "ollama" => Some(format!("ollama run {}", model.unwrap_or("llama3"))),
        "hermes_agent" => Some("ollama launch hermes".to_string()),
        "codex" => Some("codex".to_string()),
        "droid" => Some("ollama launch droid".to_string()),
        "pi" => Some("ollama launch pi".to_string()),
        "pool" => Some("pool".to_string()),
        "copilot_cli" => Some("copilot".to_string()),
        "obsidian" => Some("obsidian".to_string()),
        "goose" => Some("goose".to_string()),
        _ => Some(tool_id.to_string()),
    }
}