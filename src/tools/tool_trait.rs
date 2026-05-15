use anyhow::Result;
use crate::config::{I18n, WzllamaState};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ToolStatus {
    Installed,
    NotInstalled,
}

pub trait Tool {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self, i18n: &I18n) -> String;
    
    /// Retourne le statut de l'installation de l'outil
    #[allow(dead_code)]
    fn status(&self) -> ToolStatus;
    
    /// Retourne un message de statut internationalisé
    fn status_message(&self, i18n: &I18n) -> String {
        match self.status() {
            ToolStatus::Installed => i18n.t("tool.installed"),
            ToolStatus::NotInstalled => i18n.t("tool.not_installed"),
        }
    }
    
    /// Installe l'outil
    fn install(&self, _i18n: &I18n) -> Result<()> { Ok(()) }
    
    /// Met à jour l'outil
    fn update(&self, _i18n: &I18n) -> Result<()> { 
        anyhow::bail!("Update not supported for this tool");
    }
    
    /// Désinstalle l'outil
    fn uninstall(&self, _i18n: &I18n) -> Result<()> {
        anyhow::bail!("Uninstall not supported for this tool");
    }
    
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()>;
    
    fn supports_fleets(&self) -> bool { false }
    
    /// Pour les outils qui ont besoin de Docker (ex: Open WebUI)
    fn requires_docker(&self) -> bool { false }
}