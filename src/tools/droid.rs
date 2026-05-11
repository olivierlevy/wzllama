use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct DroidTool;

impl Tool for DroidTool {
    fn id(&self) -> &str { "droid" }
    fn name(&self) -> &str { "Droid" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.droid.description") }
    fn status(&self) -> ToolStatus {
        if shell::is_installed("droid") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self) -> Result<()> {
        shell::run_live("npm install -g @factoryai/droid")?;
        Ok(())
    }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("droid");
        display::info(&i18n.t("tool.droid.xdg"));
        Ok(())
    }
}

impl DroidTool {
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.droid.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run("sudo npm uninstall -g @factoryai/droid 2>/dev/null");
        let _ = shell::run("rm -rf ~/.factoryai 2>/dev/null");
        display::success(&i18n.t("tool.droid.uninstalled"));
        Ok(())
    }
}
