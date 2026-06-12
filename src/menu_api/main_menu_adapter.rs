use anyhow::Result;
use crate::config::I18n;
use crate::config::WzllamaState;
use crate::core::hardware::HardwareInfo;
use crate::menu_api::{MenuTree, MenuItem, MenuHandler, ActionDispatcher, ClosureAction, ActionResult, MenuMetadata};
use crate::tools;

/// Main menu runner that bridges menu_api with existing wizard functions
pub struct MainMenuRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> MainMenuRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    /// Get the menu tree structure (for API/consumers)
    /// Reflects the dynamic structure with resume option if available
    pub fn menu_tree(&self) -> MenuTree {
        let has_resume = self.state.last_tool.is_some() && self.state.last_model.is_some();
        
        let mut root = MenuItem::branch("main");
        
        // Resume option (dynamically shown)
        if has_resume {
            if let (Some(ref last_tool), Some(ref last_model)) = (&self.state.last_tool, &self.state.last_model) {
                if let Some(tool) = tools::get_tool(last_tool) {
                    let label = format!("▶️  Resume {} with {}", tool.name(), last_model);
                    root = root.add_submenu(MenuItem::leaf(&label).with_action("resume_last"));
                }
            }
        }
        
        // Static menu items
        root = root
            .add_submenu(MenuItem::leaf(&self.i18n.t("menu.main.wizard")).with_action("menu_wizard"))
            .add_submenu(MenuItem::leaf(&self.i18n.t("menu.main.models")).with_action("menu_models"))
            .add_submenu(MenuItem::leaf(&self.i18n.t("menu.main.scientific")).with_action("menu_scientific"))
            .add_submenu(MenuItem::leaf(&self.i18n.t("menu.main.tools")).with_action("menu_tools"))
            .add_submenu(MenuItem::leaf(&self.i18n.t("menu.main.cleanup")).with_action("menu_cleanup"))
            .add_submenu(MenuItem::leaf(&self.i18n.t("menu.main.config")).with_action("menu_config"))
            .add_submenu(MenuItem::leaf(&self.i18n.t("menu.main.language")).with_action("menu_language"));
        // Note: No quit item here - MenuHandler adds "✖ Quitter" automatically
        
        MenuTree::new("main")
            .with_metadata(MenuMetadata {
                title: Some(self.i18n.t("menu.main.title")),
                ..Default::default()
            })
            .with_root(root)
    }

    /// Run using MenuHandler (Phase 3 - now default)
    pub fn run(&mut self) -> Result<()> {
        self.run_with_menu_handler()
    }

    /// Run with MenuHandler (Phase 2 integration)
    /// This demonstrates how MenuHandler can be used with wizard actions
    pub fn run_with_menu_handler(&mut self) -> Result<()> {
        use crate::wizard::{menu_cleanup, menu_config, menu_models, menu_scientific, menu_tools, menu_wizard};
        use crate::menu_api::arc_action::ArcActionRunner;
        
        // Create the action runner to capture context
        let action_runner = ArcActionRunner::new(
            self.i18n.clone(),
            self.state.clone(),
            self.hw.clone()
        );
        
        // Build the menu tree
        let tree = self.menu_tree();
        
        // Create dispatcher with actions
        let mut dispatcher = ActionDispatcher::new();
        
        // Register actions for each menu item
        dispatcher.register(Box::new(action_runner.create_action(
            "menu_wizard",
            "Wizard",
            |i18n, state, hw| menu_wizard::run(i18n, state, hw)
        )));
        
        dispatcher.register(Box::new(action_runner.create_action(
            "menu_models",
            "Models",
            |i18n, state, hw| menu_models::run(i18n, state, hw)
        )));
        
        dispatcher.register(Box::new(action_runner.create_action(
            "menu_scientific",
            "Scientific",
            |i18n, state, hw| menu_scientific::run(i18n, state, hw)
        )));
        
        dispatcher.register(Box::new(action_runner.create_action(
            "menu_tools",
            "Tools",
            |i18n, state, hw| menu_tools::run(i18n, state, hw)
        )));
        
        dispatcher.register(Box::new(action_runner.create_action(
            "menu_cleanup",
            "Cleanup",
            |i18n, state, hw| menu_cleanup::run(i18n, state, hw)
        )));
        
        dispatcher.register(Box::new(action_runner.create_action(
            "menu_config",
            "Config",
            |i18n, state, hw| menu_config::run(i18n, state, hw)
        )));
        
        // Resume action (if available)
        if let (Some(ref last_tool), Some(ref last_model)) = (&self.state.last_tool, &self.state.last_model) {
            if let Some(tool) = tools::get_tool(last_tool) {
                let tool_name = last_tool.clone();
                let model_name = last_model.clone();
                let i18n_clone = self.i18n.clone();
                let state_clone = self.state.clone();
                dispatcher.register(Box::new(ClosureAction::new(
                    "resume_last",
                    &format!("Resume {} with {}", tool.name(), last_model),
                    move |_| {
                        // Execute resume logic
                        if let Some(t) = crate::tools::get_tool(&tool_name) {
                            let _ = t.launch(&i18n_clone, &state_clone, Some(&model_name));
                        }
                        Ok(ActionResult::success_with("Resumed"))
                    }
                )));
            }
        }
        
        // Run with MenuHandler (adds "✖ Quitter" automatically)
        let mut handler = MenuHandler::new(tree, dispatcher, self.i18n, self.state, self.hw);
        handler.run()
    }
}