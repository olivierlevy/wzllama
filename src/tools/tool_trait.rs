#![allow(dead_code)]

use anyhow::Result;
use crate::config::{I18n, WzllamaState};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ToolStatus {
    Installed,
    NotInstalled,
}

impl ToolStatus {
    pub fn from_installed(installed: bool) -> Self {
        if installed { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
}

pub trait Tool {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self, i18n: &I18n) -> String;
    
    /// Returns the installation status of the tool
    #[allow(dead_code)]
    fn status(&self, _state: &WzllamaState) -> ToolStatus;
    
    /// Returns an internationalized status message
    fn status_message(&self, i18n: &I18n) -> String {
        match self.status(&WzllamaState::default()) {
            ToolStatus::Installed => i18n.t("tool.installed"),
            ToolStatus::NotInstalled => i18n.t("tool.not_installed"),
        }
    }
    
    /// Installs the tool
    fn install(&self, _i18n: &I18n) -> Result<()> { Ok(()) }
    
    /// Updates the tool
    fn update(&self, _i18n: &I18n) -> Result<()> { 
        anyhow::bail!("Update not supported for this tool");
    }
    
    /// Uninstalls the tool
    fn uninstall(&self, _i18n: &I18n) -> Result<()> {
        anyhow::bail!("Uninstall not supported for this tool");
    }
    
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()>;
    
    fn supports_fleets(&self) -> bool { false }
    
    /// For tools that need Docker (e.g., Open WebUI)
    fn requires_docker(&self) -> bool { false }
}