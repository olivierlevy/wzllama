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
        if shell::is_installed_with_local_bin("codex") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self, _i18n: &I18n) -> Result<()> {
        shell::run_live("npm install -g @openai/codex")?;
        Ok(())
    }
    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        CodexTool::uninstall(i18n)
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        display::info(&i18n.t("tool.codex.auth"));
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::run(&i18n.t_with_vars("tool.codex.run_model", &[("model", &m)]));
                let cmd: String = format!("ollama launch codex --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::comment(&i18n.t("tool.codex.no_model"));
                println!("ollama launch codex");
            }
        }
        Ok(())
    }
}

impl CodexTool {
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.codex.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run_quiet("sudo npm uninstall -g @openai/codex 2>/dev/null").ok();
        let _ = shell::run_quiet("rm -rf ~/.codex 2>/dev/null").ok();
        let _ = shell::run_quiet("rm -rf ~/.local/bin/codex 2>/dev/null").ok();
        display::success(&i18n.t("tool.codex.uninstalled"));
        Ok(())
    }
}
