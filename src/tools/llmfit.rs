use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct LLMFitTool;

impl Tool for LLMFitTool {
    fn id(&self) -> &str { "llmfit" }
    fn name(&self) -> &str { "LLMFit" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.llmfit.description") }
    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        // Check if llmfit is installed via uv or as binary
        if shell::is_installed_with_local_bin("llmfit") || shell::run_quiet("uv tool list 2>/dev/null | grep -q llmfit").is_ok() {
            ToolStatus::Installed
        } else {
            ToolStatus::NotInstalled
        }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        LLMFitTool::install(i18n)
    }
    fn update(&self, i18n: &I18n) -> Result<()> {
        LLMFitTool::update(i18n)
    }
    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        LLMFitTool::uninstall(i18n)
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        LLMFitTool::launch(i18n, state, model)
    }
}

impl LLMFitTool {
    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("Installing LLMFit...");
        shell::run_live("uv tool install -U llmfit")?;
        display::success("✅ LLMFit installed");
        Ok(())
    }
    pub fn update(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("Updating LLMFit...");
        shell::run_live("uv tool install -U llmfit")?;
        display::success("✅ LLMFit updated");
        Ok(())
    }
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.llmfit.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run_quiet("uv tool uninstall llmfit 2>/dev/null");
        let _ = shell::run_quiet("rm -f ~/.local/bin/llmfit 2>/dev/null");
        display::success(&i18n.t("tool.llmfit.uninstalled"));
        Ok(())
    }
    pub fn launch(i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let _ = model;
        let _ = state;
        display::run(&i18n.t("tool.llmfit.run"));
        // Exit wzllama and launch llmfit standalone
        println!("Exiting wzllama and launching: uvx llmfit");
        shell::exec("uvx llmfit")
    }
}