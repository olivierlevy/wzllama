use anyhow::Result;
use crate::config::WzllamaState;

pub enum ToolStatus {
    Installed,
    NotInstalled { install_cmd: String },
    Running,
}

pub trait Tool {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn status(&self) -> ToolStatus;
    fn install(&self) -> Result<()>;
    fn launch(&self, state: &WzllamaState, model: Option<&str>, fleet: Option<&str>) -> Result<()>;
    fn supports_fleets(&self) -> bool { false }
}