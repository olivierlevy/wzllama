use anyhow::Result;
use crate::config::{I18n, WzllamaState};

pub enum ToolStatus {
    Installed,
    NotInstalled { install_cmd: String },
    Running,
}

pub trait Tool {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self, i18n: &I18n) -> String;
    fn status(&self) -> ToolStatus;
    fn install(&self) -> Result<()>;
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>, fleet: Option<&str>) -> Result<()>;
    fn supports_fleets(&self) -> bool { false }
}