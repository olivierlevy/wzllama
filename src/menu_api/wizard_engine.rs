//! Wizard engine - manages complex multi-step wizard workflows
//!
//! This replaces the wizard::menu_wizard implementation with
//! a menu_api-based approach.

use crate::config::{I18n, WzllamaState};
use crate::core::{llmfit_api, localmax_models, ollama_api, HardwareInfo};
use crate::display;
use crate::menu_api::wizard_helpers::{get_priority_tools_for_usecase, UseCase};
use crate::menu_api::{
    enter_alternate_screen, exit_alternate_screen, MenuItem, MenuMetadata, MenuTree,
};
use crate::tools::{self, tool_trait::ToolStatus};
use crate::wizard::menu_header;
use anyhow::Result;
use dialoguer::Select;
use std::collections::HashSet;

/// Wizard engine that drives multi-step workflows
pub struct WizardEngine<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> WizardEngine<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    /// Run the complete wizard workflow (migrated from wizard::menu_wizard::run)
    pub fn run(&mut self) -> Result<()> {
        enter_alternate_screen();

        loop {
            // Affiche le header
            menu_header::render(
                self.i18n,
                "wizard.title",
                true,
                self.state.last_model.as_deref(),
                self.hw.ram_gb,
                self.hw.total_vram_mb as f64 / 1024.0,
            );

            let use_cases = UseCase::all();
            let display_names: Vec<String> = use_cases
                .iter()
                .map(|uc| uc.display_name(self.i18n))
                .collect();

            // Retour en premier item
            let back_option = self.i18n.t("menu.back");
            let mut all_items = vec![back_option];
            all_items.extend(display_names);

            let sel = Select::new()
                .with_prompt(self.i18n.t("wizard.usecase.choose"))
                .items(&all_items)
                .default(0)
                .interact_opt()?;

            match sel {
                Some(0) => break, // Retour
                Some(s) if s <= use_cases.len() => {
                    if self.handle_usecase_selection(use_cases[s - 1])? {
                        exit_alternate_screen();
                        return Ok(());
                    }
                }
                _ => break,
            }
        }

        exit_alternate_screen();
        Ok(())
    }

    /// Handle use case selection - shows models and allows download
    fn handle_usecase_selection(&mut self, use_case: UseCase) -> Result<bool> {
        // Get local models
        let local_models = ollama_api::get_models();
        let local_names: HashSet<String> = local_models.iter().map(|m| m.name.clone()).collect();

        // Get API models
        let api_models = self.get_models_from_llmfit(use_case);

        // Build model choices
        let model_names: Vec<String> = local_models.iter().map(|m| m.name.clone()).collect();
        let available: Vec<_> = api_models
            .iter()
            .filter(|m| !local_names.contains(&m.name))
            .collect();

        let mut all_model_choices: Vec<String> = model_names.clone();
        for model in &available {
            all_model_choices.push(format!("📥 {} (download)", model.name));
        }

        // Add launch option
        if let Some(ref model) = self.state.last_model {
            all_model_choices.push(
                self.i18n
                    .t_with_vars("wizard.action.launch_with_current", &[("model", model)]),
            );
        } else {
            all_model_choices.push(self.i18n.t("wizard.action.launch_with_current_no_model"));
        }

        // Retour en premier item
        all_model_choices.insert(0, self.i18n.t("menu.back"));

        let sel = Select::new()
            .with_prompt(self.i18n.t("wizard.usecase.choose_model"))
            .items(&all_model_choices)
            .default(0)
            .interact_opt()?;

        match sel {
            Some(0) => Ok(false), // Retour
            Some(s) if s <= model_names.len() => {
                let selected_model = &local_models[s - 1].name;
                self.state.last_model = Some(selected_model.clone());
                crate::config::state::save(self.state)?;
                display::success(
                    &self
                        .i18n
                        .t_with_vars("wizard.model_selected", &[("model", selected_model)]),
                );
                self.launch_tool_for_usecase(&use_case, selected_model)?;
                Ok(true)
            }
            Some(s) if s <= model_names.len() + available.len() => {
                let idx = s - model_names.len() - 1;
                let chosen_model = &available[idx];
                display::info(&format!(
                    "{}...",
                    self.i18n
                        .t_with_vars("wizard.downloading", &[("model", &chosen_model.name)])
                ));
                if let Err(e) = ollama_api::pull_model(&chosen_model.name) {
                    display::warning(&format!(
                        "{}: {}",
                        self.i18n.t("models.localmaxxing_download_error"),
                        e
                    ));
                } else {
                    self.state.last_model = Some(chosen_model.name.clone());
                    crate::config::state::save(self.state)?;
                    display::success(&self.i18n.t_with_vars(
                        "models.downloaded_success",
                        &[("model", &chosen_model.name)],
                    ));
                    self.launch_tool_for_usecase(&use_case, &chosen_model.name)?;
                    return Ok(true);
                }
                Ok(false)
            }
            Some(s) if s == model_names.len() + available.len() + 1 => {
                // Launch with current model
                if let Some(ref model) = self.state.last_model {
                    let model_name = model.clone();
                    self.launch_tool_for_usecase(&use_case, &model_name)?;
                } else {
                    display::warning(&self.i18n.t("tool.ollama.choose_model"));
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Get models from llmfit API (fallback to localmaxxing)
    fn get_models_from_llmfit(&self, use_case: UseCase) -> Vec<llmfit_api::LLMFitModel> {
        let client = llmfit_api::LLMFitClient::new();
        if !client.is_running() {
            return vec![];
        }

        match client.get_top_models(Some(20), None, Some(use_case.as_str())) {
            Ok(models) => models,
            Err(_) => vec![],
        }
    }

    /// Launch tool for a given use case with selected model
    fn launch_tool_for_usecase(&mut self, use_case: &UseCase, model: &str) -> Result<()> {
        let tools = get_priority_tools_for_usecase(*use_case, self.state);

        // Filter to only installed tools
        let installed_tools: Vec<String> = tools
            .into_iter()
            .filter(|tool_id| {
                let tool = tools::get_tool(tool_id);
                tool.as_ref()
                    .map(|t| t.status(self.state) == ToolStatus::Installed)
                    .unwrap_or(false)
            })
            .collect();

        if installed_tools.is_empty() {
            display::warning(&self.i18n.t("wizard.no_tools_installed"));
            return Ok(());
        }

        if installed_tools.len() == 1 {
            display::info(&self.i18n.t("wizard.one_tool_installed"));
            let tool_id = installed_tools[0].clone();
            self.state.last_tool = Some(tool_id.clone());
            let tool = tools::get_tool(&tool_id).unwrap();
            tool.launch(self.i18n, self.state, Some(model))?;
            return Ok(());
        }

        // Show tool selection menu
        let tool_displays: Vec<String> = installed_tools
            .iter()
            .filter_map(|tool_id| {
                tools::get_tool(tool_id)
                    .map(|t| format!("🔧 {} - {}", t.name(), t.description(self.i18n)))
            })
            .collect();

        let mut items: Vec<String> = vec![self.i18n.t("menu.back")];
        items.extend(tool_displays);

        let sel = Select::new()
            .with_prompt(self.i18n.t("wizard.select_tool"))
            .items(&items)
            .default(0)
            .interact_opt()?;

        match sel {
            Some(0) => {} // Retour
            Some(s) if s <= installed_tools.len() => {
                let tool_id = installed_tools[s - 1].clone();
                self.state.last_tool = Some(tool_id.clone());
                crate::config::state::save(self.state)?;
                let tool = tools::get_tool(&tool_id).unwrap();
                tool.launch(self.i18n, self.state, Some(model))?;
            }
            _ => {}
        }

        Ok(())
    }
}

/// Build the wizard menu tree (public for menu_api use)
pub fn build_menu_tree(i18n: &I18n) -> MenuTree {
    let use_cases = UseCase::all();
    let mut items = vec![MenuItem::leaf("↩️ Retour")];
    items.extend(use_cases.iter().map(|uc| {
        MenuItem::leaf(&uc.display_name(i18n))
            .with_action(&format!("wizard_usecase_{}", uc.as_str()))
    }));

    MenuTree::new("wizard")
        .with_metadata(MenuMetadata {
            title: Some("🤖 Wizard".to_string()),
            ..Default::default()
        })
        .with_root(MenuItem::branch("wizard").add_submenus(items))
}

/// Wizard engine runner wrapper
pub struct WizardEngineRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> WizardEngineRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut engine = WizardEngine::new(self.i18n, self.state, self.hw);
        engine.run()
    }
}
