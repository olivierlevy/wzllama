use anyhow::Result;
use crate::core::shell;
use crate::display;

/// Flatpak utility tool - not exposed in menus
pub struct FlatpakTool;

impl FlatpakTool {
    /// Install a flatpak application
    pub fn install(app_id: &str) -> Result<()> {
        // Check if flatpak is installed
        if shell::run("which flatpak").is_err() {
            anyhow::bail!("Flatpak is not installed. Install it with: sudo apt install flatpak");
        }
        
        display::info(&format!("Installing {} via Flatpak...", app_id));
        shell::run_live(&format!("flatpak install flathub {} -y", app_id))?;
        display::success(&format!("{} installed!", app_id));
        Ok(())
    }
    
    /// Check if a flatpak app is installed
    pub fn is_installed(app_id: &str) -> bool {
        shell::run(&format!("flatpak info {}", app_id)).is_ok()
    }
    
    /// Uninstall a flatpak application
    pub fn uninstall(app_id: &str) -> Result<()> {
        if !Self::is_installed(app_id) {
            display::info(&format!("{} is not installed.", app_id));
            return Ok(());
        }
        
        display::info(&format!("Uninstalling {}...", app_id));
        shell::run_live(&format!("flatpak uninstall {} -y", app_id))?;
        display::success(&format!("{} uninstalled!", app_id));
        Ok(())
    }
}