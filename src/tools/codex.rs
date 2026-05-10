use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct CodexTool;

impl Tool for CodexTool {
    fn id(&self) -> &str { "codex" }
    fn name(&self) -> &str { "Codex" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.codex.description") }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("codex") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "npm install -g @openai/codex".into() } }
    }

    fn install(&self) -> Result<()> { shell::run("npm install -g @openai/codex")?; Ok(()) }

    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("codex");
        display::info(&i18n.t("tool.codex.auth"));
        Ok(())
    }
}