use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct PoolTool;

impl Tool for PoolTool {
    fn id(&self) -> &str { "pool" }
    fn name(&self) -> &str { "Pool" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.pool.description") }
    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        if shell::is_installed("pool") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        PoolTool::install(i18n)
    }
    fn update(&self, i18n: &I18n) -> Result<()> {
        PoolTool::update(i18n)
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        PoolTool::launch(i18n, state, model)
    }
    
    fn supports_agentic(&self) -> bool { true }
}

impl PoolTool {
    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("ℹ️  https://github.com/poolsideai/pool");
        #[cfg(unix)]
        shell::run_live("curl -fsSL https://downloads.poolside.ai/pool/install.sh | sh")?;
        #[cfg(not(unix))]
        {
            display::info("Opening Pool download page...");
            shell::open_url("https://github.com/poolsideai/pool");
            display::info("Download and install Pool from https://github.com/poolsideai/pool");
        }
        Ok(())
    }
    pub fn update(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("Updating Pool...");
        #[cfg(unix)]
        shell::run_live("curl -fsSL https://downloads.poolside.ai/pool/install.sh | sh")?;
        #[cfg(not(unix))]
        shell::open_url("https://github.com/poolsideai/pool");
        display::success("✅ Pool updated");
        Ok(())
    }
    pub fn launch(i18n: &I18n, _state: &WzllamaState, _model: Option<&str>) -> Result<()> {
        let _ = i18n;
        println!("ℹ️  https://github.com/poolsideai/pool");
        println!("pool");
        shell::exec("pool")
    }
}