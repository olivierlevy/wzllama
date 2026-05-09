use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct DroidTool;

impl Tool for DroidTool {
    fn id(&self) -> &str { "droid" }
    fn name(&self) -> &str { "Droid" }
    fn description(&self) -> &str { "Agent de codage de Factory (terminal + IDE)" }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("droid") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "npm install -g @factoryai/droid".into() } }
    }

    fn install(&self) -> Result<()> { shell::run("npm install -g @factoryai/droid")?; Ok(()) }

    fn launch(&self, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("droid");
        Ok(())
    }
}