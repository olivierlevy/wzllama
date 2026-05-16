use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct HermesTool;

impl Tool for HermesTool {
    fn id(&self) -> &str { "hermes_agent" }
    fn name(&self) -> &str { "Hermes Agent" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.hermes.description") }
    fn status(&self) -> ToolStatus {
        if shell::is_installed_with_local_bin("hermes") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        HermesTool::install(i18n)
    }
    fn update(&self, i18n: &I18n) -> Result<()> {
        HermesTool::update(i18n)
    }
    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        HermesTool::uninstall(i18n)
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        HermesTool::launch(i18n, state, model)
    }
}

impl HermesTool {
    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        match WzllamaState::load().last_model.as_deref() {
            Some(m) => {
                let cmd: String = format!("ollama launch hermes --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                shell::run_live("curl -fsSL https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh | bash -s -- --no-venv --skip-setup")?;
            }
        }
        Ok(())
    }
    pub fn update(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("Updating Hermes...");
        // Re-run install script for update
        shell::run_live("curl -fsSL https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh | bash -s -- --no-venv --skip-setup")?;
        display::success("✅ Hermes updated");
        Ok(())
    }
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.hermes.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run_quiet("sudo rm -f /usr/bin/hermes ~/.local/bin/hermes 2>/dev/null");
        let _ = shell::run_quiet("rm -rf ~/.hermes* 2>/dev/null");
        display::success(&i18n.t("tool.hermes.uninstalled"));
        Ok(())
    }
    pub fn launch(i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::run(&i18n.t_with_vars("tool.hermes.run_model", &[("model", &m)]));
                let cmd: String = format!("ollama launch hermes --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::comment(&i18n.t("tool.hermes.no_model"));
                println!("ollama launch hermes");
            }
        }
        Ok(())
    }
}
