use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenCodeTool;

impl Tool for OpenCodeTool {
    fn id(&self) -> &str { "opencode" }
    fn name(&self) -> &str { "OpenCode" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.claude.description") }
    fn status(&self) -> ToolStatus {
        if shell::is_installed("opencode") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "npm install -g @opencode-ai/cli".into() } }
    }

    fn install(&self) -> Result<()> { shell::run("npm install -g @opencode-ai/cli")?; Ok(()) }

    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("opencode");
        display::info(&i18n.t("tool.opencode.auth"));
        Ok(())
    }
}