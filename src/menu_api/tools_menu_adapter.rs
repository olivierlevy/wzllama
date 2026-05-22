//! Tools menu adapter for menu_api
//!
//! Migrated from wizard/menu_tools.rs to use menu_api system with ToolsEngine.

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{MenuTree, MenuItem, MenuMetadata, ToolsEngineRunner};

/// Tools menu runner using ToolsEngine
pub struct ToolsMenuRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> ToolsMenuRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    /// Get the menu tree structure
    pub fn menu_tree(&self) -> MenuTree {
        MenuTree::new("tools")
            .with_metadata(MenuMetadata {
                title: Some(self.i18n.t("menu.main.tools").to_string()),
                ..Default::default()
            })
            .with_root(
                MenuItem::branch("tools")
                    .add_submenu(MenuItem::leaf("↩️ Retour"))
                    .add_submenu(MenuItem::leaf(&self.i18n.t("tools.docker")).with_action("tool_docker"))
                    .add_submenu(MenuItem::leaf(&self.i18n.t("tools.ollama")).with_action("tool_ollama"))
                    .add_submenu(MenuItem::leaf(&self.i18n.t("tools.open_webui")).with_action("tool_open_webui"))
                    .add_submenu(MenuItem::leaf(&self.i18n.t("tools.openclaw")).with_action("tool_openclaw"))
                    .add_submenu(MenuItem::leaf(&self.i18n.t("tools.hermes_agent")).with_action("tool_hermes_agent"))
                    .add_submenu(MenuItem::leaf(&self.i18n.t("tools.opencode")).with_action("tool_opencode"))
            )
    }

    /// Run the tools menu using ToolsEngineRunner
    pub fn run(&mut self) -> Result<()> {
        let mut runner = ToolsEngineRunner::new(self.i18n, self.state, self.hw);
        runner.run()
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
        )
}