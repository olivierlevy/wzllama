use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct ObsidianTool;

impl Tool for ObsidianTool {
    fn id(&self) -> &str { "obsidian" }
    fn name(&self) -> &str { "Obsidian" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.obsidian.description") }
    fn status(&self) -> ToolStatus {
        #[cfg(target_os = "linux")]
        {
            // Check if obsidian is installed via flatpak or apt
            if shell::run("which obsidian").is_ok() 
                || std::path::Path::new("/app/bin/obsidian").exists()
                || shell::run("flatpak info md.obsidian.Obsidian").is_ok() {
                ToolStatus::Installed
            } else {
                ToolStatus::NotInstalled
            }
        }
        #[cfg(target_os = "macos")]
        {
            if shell::run("which obsidian").is_ok() 
                || std::path::Path::new("/Applications/Obsidian.app").exists() {
                ToolStatus::Installed
            } else {
                ToolStatus::NotInstalled
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            ToolStatus::NotInstalled
        }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        ObsidianTool::install(i18n)
    }
    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        ObsidianTool::uninstall(i18n)
    }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>) -> Result<()> {
        ObsidianTool::launch(i18n)
    }
}

impl ObsidianTool {
    pub fn install(i18n: &I18n) -> Result<()> {
        display::info(&i18n.t("tool.obsidian.install_info"));
        
        #[cfg(target_os = "linux")]
        {
            // Try flatpak first (recommended for Linux)
            if shell::run("which flatpak").is_ok() {
                display::info("Installing Obsidian via Flatpak...");
                shell::run_live("flatpak install flathub md.obsidian.Obsidian -y")?;
            } else {
                display::warning("Flatpak not found. Please install Obsidian manually from https://obsidian.md/download");
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            display::info("Please download Obsidian from https://obsidian.md/download and install it manually.");
        }
        
        display::success(&i18n.t("tool.obsidian.installed"));
        Ok(())
    }
    
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new()
            .with_prompt(i18n.t("tool.obsidian.uninstall_confirm"))
            .default(false)
            .interact()?
        {
            return Ok(());
        }
        
        #[cfg(target_os = "linux")]
        {
            // Try apt first
            if shell::run("dpkg -l obsidian 2>/dev/null").is_ok() {
                display::info("Removing Obsidian via apt...");
                shell::run_live("sudo apt remove obsidian -y")?;
            }
            // Try flatpak
            else if shell::run("flatpak info md.obsidian.Obsidian").is_ok() {
                display::info("Removing Obsidian via Flatpak...");
                shell::run_live("flatpak uninstall md.obsidian.Obsidian -y")?;
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if std::path::Path::new("/Applications/Obsidian.app").exists() {
                display::info("Removing Obsidian.app...");
                shell::run("rm -rf /Applications/Obsidian.app")?;
            }
        }
        
        display::success(&i18n.t("tool.obsidian.uninstalled"));
        Ok(())
    }
    
    pub fn launch(i18n: &I18n) -> Result<()> {
        display::info("📝 Obsidian is a local-first knowledge base");
        display::info("💡 Tip: Use with local LLM models for AI-powered note-taking");
        println!();
        
        #[cfg(target_os = "linux")]
        {
            // Try different ways to launch
            if shell::run("which obsidian").is_ok() {
                shell::run("obsidian &")?;
            } else if shell::run("flatpak info md.obsidian.Obsidian").is_ok() {
                shell::run("flatpak run md.obsidian.Obsidian &")?;
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            shell::run("open -a Obsidian")?;
        }
        
        Ok(())
    }
}