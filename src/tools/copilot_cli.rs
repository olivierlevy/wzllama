use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct CopilotCliTool;

impl Tool for CopilotCliTool {
    fn id(&self) -> &str { "copilot_cli" }
    fn name(&self) -> &str { "Copilot CLI" }
    fn description(&self) -> &str { "Agent de codage IA de GitHub pour le terminal" }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("gh-copilot") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "gh extension install github/gh-copilot".into() } }
    }

    fn install(&self) -> Result<()> { shell::run("gh extension install github/gh-copilot")?; Ok(()) }

    fn launch(&self, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("gh copilot");
        Ok(())
    }
}