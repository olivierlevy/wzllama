use anyhow::Result;
use dialoguer::Confirm;
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
        if shell::is_installed("codex") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self) -> Result<()> {
        shell::run_live("npm install -g @openai/codex")?;
        Ok(())
    }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("codex");
        display::info(&i18n.t("tool.codex.auth"));
        Ok(())
    }
}

impl CodexTool {
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.codex.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run("sudo npm uninstall -g @openai/codex 2>/dev/null");
        display::success(&i18n.t("tool.codex.uninstalled"));
        Ok(())
    }
}
