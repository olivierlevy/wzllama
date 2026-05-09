use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct ClaudeCodeTool;

impl Tool for ClaudeCodeTool {
    fn id(&self) -> &str { "claude_code" }
    fn name(&self) -> &str { "Claude Code" }
    fn description(&self) -> &str { "Outil de codage d'Anthropic avec sous-agents" }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("claude-code") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "npm install -g @anthropic-ai/claude-code".into() } }
    }

    fn install(&self) -> Result<()> { shell::run("npm install -g @anthropic-ai/claude-code")?; Ok(()) }

    fn launch(&self, _state: &WzllamaState, model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("export ANTHROPIC_BASE_URL=http://localhost:11434/v1");
        println!("export ANTHROPIC_API_KEY=ollama");
        match model {
            Some(m) => println!("claude-code --model {}", m),
            None => println!("claude-code"),
        }
        Ok(())
    }
}