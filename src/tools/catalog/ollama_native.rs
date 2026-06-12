use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};
use super::CatalogEntry;

/// A tool backed by a catalog entry.
/// Install/update/launch all delegate to `ollama launch <slug>` or the entry's `install_cmd`.
pub struct OllamaNativeTool {
    pub entry: CatalogEntry,
}

impl OllamaNativeTool {
    pub fn new(entry: CatalogEntry) -> Self {
        Self { entry }
    }
}

impl Tool for OllamaNativeTool {
    fn id(&self) -> &str {
        &self.entry.id
    }

    fn name(&self) -> &str {
        &self.entry.name
    }

    fn description(&self, _i18n: &I18n) -> String {
        self.entry.description_fallback.clone()
    }

    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        ToolStatus::from_installed(shell::is_installed_with_local_bin(&self.entry.slug))
    }

    fn install(&self, _i18n: &I18n) -> Result<()> {
        match &self.entry.install_cmd {
            Some(cmd) => {
                display::info(&format!("Installing {} via: {}", self.entry.name, cmd));
                shell::run_live(cmd)
            }
            None => {
                let cmd = format!("ollama launch {}", self.entry.slug);
                display::info(&format!("Installing {} via: {}", self.entry.name, cmd));
                shell::exec(&cmd)
            }
        }
    }

    fn update(&self, _i18n: &I18n) -> Result<()> {
        match &self.entry.install_cmd {
            Some(cmd) => {
                display::info(&format!("Updating {} via: {}", self.entry.name, cmd));
                shell::run_live(cmd)
            }
            None => {
                let cmd = format!("ollama launch {}", self.entry.slug);
                display::info(&format!("Updating {} via: {}", self.entry.name, cmd));
                shell::exec(&cmd)
            }
        }
    }

    fn launch(&self, _i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        let cmd = match model {
            Some(m) => format!("ollama launch {} --model {}", self.entry.slug, m),
            None => format!("ollama launch {}", self.entry.slug),
        };
        display::run(&cmd);
        shell::exec(&cmd)
    }
}
