//! Menu handler - interactive menu navigation and execution with dynamic submenu support

use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::menu_item::MenuItem;
use crate::menu_api::menu_tree::MenuTree;
use crate::menu_api::tool_action::{ActionContext, ActionDispatcher, ActionResult};
use anyhow::Result;
use dialoguer::{Confirm, Input, Select};
use log::{error, warn};

/// Navigation state tracking
#[derive(Debug, Clone)]
pub struct NavigationState {
    /// Stack of menu positions for back navigation
    pub history: Vec<usize>,
    /// Current menu position
    pub current_index: usize,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            history: vec![0],
            current_index: 0,
        }
    }
}

/// Menu handler for interactive navigation with dynamic submenu support
pub struct MenuHandler<'a> {
    tree: MenuTree,
    dispatcher: ActionDispatcher,
    state: NavigationState,
    current_menu: MenuItem,
    /// Reference to i18n for dynamic content
    i18n: &'a I18n,
    /// Reference to app state for dynamic content
    app_state: &'a mut WzllamaState,
    /// Reference to hardware info
    hw: &'a HardwareInfo,
}

impl<'a> MenuHandler<'a> {
    /// Create a new menu handler
    pub fn new(
        tree: MenuTree,
        dispatcher: ActionDispatcher,
        i18n: &'a I18n,
        state: &'a mut WzllamaState,
        hw: &'a HardwareInfo,
    ) -> Self {
        let current_menu = tree.root.clone();
        Self {
            tree,
            dispatcher,
            state: NavigationState::default(),
            current_menu,
            i18n,
            app_state: state,
            hw,
        }
    }

    /// Run the interactive menu loop
    pub fn run(&mut self) -> Result<()> {
        use crate::wizard::menu_header;

        loop {
            // Build dynamic submenu if needed
            self.build_dynamic_submenus();

            // Display header with resources (like original wizard menu)
            menu_header::render(
                self.i18n,
                "menu.main.title",
                true,
                self.app_state.last_model.as_deref(),
                self.hw.ram_gb,
                self.hw.total_vram_mb as f64 / 1024.0,
            );

            let items: Vec<String> = self
                .current_menu
                .submenus
                .iter()
                .map(|i| i.label.clone())
                .collect();

            // Check if "Retour" is already in position 0 (wizard pattern)
            let has_back_in_items = !items.is_empty()
                && (items[0].to_lowercase().contains("retour") || items[0].contains("↩️"));

            // Menu handler adds "Retour" and "Quitter" at the end if not already present
            let has_back = self.state.history.len() > 1 && !has_back_in_items;
            let mut all_items = items.clone();
            if has_back {
                all_items.push("↩️ Retour".to_string());
            }
            all_items.push("✖ Quitter".to_string());

            let selection = Select::new()
                .with_prompt(self.get_prompt())
                .items(&all_items)
                .default(self.state.current_index.min(all_items.len() - 1))
                .interact_opt()?;

            let selection = match selection {
                Some(s) => s,
                None => break, // Escape pressed
            };

            // Handle quit
            if selection == all_items.len() - 1 {
                break;
            }

            // Handle back (only if added by menu handler, not in items)
            let back_index = if has_back { items.len() } else { usize::MAX };
            if selection == back_index {
                self.navigate_back();
                continue;
            }

            // Navigate submenus or execute action
            self.navigate_to(selection, has_back_in_items)?;
        }

        Ok(())
    }

    /// Build dynamic submenus for items that have dynamic generators
    fn build_dynamic_submenus(&mut self) {
        // Dynamic submenu support - items can have dynamic generators
        // This is handled via the dynamic_generators module
    }

    /// Get the current menu items (cloned)
    pub fn current_items(&self) -> Vec<MenuItem> {
        self.current_menu.submenus.clone()
    }

    /// Navigate to a submenu or execute action
    fn navigate_to(&mut self, index: usize, has_back_in_items: bool) -> Result<()> {
        // Handle "Retour" in position 0 - navigate back to parent
        if has_back_in_items && index == 0 {
            if self.state.history.len() > 1 {
                self.navigate_back();
            }
            return Ok(());
        }

        let adjusted_index = if has_back_in_items {
            index - 1 // Adjust for "Retour" in position 0
        } else {
            index
        };

        if adjusted_index >= self.current_menu.submenus.len() {
            return Ok(());
        }

        let selected = self.current_menu.submenus[adjusted_index].clone();

        if selected.submenus.is_empty() {
            // Leaf node - execute action if present
            if let Some(ref action_id) = selected.action_id {
                self.execute_action(action_id)?;
            }
        } else {
            // Branch - navigate into submenu
            self.state.history.push(adjusted_index);
            self.state.current_index = 0;
            self.current_menu = selected;
        }

        Ok(())
    }

    /// Navigate back to parent menu
    fn navigate_back(&mut self) {
        if self.state.history.len() > 1 {
            self.state.history.pop();
            self.state.current_index = *self.state.history.last().unwrap_or(&0);

            // Rebuild current menu from root
            self.rebuild_current_menu();
        }
    }

    /// Rebuild current menu from navigation history
    fn rebuild_current_menu(&mut self) {
        let mut current = self.tree.root.clone();

        for &idx in &self.state.history[1..] {
            if idx < current.submenus.len() {
                current = current.submenus[idx].clone();
            }
        }

        self.current_menu = current;
    }

    /// Execute an action by ID
    fn execute_action(&self, action_id: &str) -> Result<()> {
        let ctx = ActionContext::new();

        match self.dispatcher.execute(action_id, &ctx) {
            Ok(result) => {
                if result.success {
                    if let Some(msg) = result.message {
                        println!("✓ {}", msg);
                    }
                } else if let Some(msg) = result.message {
                    warn!("{}", msg);
                }
            }
            Err(e) => {
                error!("Action '{}' failed: {}", action_id, e);
            }
        }

        Ok(())
    }

    /// Get the current prompt text
    fn get_prompt(&self) -> String {
        self.tree
            .metadata
            .title
            .clone()
            .unwrap_or_else(|| "Menu".to_string())
    }

    /// Register an action with the handler
    pub fn register_action(&mut self, action: Box<dyn crate::menu_api::tool_action::ToolAction>) {
        self.dispatcher.register(action);
    }

    /// Get access to the dispatcher for testing/inspection
    pub fn dispatcher(&self) -> &ActionDispatcher {
        &self.dispatcher
    }

    /// Get the tree root for inspection
    pub fn root(&self) -> &MenuItem {
        &self.current_menu
    }
}

/// Non-interactive menu runner (returns selection without dialog)
pub struct MenuRunner {
    tree: MenuTree,
    dispatcher: ActionDispatcher,
}

impl MenuRunner {
    pub fn new(tree: MenuTree, dispatcher: ActionDispatcher) -> Self {
        Self { tree, dispatcher }
    }

    /// Get menu structure without interaction (for TUI rendering)
    pub fn get_structure(&self) -> MenuItem {
        self.tree.root.clone()
    }

    /// Execute action by path
    pub fn execute_path(&self, path: &str, ctx: &ActionContext) -> Result<ActionResult> {
        let item = self
            .tree
            .find_by_path(path)
            .ok_or_else(|| anyhow::anyhow!("Menu path '{}' not found", path))?;

        if let Some(ref action_id) = item.action_id {
            self.dispatcher.execute(action_id, ctx)
        } else {
            Ok(ActionResult::success())
        }
    }
}
