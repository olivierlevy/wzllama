//! Models menu adapter for menu_api
//!
//! Migrated from wizard/menu_models.rs to use menu_api system with ModelsEngine.

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{MenuTree, MenuItem, MenuMetadata, ModelsEngineRunner};

/// Models menu runner using ModelsEngine
pub struct ModelsMenuRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> ModelsMenuRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    /// Get the menu tree structure
    pub fn menu_tree(&self) -> MenuTree {
        MenuTree::new("models")
            .with_metadata(MenuMetadata {
                title: Some(self.i18n.t("menu.main.models").to_string()),
                ..Default::default()
            })
            .with_root(
                MenuItem::branch("models")
                    .add_submenu(MenuItem::leaf("↩️ Retour"))
                    .add_submenu(MenuItem::leaf(&self.i18n.t("models.installed")).with_action("models_installed"))
                    .add_submenu(MenuItem::leaf(&self.i18n.t("models.by_org")).with_action("models_by_org"))
                    .add_submenu(MenuItem::leaf(&self.i18n.t("models.search")).with_action("models_search"))
            )
    }

    /// Run the models menu using ModelsEngineRunner
    pub fn run(&mut self) -> Result<()> {
        let mut runner = ModelsEngineRunner::new(self.i18n, self.state, self.hw);
        runner.run()
    }
}

/// Build the models menu tree
pub fn build_menu_tree(i18n: &I18n) -> MenuTree {
    MenuTree::new("models")
        .with_metadata(MenuMetadata {
            title: Some(i18n.t("menu.main.models").to_string()),
            ..Default::default()
        })
        .with_root(
            MenuItem::branch("models")
                .add_submenu(MenuItem::leaf("↩️ Retour"))
                .add_submenu(MenuItem::leaf(&i18n.t("models.installed")).with_action("models_installed"))
                .add_submenu(MenuItem::leaf(&i18n.t("models.by_org")).with_action("models_by_org"))
                .add_submenu(MenuItem::leaf(&i18n.t("models.search")).with_action("models_search"))
        )
}