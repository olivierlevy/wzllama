use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct PiTool;

impl Tool for PiTool {
    fn id(&self) -> &str { "pi" }
    fn name(&self) -> &str { "Pi" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.pi.description") }
    fn status(&self) -> ToolStatus {
        if shell::is_installed("pi") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self) -> Result<()> {
        shell::run_live("npm install -g pi-agent")?;
        Ok(())
    }
    // Le binaire s'appelle "pi"
    fn launch(&self, _i18n: &I18n, _state: &WzllamaState, model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        match model {
            Some(m) => println!("pi --model ollama/{}", m),
            None => println!("pi"),
        }
        Ok(())
    }
}

impl PiTool {
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.pi.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run("sudo npm uninstall -g @mariozechner/pi-coding-agent 2>/dev/null");
        let _ = shell::run("rm -rf ~/.pi-agent 2>/dev/null");
        display::success(&i18n.t("tool.pi.uninstalled"));
        Ok(())
    }
}