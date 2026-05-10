use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct HermesTool;

impl Tool for HermesTool {
    fn id(&self) -> &str { "hermes_agent" }
    fn name(&self) -> &str { "Hermes Agent" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.hermes.description") }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("hermes") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "curl -fsSL https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh | bash".into() } }
    }

    fn install(&self) -> Result<()> {
        shell::run("curl -fsSL https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh | bash")?;
        Ok(())
    }

    fn launch(&self, i18n: &I18n, _state: &WzllamaState, model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        match model {
            Some(m) => println!("hermes --model ollama/{}", m),
            None => {
                display::info(&i18n.t("tool.hermes.no_model"));
                println!("hermes setup");
            }
        }
        Ok(())
    }
}