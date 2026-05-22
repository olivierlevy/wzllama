use anyhow::Result;
use crate::config::I18n;
use crate::config::WzllamaState;
use crate::core::hardware::HardwareInfo;
use crate::menu_api::{MenuTree, MenuItem, MenuHandler, ActionDispatcher, ClosureAction, ActionResult, MenuMetadata};
use crate::tools;
use crate::wizard::{menu_cleanup, menu_config, menu_models, menu_scientific, menu_tools, menu_wizard};
use crate::menu_api::arc_action::ArcActionRunner;

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

    fn run_with_dialoguer(&mut self) -> Result<()> {
        use dialoguer::Select;
        use colored::*;
        use crate::core::{system, ollama_api};
        use crate::display;
        use crate::tools;
        use crate::wizard::{menu_cleanup, menu_config, menu_models, menu_scientific, menu_tools, menu_wizard};
        use std::io::Write;

        loop {
            print!("\x1b[2J\x1b[H");
            std::io::stdout().flush().ok();

            let ram_avail = system::get_available_ram_gb();
            let vram_avail = system::get_available_vram_gb();
            let running = ollama_api::get_running_models();

            let (term_width, term_height) = display::get_terminal_size();
            let compact = term_height < 25 || term_width < 70;

            if compact {
                display::section(&self.i18n.t("menu.main.title"));
                println!("   💾 {:.1}/{:.1} Go | 🎮 {:.1}/{:.1} Go",
                    ram_avail, self.hw.ram_gb,
                    vram_avail.unwrap_or(0.0), self.hw.total_vram_mb as f64 / 1024.0);
            } else {
                display::header(&self.i18n.t("menu.main.title"));
                display::resources_with_bars(self.hw.ram_gb, ram_avail,
                    self.hw.total_vram_mb as f64 / 1024.0, vram_avail, &running, self.state.last_model.as_deref());
            }

            let mut items: Vec<String> = vec![];
            let has_resume = self.state.last_tool.is_some() && self.state.last_model.is_some();

            if has_resume {
                if let (Some(ref last_tool), Some(ref last_model)) = (&self.state.last_tool, &self.state.last_model) {
                    if let Some(tool) = tools::get_tool(last_tool) {
                        items.push(self.i18n.t_with_vars("menu.main.resume", &[("tool", tool.name()), ("model", last_model)]));
                    }
                }
            }

            items.push(self.i18n.t("menu.main.wizard"));
            items.push(self.i18n.t("menu.main.models"));
            items.push(self.i18n.t("menu.main.scientific"));
            items.push(self.i18n.t("menu.main.tools"));
            items.push(self.i18n.t("menu.main.cleanup"));
            items.push(self.i18n.t("menu.main.config"));
            items.push(self.i18n.t("menu.main.language"));
            items.push(self.i18n.t("menu.main.quit"));

            let reserved = if compact { 5 } else { 15 };
            let choice = match Select::new()
                .with_prompt(self.i18n.t("menu.main.choose"))
                .items(&items)
                .default(0)
                .max_length(display::menu_max_items(items.len(), reserved))
                .interact_opt()? {
                Some(c) => c,
                None => break,
            };

            let base_offset = has_resume as usize;
            match choice {
                n if has_resume && n == 0 => {
                    if let (Some(ref last_tool), Some(ref last_model)) = (&self.state.last_tool, &self.state.last_model) {
                        if let Some(tool) = tools::get_tool(last_tool) {
                            tool.launch(self.i18n, self.state, Some(last_model))?;
                        }
                    }
                }
                n if n == base_offset => menu_wizard::run(self.i18n, self.state, self.hw)?,
                n if n == 1 + base_offset => menu_models::run(self.i18n, self.state, self.hw)?,
                n if n == 2 + base_offset => menu_scientific::run(self.i18n, self.state, self.hw)?,
                n if n == 3 + base_offset => menu_tools::run(self.i18n, self.state, self.hw)?,
                n if n == 4 + base_offset => menu_cleanup::run(self.i18n, self.state, self.hw)?,
                n if n == 5 + base_offset => menu_config::run(self.i18n, self.state, self.hw)?,
                n if n == 6 + base_offset => {
                    self.change_language()?;
                    return Ok(());
                }
                n if n == 7 + base_offset => break,
                _ => break,
            }
        }
        Ok(())
    }

    fn change_language(&mut self) -> Result<()> {
        let languages = crate::config::i18n::get_available_languages();
        let mut all_items = vec!["↩️  Retour".to_string()];
        for l in &languages {
            all_items.push(format!("{} ({})", l.name, l.code));
        }

        let sel = match dialoguer::Select::new()
            .with_prompt("🌍 Langue / Language")
            .items(&all_items)
            .default(0)
            .interact_opt()? {
            Some(s) => s,
            None => return Ok(()),
        };

        if sel != 0 {
            let _ = crate::config::i18n::load(&languages[sel - 1].code)?;
            crate::config::state::set_language(&languages[sel - 1].code, self.state);
            return Ok(());
        }
        Ok(())
    }
}