//! Tools engine - manages tool installation and launch workflows
//!
//! This wraps the existing wizard::menu_tools logic with a menu_api interface.

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{MenuTree, MenuItem, MenuMetadata};

/// Tools engine that drives tool workflows
pub struct ToolsEngine<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> ToolsEngine<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    /// Run the tools workflow (delegates to wizard::menu_tools)
    pub fn run(&mut self) -> Result<()> {
        crate::wizard::menu_tools::run(self.i18n, self.state, self.hw)
    }
}

/// Build the tools menu tree
pub fn build_menu_tree(i18n: &I18n) -> MenuTree {
    MenuTree::new("tools")
        .with_metadata(MenuMetadata {
            title: Some(i18n.t("menu.main.tools").to_string()),
            ..Default::default()
        })
        .with_root(
            MenuItem::branch("tools")
                .add_submenu(MenuItem::leaf("↩️ Retour"))
                .add_submenu(MenuItem::leaf(&i18n.t("tools.docker")).with_action("tool_docker"))
                .add_submenu(MenuItem::leaf(&i18n.t("tools.ollama")).with_action("tool_ollama"))
                .add_submenu(MenuItem::leaf(&i18n.t("tools.open_webui")).with_action("tool_open_webui"))
                .add_submenu(MenuItem::leaf(&i18n.t("tools.openclaw")).with_action("tool_openclaw"))
                .add_submenu(MenuItem::leaf(&i18n.t("tools.hermes_agent")).with_action("tool_hermes_agent"))
                .add_submenu(MenuItem::leaf(&i18n.t("tools.opencode")).with_action("tool_opencode"))
                .add_submenu(MenuItem::leaf(&i18n.t("tools.codex")).with_action("tool_codex"))
                .add_submenu(MenuItem::leaf(&i18n.t("tools.droid")).with_action("tool_droid"))
        )
}

/// Tools engine runner wrapper
pub struct ToolsEngineRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> ToolsEngineRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut engine = ToolsEngine::new(self.i18n, self.state, self.hw);
        engine.run()
    }
}