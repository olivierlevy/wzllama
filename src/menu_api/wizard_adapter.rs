//! Wizard adapter for integrating existing menu functions with menu_api
//!
//! This module provides adapters to wrap existing `fn run(i18n, state, hw)` style
//! menu functions into the new ToolAction-based system.

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{
    tool_action::{ActionContext, ActionResult, ToolAction},
    menu_tree::{MenuTree, MenuConfig, MenuConfigItem},
    ActionDispatcher,
};

/// Wrapper for existing wizard menu functions
/// 
/// Existing wizard menus have signature: `fn run(&I18n, &mut WzllamaState, &HardwareInfo) -> Result<()>`
#[allow(dead_code)]
pub struct WizardAction<F> 
where
    F: Fn(&I18n, &mut WzllamaState, &HardwareInfo) -> Result<()> + Send + Sync,
{
    /// Unique identifier for this action
    pub id: String,
    /// Display name
    pub name: String,
    /// The wrapped function
    pub func: F,
    /// Path to the submenu this action represents (optional)
    pub submenu_path: Option<String>,
}

impl<F> WizardAction<F>
where
    F: Fn(&I18n, &mut WzllamaState, &HardwareInfo) -> Result<()> + Send + Sync,
{
    /// Create a new wizard action
    #[allow(dead_code)]
    pub fn new(id: &str, name: &str, func: F) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            func,
            submenu_path: None,
        }
    }

    /// Set the submenu path for nested navigation
    #[allow(dead_code)]
    pub fn with_submenu_path(mut self, path: &str) -> Self {
        self.submenu_path = Some(path.to_string());
        self
    }
}

impl<F> ToolAction for WizardAction<F>
where
    F: Fn(&I18n, &mut WzllamaState, &HardwareInfo) -> Result<()> + Send + Sync,
{
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, _ctx: &ActionContext) -> Result<ActionResult> {
        // Extract i18n, state, hw from context parameters
        // This requires the context to have these values stored
        Err(anyhow::anyhow!(
            "WizardAction requires I18n, WzllamaState, and HardwareInfo in context. \
             Use WizardActionRunner to execute directly."
        ))
    }
}

/// Runner for WizardAction that has access to the required context
#[allow(dead_code)]
pub struct WizardActionRunner<'a> {
    pub i18n: &'a I18n,
    pub state: &'a mut WzllamaState,
    pub hw: &'a HardwareInfo,
}

impl<'a> WizardActionRunner<'a> {
    /// Execute a wizard action with the current context
    #[allow(dead_code)]
    pub fn execute_action<F>(&mut self, action: &WizardAction<F>) -> Result<()>
    where
        F: Fn(&I18n, &mut WzllamaState, &HardwareInfo) -> Result<()> + Send + Sync,
    {
        (action.func)(self.i18n, self.state, self.hw)
    }
}

/// Builder for creating MenuTree from existing wizard modules
#[allow(dead_code)]
pub struct WizardMenuBuilder {
    items: Vec<MenuConfigItem>,
}

impl WizardMenuBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a menu item for a wizard function
    pub fn add_item(mut self, label: &str, action_id: &str) -> Self {
        self.items.push(MenuConfigItem {
            label: label.to_string(),
            action_id: Some(action_id.to_string()),
            children: None,
            condition: None,
        });
        self
    }

    /// Add a submenu branch
    pub fn add_branch(mut self, label: &str, children: Vec<MenuConfigItem>) -> Self {
        self.items.push(MenuConfigItem {
            label: label.to_string(),
            action_id: None,
            children: Some(children),
            condition: None,
        });
        self
    }

    /// Build the MenuTree
    pub fn build(self, title: &str) -> MenuTree {
        let config = MenuConfig {
            version: Some("1.0".to_string()),
            title: Some(title.to_string()),
            description: Some("Migrated wizard menu".to_string()),
            items: self.items,
        };
        MenuTree::from(config)
    }
}

/// Creates a MenuTree for the main wizard menu
#[allow(dead_code)]
pub fn create_main_menu_tree() -> MenuTree {
    WizardMenuBuilder::new()
        .add_branch(&i18n_key_to_label("menu.main.wizard"), vec![
            MenuConfigItem {
                label: i18n_key_to_label("menu.main.models"),
                action_id: Some("wizard_models".to_string()),
                children: None,
                condition: None,
            },
            MenuConfigItem {
                label: i18n_key_to_label("menu.main.scientific"),
                action_id: Some("wizard_scientific".to_string()),
                children: None,
                condition: None,
            },
        ])
        .add_item(&i18n_key_to_label("menu.main.tools"), "wizard_tools")
        .add_item(&i18n_key_to_label("menu.main.cleanup"), "wizard_cleanup")
        .add_item(&i18n_key_to_label("menu.main.config"), "wizard_config")
        .add_item(&i18n_key_to_label("menu.main.language"), "wizard_language")
        .add_item(&i18n_key_to_label("menu.main.quit"), "wizard_quit")
        .build("wzllama Main Menu")
}

/// Convert i18n key to a placeholder label (real implementation would use i18n)
#[allow(dead_code)]
fn i18n_key_to_label(key: &str) -> String {
    key.split('.').next_back().unwrap_or(key).to_string()
}

/// Adapter that runs the new menu_api system with existing wizard functions
#[allow(dead_code)]
pub struct WizardAdapter {
    dispatcher: ActionDispatcher,
}

impl WizardAdapter {
    /// Create a new adapter with registered wizard actions
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            dispatcher: ActionDispatcher::new(),
        }
    }

    /// Register a wizard action by its function
    #[allow(dead_code)]
    pub fn register<F>(&mut self, action: WizardAction<F>)
    where
        F: Fn(&I18n, &mut WzllamaState, &HardwareInfo) -> Result<()> + Send + Sync + 'static,
    {
        let _ = action; // Prevent unused warning
    }

    /// Run a menu using the new system, falling back to legacy execution
    #[allow(dead_code)]
    pub fn run_menu(
        &self,
        tree: &MenuTree,
        _i18n: &I18n,
        _state: &mut WzllamaState,
        _hw: &HardwareInfo,
    ) -> Result<()> {
        println!("Menu: {:?}", tree.metadata.title);
        for (path, item) in tree.get_flat_items() {
            if item.has_action() {
                if let Some(ref action_id) = item.action_id {
                    println!("  {} -> {}", path, action_id);
                }
            }
        }
        
        Ok(())
    }
}

/// Example of how to migrate a wizard menu function to menu_api
/// 
/// Before (current style):
/// ```ignore
/// pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
///     loop {
///         // menu logic
///     }
/// }
/// ```
/// 
/// After (new style):
/// ```ignore
/// pub fn run(handler: &mut MenuHandler, i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
///     let tree = MenuTree::new("Tools");
///     handler.set_tree(tree)?;
///     handler.run()?;
///     Ok(())
/// }
/// ```
pub mod migration_example {
    use super::*;

    /// Example menu tree for tools
    #[allow(dead_code)]
    pub fn create_tools_menu() -> MenuTree {
        WizardMenuBuilder::new()
            .add_branch("Installer", vec![
                MenuConfigItem {
                    label: "Ollama".to_string(),
                    action_id: Some("install_ollama".to_string()),
                    children: None,
                    condition: None,
                },
                MenuConfigItem {
                    label: "Open WebUI".to_string(),
                    action_id: Some("install_openwebui".to_string()),
                    children: None,
                    condition: None,
                },
            ])
            .add_branch("Lancer", vec![
                MenuConfigItem {
                    label: "Chat".to_string(),
                    action_id: Some("launch_chat".to_string()),
                    children: None,
                    condition: None,
                },
            ])
            .build("Tools Menu")
    }
}