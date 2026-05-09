use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct PiTool;

impl Tool for PiTool {
    fn id(&self) -> &str { "pi" }
    fn name(&self) -> &str { "Pi" }
    fn description(&self) -> &str { "Agent IA minimal avec support plugins" }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("pi-agent") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "npm install -g pi-agent".into() } }
    }

    fn install(&self) -> Result<()> { shell::run("npm install -g pi-agent")?; Ok(()) }

    // Le binaire s'appelle "pi"
    fn launch(&self, _state: &WzllamaState, model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        match model {
            Some(m) => println!("pi --model ollama/{}", m),
            None => println!("pi"),
        }
        Ok(())
    }
}