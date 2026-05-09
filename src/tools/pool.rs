use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct PoolTool;

impl Tool for PoolTool {
    fn id(&self) -> &str { "pool" }
    fn name(&self) -> &str { "Pool" }
    fn description(&self) -> &str { "Agent de codage de Poolside (https://github.com/poolsideai/pool)" }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("pool") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "Voir https://github.com/poolsideai/pool".into() } }
    }

    fn install(&self) -> Result<()> {
        println!("ℹ️  Installation manuelle : https://github.com/poolsideai/pool");
        Ok(())
    }

    fn launch(&self, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("pool");
        Ok(())
    }
}