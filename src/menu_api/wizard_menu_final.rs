//! Wizard menu implementation using MenuHandler with dynamic submenus
//!
//! This completely replaces wizard::menu_wizard with menu_api driven navigation

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{
    MenuTree, MenuItem, MenuHandler, ActionDispatcher, MenuMetadata,
    generate_models_menu, generate_tools_menu, generate_usecase_menu,
};

/// Wizard menu runner using MenuHandler
pub struct WizardMenuRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> WizardMenuRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    /// Build the complete wizard menu tree with dynamic submenus
    pub fn build_menu_tree(&self) -> MenuTree {
        MenuTree::new("wizard")
            .with_metadata(MenuMetadata {
                title: Some(self.i18n.t("menu.wizard.title").to_string()),
                ..Default::default()
            })
            .with_root(
                MenuItem::branch("wizard")
                    .add_submenu(MenuItem::leaf("↩️ Retour"))
                    .add_submenu(
                        MenuItem::leaf("🧙 Wizard")
                            .with_dynamic(|i18n, state, hw| generate_usecase_menu(i18n, state, hw))
                    )
                    .add_submenu(
                        MenuItem::leaf("📦 Models")
                            .with_dynamic(|i18n, state, hw| generate_models_menu(i18n, state, hw))
                    )
                    .add_submenu(
                        MenuItem::leaf("🛠️ Tools")
                            .with_dynamic(|i18n, state, hw| generate_tools_menu(i18n, state, hw))
                    )
            )
    }

    /// Run the wizard workflow
    pub fn run(&mut self) -> Result<()> {
        let tree = self.build_menu_tree();
        let dispatcher = ActionDispatcher::new();
        
        let mut handler = MenuHandler::new(
            tree,
            dispatcher,
            self.i18n,
            self.state,
            self.hw,
        );
        
        handler.run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_menu_structure() {
        let i18n = I18n::default();
        let mut state = WzllamaState::default();
        let hw = HardwareInfo::default();
        
        let runner = WizardMenuRunner::new(&i18n, &mut state, &hw);
        let tree = runner.build_menu_tree();
        
        assert_eq!(tree.id, "wizard");
        assert!(tree.root.submenus.len() > 0);
    }
}