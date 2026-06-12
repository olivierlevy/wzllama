//! Real wizard action wrapper for menu_api execution
//!
//! This provides concrete ToolAction implementations that can execute
//! existing wizard functions.

use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{MenuItem, MenuTree};
use std::sync::{Arc, Mutex};

/// Thread-safe container for wizard execution context
pub struct WizardContext {
    pub i18n: Arc<I18n>,
    pub state: Arc<Mutex<WzllamaState>>,
    pub hw: Arc<HardwareInfo>,
}

impl Clone for WizardContext {
    fn clone(&self) -> Self {
        Self {
            i18n: Arc::clone(&self.i18n),
            state: Arc::clone(&self.state),
            hw: Arc::clone(&self.hw),
        }
    }
}

/// Create a wizard context for menu handling
pub fn create_wizard_context(i18n: I18n, state: WzllamaState, hw: HardwareInfo) -> WizardContext {
    WizardContext {
        i18n: Arc::new(i18n),
        state: Arc::new(Mutex::new(state)),
        hw: Arc::new(hw),
    }
}

/// Build the cleanup menu tree structure
pub fn build_cleanup_menu_tree() -> MenuTree {
    let root = MenuItem::branch("cleanup")
        .add_submenu(MenuItem::leaf("↩️ Retour"))
        .add_submenu(MenuItem::leaf("🧹 Nettoyer les outils").with_action("cleanup_tools"))
        .add_submenu(MenuItem::leaf("🧹 Nettoyer les modèles").with_action("cleanup_models"));

    MenuTree::new("cleanup").with_root(root)
}

// Note: The actual WizardToolAction implementation requires trait object
// compatibility which is complex with Fn closures. For now, the adapters
// in menu_api/*_adapter.rs files provide the bridge between MenuTree
// structures and wizard functions.
