use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenCodeTool;

impl Tool for OpenCodeTool {
    fn id(&self) -> &str { "opencode" }
    fn name(&self) -> &str { "OpenCode" }
    fn description(&self) -> &str { "Agent de codage open-source d'Anomaly" }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("opencode") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "npm install -g @opencode-ai/cli".into() } }
    }

    fn install(&self) -> Result<()> { shell::run("npm install -g @opencode-ai/cli")?; Ok(()) }

    fn launch(&self, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("opencode");
        Ok(())
    }
}