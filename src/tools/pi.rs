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
    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        if shell::is_installed_with_local_bin("pi") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        PiTool::install(i18n)
    }
    fn update(&self, i18n: &I18n) -> Result<()> {
        PiTool::update(i18n)
    }
    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        PiTool::uninstall(i18n)
    }
    // Le binaire s'appelle "pi"
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        PiTool::launch(i18n, state, model)
    }
}

impl PiTool {
    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        match WzllamaState::load().last_model.as_deref() {
            Some(m) => {
                let cmd: String = format!("ollama launch pi --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                shell::run_live("npm install -g pi-agent")?;
            }
        }
        Ok(())
    }
    pub fn update(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("Updating Pi...");
        match WzllamaState::load().last_model.as_deref() {
            Some(m) => {
                let cmd: String = format!("ollama launch pi --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                shell::run_live("npm update -g pi-agent")?;
            }
        }
        display::success("✅ Pi updated");
        Ok(())
    }
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.pi.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run_quiet("sudo npm uninstall -g @earendil-works/pi-coding-agent 2>/dev/null").ok();
        let _ = shell::run_quiet("rm -rf ~/.pi 2>/dev/null").ok();
        let _ = shell::run_quiet("rm -rf ~/.local/bin/pi 2>/dev/null").ok();
        display::success(&i18n.t("tool.pi.uninstalled"));
        Ok(())
    }
    pub fn launch(i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::comment(&format!("pi --model ollama/{}", m));
                display::run(&i18n.t_with_vars("tool.pi.run_model", &[("model", &m)]));
                let cmd: String = format!("ollama launch pi --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::comment(&i18n.t("tool.pi.no_model"));
                println!("ollama launch pi");
            }
        }
        Ok(())
    }
}