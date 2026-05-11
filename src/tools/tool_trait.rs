use anyhow::Result;
use crate::config::{I18n, WzllamaState};

#[derive(Debug, Clone, PartialEq)]
pub enum ToolStatus {
    Installed,
    NotInstalled,
}

pub trait Tool {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self, i18n: &I18n) -> String;
    fn status(&self) -> ToolStatus;
    fn install(&self) -> Result<()>;
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()>;
    fn supports_fleets(&self) -> bool { false }
    /// Pour les outils qui ont besoin de Docker (ex: Open WebUI)
    fn requires_docker(&self) -> bool { false }
}