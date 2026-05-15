use anyhow::Result;
use colored::*;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenClawTool;

impl Tool for OpenClawTool {
    fn id(&self) -> &str { "openclaw" }
    fn name(&self) -> &str { "OpenClaw" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.openclaw.description") }
    fn supports_fleets(&self) -> bool { true }
    fn status(&self) -> ToolStatus {
        if shell::is_installed("openclaw") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self, _i18n: &I18n) -> Result<()> {
        match WzllamaState::load().last_model.as_deref() {
            Some(m) => {
                let cmd: String = format!("ollama launch openclaw --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                shell::run_live("npm install -g openclaw")?;
            }
        }
        Ok(())
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                //FIXME try openclaw --profile si ça ne marche pas via ollama
                display::comment(&i18n.t_with_vars("tool.openclaw.run_profile", &[("profile", &m)]));
                display::comment(&format!("openclaw --profile {}", m));
                display::run(&i18n.t_with_vars("tool.openclaw.run_model", &[("model", &m)]));
                let cmd: String = format!("ollama launch openclaw --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::comment(&i18n.t("tool.openclaw.no_model"));
                println!("ollama launch openclaw");
            }
        }
        Ok(())
    }
}

impl OpenClawTool {
    pub fn run_fleet(i18n: &I18n, project_name: &str) -> Result<()> {
        display::comment(&i18n.t("fleet.launching").bold());
        let cmd: String = format!("openclaw --profile {}", project_name.cyan());
        println!("{}", &cmd.yellow());
        shell::exec(&format!("openclaw --profile {}", project_name));
    }
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.openclaw.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run("openclaw uninstall --all --yes --non-interactive 2>/dev/null");
        let _ = shell::run("sudo npm uninstall -g openclaw 2>/dev/null");
        // Nettoyer les résidus
        let _ = shell::run("rm -f ~/.local/bin/openclaw 2>/dev/null");
        let _ = shell::run("rm -rf ~/.openclaw* 2>/dev/null");
        let _ = shell::run("systemctl --user disable openclaw-gateway-* 2>/dev/null");
        display::success(&i18n.t("tool.openclaw.uninstalled"));
        Ok(())
    }
}