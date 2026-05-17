use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::flatpak::FlatpakTool;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct ObsidianTool;

impl Tool for ObsidianTool {
    fn id(&self) -> &str { "obsidian" }
    fn name(&self) -> &str { "Obsidian" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.obsidian.description") }
    fn status(&self) -> ToolStatus { ObsidianTool::get_status() }
    fn install(&self, i18n: &I18n) -> Result<()> { ObsidianTool::install(i18n) }
    fn uninstall(&self, i18n: &I18n) -> Result<()> { ObsidianTool::uninstall(i18n) }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>) -> Result<()> { 
        ObsidianTool::launch(i18n) 
    }
    fn supports_fleets(&self) -> bool { false }
}

impl ObsidianTool {
    /// Check if Obsidian is installed
    pub fn get_status() -> ToolStatus {
        #[cfg(target_os = "linux")]
        {
            if shell::run("which obsidian").is_ok() 
                || std::path::Path::new("/app/bin/obsidian").exists()
                || FlatpakTool::is_installed("md.obsidian.Obsidian") {
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
    
    /// Check if nomic-embed-text is available
    pub fn check_embedding_model() -> bool {
        shell::run("ollama list | grep nomic-embed-text").is_ok()
    }
    
    /// Install Obsidian automatically
    pub fn install(i18n: &I18n) -> Result<()> {
        display::info(&i18n.t("tool.obsidian.install_info"));
        
        #[cfg(target_os = "linux")]
        {
            // Ensure Flatpak is installed first
            FlatpakTool::install()?;
            
            // Install Obsidian via Flatpak
            display::info("Installing Obsidian via Flatpak...");
            match FlatpakTool::install_app("md.obsidian.Obsidian") {
                Ok(_) => display::success(&i18n.t("tool.obsidian.installed")),
                Err(e) => display::warning(&format!("Installation failed: {}", e)),
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            display::warning("Please download Obsidian from https://obsidian.md/download and install it manually.");
        }
        
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
            else if FlatpakTool::is_installed("md.obsidian.Obsidian") {
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
        // Vérifier que Ollama est installé
        if shell::run("which ollama").is_err() {
            display::error(&i18n.t("ollama.not_installed"));
            display::info(&i18n.t("ollama.install_now"));
            anyhow::bail!("Ollama is required for Obsidian AI features");
        }
        
        display::info("📝 Obsidian is a local-first knowledge base");
        println!();
        display::info(&i18n.t("tool.obsidian.llm_setup"));
        display::info(&i18n.t("tool.obsidian.llm_step1"));
        display::info(&i18n.t("tool.obsidian.llm_step2"));
        display::info(&i18n.t("tool.obsidian.llm_step3"));
        display::info(&i18n.t("tool.obsidian.llm_step4"));
        display::info(&i18n.t("tool.obsidian.llm_step5"));
        println!();
        display::info(&i18n.t("tool.obsidian.embedding_model"));
        
        // Check if Ollama is running
        if shell::run("curl -s http://localhost:11434/api/tags > /dev/null 2>&1").is_ok() {
            display::success(&i18n.t("tool.obsidian.ollama_running"));
        } else {
            display::warning(&i18n.t("tool.obsidian.ollama_not_running"));
        }
        
        // Check embedding model
        if ObsidianTool::check_embedding_model() {
            display::success("✅ nomic-embed-text is available");
        } else {
            display::info("💡 Run 'ollama pull nomic-embed-text' for semantic search");
        }
        
        #[cfg(target_os = "linux")]
        {
            display::info(&i18n.t("tool.obsidian.cors_hint"));
        }
        
        println!();
        
        #[cfg(target_os = "linux")]
        {
            if shell::run("which obsidian").is_ok() {
                shell::run("obsidian &")?;
            } else if FlatpakTool::is_installed("md.obsidian.Obsidian") {
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