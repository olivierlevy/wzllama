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
        if shell::is_installed("opencode") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self, _i18n: &I18n) -> Result<()> {
        shell::run_live("npm install -g @opencode-ai/cli")?;
        shell::exec("opencode auth login");
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        display::info(&i18n.t("tool.opencode.auth"));
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::run(&i18n.t_with_vars("tool.opencode.run_model", &[("model", &m)]));
                let cmd: String = format!("ollama launch opencode --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::comment(&i18n.t("tool.opencode.no_model"));
                println!("ollama launch opencode");
            }
        }
        Ok(())
    }
}

impl OpenCodeTool {
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.opencode.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run_quiet("sudo npm uninstall -g opencode-ai 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -f /usr/bin/opencode ~/.local/bin/opencode 2>/dev/null");
        let _ = shell::run_quiet("rm -rf ~/.opencode* 2>/dev/null");
        display::success(&i18n.t("tool.opencode.uninstalled"));
        Ok(())
    }
}
