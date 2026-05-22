//! Wizard menu adapter for menu_api
//!
//! Migrated from wizard/menu_wizard.rs to use menu_api system with WizardEngine.

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{MenuTree, MenuItem, MenuMetadata, WizardEngineRunner};
use crate::menu_api::wizard_helpers::UseCase;

/// Wizard menu runner using WizardEngine
pub struct WizardMenuRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> WizardMenuRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    /// Get the menu tree structure
    pub fn menu_tree(&self) -> MenuTree {
        let use_cases = UseCase::all();
        let mut items = vec![MenuItem::leaf("↩️ Retour")];
        items.extend(use_cases.iter().map(|uc| {
            MenuItem::leaf(&uc.display_name(self.i18n)).with_action(&format!("wizard_usecase_{}", uc.as_str()))
        }));

        MenuTree::new("wizard")
            .with_metadata(MenuMetadata {
                title: Some("🤖 Wizard".to_string()),
                ..Default::default()
            })
            .with_root(MenuItem::branch("wizard").add_submenus(items))
    }

    /// Run the wizard menu using WizardEngineRunner
    pub fn run(&mut self) -> Result<()> {
        let mut runner = WizardEngineRunner::new(self.i18n, self.state, self.hw);
        runner.run()
    }
}

/// Build the wizard menu tree (public for menu_api use)
pub fn build_menu_tree() -> MenuTree {
    let use_cases = UseCase::all();
    let mut items = vec![MenuItem::leaf("↩️ Retour")];
    items.extend(use_cases.iter().map(|uc| {
        MenuItem::leaf(&uc.display_name(&crate::config::I18n::default())).with_action(&format!("usecase_{}", uc.as_str()))
    }));

    MenuTree::new("wizard")
        .with_metadata(MenuMetadata {
            title: Some("🤖 Wizard".to_string()),
            ..Default::default()
        })
        .with_root(MenuItem::branch("wizard").add_submenus(items))
}