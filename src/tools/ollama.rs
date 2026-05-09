use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::{shell, ollama_api};
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OllamaTool;

impl Tool for OllamaTool {
    fn id(&self) -> &str { "ollama" }
    fn name(&self) -> &str { "Ollama" }
    fn description(&self) -> &str { "Chat avec un modèle IA local" }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("ollama") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "curl -fsSL https://ollama.com/install.sh | sh".into() } }
    }

    fn install(&self) -> Result<()> {
        shell::run("curl -fsSL https://ollama.com/install.sh | sh")?;
        Ok(())
    }

    fn launch(&self, _state: &WzllamaState, model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        match model {
            Some(m) => { shell::run(&format!("ollama run {}", m))?; }
            None => { println!("ℹ️  Choisissez un modèle d'abord"); }
        }
        Ok(())
    }
}