use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenClawTool;

impl Tool for OpenClawTool {
    fn id(&self) -> &str { "openclaw" }
    fn name(&self) -> &str { "OpenClaw" }
    fn description(&self) -> &str { "Assistant IA personnel avec 100+ skills" }
    fn supports_fleets(&self) -> bool { true }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("openclaw") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "npm install -g openclaw".into() } }
    }

    fn install(&self) -> Result<()> {
        shell::run("npm install -g openclaw")?;
        Ok(())
    }

    fn launch(&self, _state: &WzllamaState, _model: Option<&str>, fleet: Option<&str>) -> Result<()> {
        match fleet {
            Some(f) => { shell::run(&format!("openclaw --profile {}", f))?; }
            None => { shell::run("openclaw")?; }
        }
        Ok(())
    }
}