//! Cleanup menu adapter for menu_api
//!
//! Migrated from wizard/menu_cleanup.rs to use menu_api system.

use crate::config::I18n;
use crate::config::WzllamaState;
use crate::core::HardwareInfo;
use crate::menu_api::arc_action::ArcActionRunner;
use crate::menu_api::{
    ActionDispatcher, ActionResult, ClosureAction, MenuHandler, MenuItem, MenuMetadata, MenuTree,
};
use anyhow::Result;

/// Cleanup menu runner using menu_api
pub struct CleanupMenuRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> CleanupMenuRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    /// Get the menu tree structure
    pub fn menu_tree(&self) -> MenuTree {
        let root = MenuItem::branch("cleanup")
            .add_submenu(MenuItem::leaf("↩️ Retour"))
            .add_submenu(
                MenuItem::leaf(&self.i18n.t("cleanup.menu_tools")).with_action("cleanup_tools"),
            )
            .add_submenu(
                MenuItem::leaf(&self.i18n.t("cleanup.menu_models")).with_action("cleanup_models"),
            );

        MenuTree::new("cleanup")
            .with_metadata(MenuMetadata {
                title: Some(self.i18n.t("cleanup.title")),
                ..Default::default()
            })
            .with_root(root)
    }

    /// Run the cleanup menu using MenuHandler
    pub fn run(&mut self) -> Result<()> {
        // Build menu tree (MenuHandler will display header with resources)

        // Create action runner to capture context
        let action_runner =
            ArcActionRunner::new(self.i18n.clone(), self.state.clone(), self.hw.clone());

        // Build menu tree
        let tree = self.menu_tree();

        // Create dispatcher with actions
        let mut dispatcher = ActionDispatcher::new();

        dispatcher.register(Box::new(action_runner.create_action(
            "cleanup_tools",
            "Nettoyer les outils",
            |i18n, state, _| crate::wizard::cleanup_tools::run(i18n, state),
        )));

        dispatcher.register(Box::new(action_runner.create_action(
            "cleanup_models",
            "Nettoyer les modèles",
            |i18n, state, _| crate::wizard::cleanup_models::run(i18n, state),
        )));

        // Run with MenuHandler (displays header and handles navigation)
        let mut handler = MenuHandler::new(tree, dispatcher, self.i18n, self.state, self.hw);
        handler.run()
    }
}
