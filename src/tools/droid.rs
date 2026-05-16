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
        if shell::is_installed_with_local_bin("droid") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        DroidTool::install(i18n)
    }
    fn update(&self, i18n: &I18n) -> Result<()> {
        DroidTool::update(i18n)
    }
    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        DroidTool::uninstall(i18n)
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        DroidTool::launch(i18n, state, model)
    }
}

impl DroidTool {
    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        let xdg_cmd = crate::core::system::get_package_install_command("xdg-utils")?;
        shell::run_live(&xdg_cmd)?;
        shell::run_live("npm install -g @factoryai/droid")?;
        Ok(())
    }
    pub fn update(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("Updating Droid...");
        shell::run_live("npm update -g @factoryai/droid")?;
        display::success("✅ Droid updated");
        Ok(())
    }
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.droid.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run_quiet("sudo npm uninstall -g @factoryai/droid 2>/dev/null");
        let _ = shell::run_quiet("rm -rf ~/.factoryai 2>/dev/null");
        display::success(&i18n.t("tool.droid.uninstalled"));
        Ok(())
    }
    pub fn launch(i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        display::info(&i18n.t("tool.droid.xdg"));
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::run(&i18n.t_with_vars("tool.droid.run_model", &[("model", &m)]));
                let cmd: String = format!("ollama launch droid --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::comment(&i18n.t("tool.droid.no_model"));
                println!("ollama launch droid");
            }
        }
        Ok(())
    }
}
