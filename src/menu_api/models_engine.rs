//! Models engine - manages model selection and download workflows
//!
//! This wraps the existing wizard::menu_models logic with a menu_api interface.

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{MenuTree, MenuItem, MenuMetadata};
use crate::menu_api::wizard_helpers::UseCase;

/// Models engine that drives model workflows
pub struct ModelsEngine<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> ModelsEngine<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    /// Run the models workflow (delegates to wizard::menu_models)
    pub fn run(&mut self) -> Result<()> {
        crate::wizard::menu_models::run(self.i18n, self.state, self.hw)
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

/// Models engine runner wrapper
pub struct ModelsEngineRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> ModelsEngineRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut engine = ModelsEngine::new(self.i18n, self.state, self.hw);
        engine.run()
    }
}