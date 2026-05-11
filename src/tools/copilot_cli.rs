use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct CopilotCliTool;

impl Tool for CopilotCliTool {
    fn id(&self) -> &str { "copilot_cli" }
    fn name(&self) -> &str { "Copilot CLI" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.copilot.description") }
    fn status(&self) -> ToolStatus {
        if shell::is_installed("gh-copilot") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self) -> Result<()> {
        shell::run_live("gh extension install github/gh-copilot")?;
        Ok(())
    }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>) -> Result<()> {
        display::info(&i18n.t("tool.copilot.auth"));
        println!("gh auth login");
        println!("gh copilot");
        Ok(())
    }
}