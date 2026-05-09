use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct HermesTool;

impl Tool for HermesTool {
    fn id(&self) -> &str { "hermes_agent" }
    fn name(&self) -> &str { "Hermes Agent" }
    fn description(&self) -> &str { "Agent IA auto-améliorant de Nous Research" }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("hermes-agent") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "pip install hermes-agent".into() } }
    }

    fn install(&self) -> Result<()> { shell::run("pip install hermes-agent")?; Ok(()) }

    fn launch(&self, _state: &WzllamaState, model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        match model {
            Some(m) => println!("hermes-agent --model ollama/{}", m),
            None => println!("hermes-agent"),
        }
        Ok(())
    }
}