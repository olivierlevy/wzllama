use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenCodeTool;

impl Tool for OpenCodeTool {
    fn id(&self) -> &str { "opencode" }
    fn name(&self) -> &str { "OpenCode" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.opencode.description") }
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

impl OpenCodeTool {
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.opencode.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run("sudo npm uninstall -g opencode-ai 2>/dev/null");
        let _ = shell::run("sudo rm -f /usr/bin/opencode ~/.local/bin/opencode 2>/dev/null");
        let _ = shell::run("rm -rf ~/.opencode* 2>/dev/null");
        display::success(&i18n.t("tool.opencode.uninstalled"));
        Ok(())
    }
}
