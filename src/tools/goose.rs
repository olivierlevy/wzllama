use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct GooseTool;

impl Tool for GooseTool {
    fn id(&self) -> &str { "goose" }
    fn name(&self) -> &str { "Goose" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.goose.description") }
    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        if shell::is_installed("goose") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        GooseTool::install(i18n)
    }
    fn update(&self, i18n: &I18n) -> Result<()> {
        GooseTool::update(i18n)
    }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>) -> Result<()> {
        GooseTool::launch(i18n)
    }
    
    fn supports_agentic(&self) -> bool { true }
}

impl GooseTool {
    pub fn install(i18n: &I18n) -> Result<()> {
        display::info(&i18n.t("tool.goose.install_info"));
        #[cfg(unix)]
        shell::run_live("curl -fsSL https://github.com/aaif-goose/goose/releases/download/stable/download_cli.sh | CONFIGURE=false bash")?;
        #[cfg(not(unix))]
        {
            display::info("Opening Goose download page...");
            shell::open_url("https://github.com/block/goose/releases");
            display::info("Download the Windows installer from https://github.com/block/goose/releases");
        }
        display::success(&i18n.t("tool.goose.installed"));
        Ok(())
    }
    
    pub fn update(i18n: &I18n) -> Result<()> {
        display::info(&i18n.t("tool.goose.updating"));
        shell::run_live("goose update")?;
        display::success(&i18n.t("tool.goose.updated"));
        Ok(())
    }
    
    pub fn launch(i18n: &I18n) -> Result<()> {
        display::info(&i18n.t("tool.goose.launch_hint"));
        println!("goose");
        Ok(())
    }
}