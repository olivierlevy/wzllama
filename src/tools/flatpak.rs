use anyhow::Result;
use crate::core::shell;
use crate::core::system::detect_distro;
use crate::display;

/// Flatpak utility tool - not exposed in menus
pub struct FlatpakTool;

impl FlatpakTool {
    /// Install Flatpak on any Linux distribution
    pub fn install() -> Result<()> {
        if shell::run("flatpak --version").is_ok() {
            return Ok(());  // Already installed
        }
        
        display::info("Installing Flatpak...");
        
        match detect_distro() {
            "debian" => {
                shell::run_live("sudo apt update && sudo apt install -y flatpak")?;
            }
            "fedora" => {
                shell::run_live("sudo dnf install -y flatpak")?;
            }
            "rhel" => {
                shell::run_live("sudo yum install -y flatpak")?;
            }
            "arch" => {
                shell::run_live("sudo pacman -S --noconfirm flatpak")?;
            }
            "opensuse" => {
                shell::run_live("sudo zypper install -y flatpak")?;
            }
            "gentoo" => {
                display::warning("On Gentoo, enable the ~amd64 keyword and run: emerge sys-apps/flatpak");
                anyhow::bail!("Manual installation required on Gentoo");
            }
            "void" => {
                shell::run_live("sudo xbps-install -S flatpak")?;
            }
            "nixos" => {
                display::warning("On NixOS, add 'services.flatpak.enable = true;' to /etc/nixos/configuration.nix and run: sudo nixos-rebuild switch");
                anyhow::bail!("Manual configuration required on NixOS");
            }
            _ => {
                display::warning("Unknown distribution. Please install Flatpak manually.");
                anyhow::bail!("Cannot install Flatpak on unknown distribution");
            }
        }
        
        // Add Flathub repository
        display::info("Adding Flathub repository...");
        if shell::run("flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo").is_ok() {
            display::success("Flatpak installed and Flathub added!");
        } else {
            display::warning("Flatpak installed but could not add Flathub repository");
        }
        
        Ok(())
    }
    
    /// Install a flatpak application
    pub fn install_app(app_id: &str) -> Result<()> {
        // Ensure Flatpak is installed first
        if shell::run("flatpak --version").is_err() {
            Self::install()?;
        }
        
        // Check if already installed
        if Self::is_installed(app_id) {
            display::info(&format!("{} is already installed", app_id));
            return Ok(());
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