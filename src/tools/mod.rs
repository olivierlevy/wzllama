pub mod tool_trait;
pub mod catalog;
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
pub mod flatpak; // Utility tool, not exposed in menus
pub mod llmfit;

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
        Box::new(llmfit::LLMFitTool),
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
            "llmfit" => state.installed.llmfit,
            _ => false,
        };
        ToolInfo {
            id: t.id().to_string(),
            name: t.name().to_string(),
            description: t.description(i18n),
            installed,
        }
    }).collect()
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub installed: bool,
}
