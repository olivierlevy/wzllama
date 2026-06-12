//! Wizard menu implementation - migrated from wizard::menu_wizard
//!
//! This file contains the core wizard logic that was in src/wizard/menu_wizard.rs

use crate::config::state;
use crate::config::{I18n, WzllamaState};
use crate::core::{llmfit_api, localmax_models, ollama_api, HardwareInfo};
use crate::display;
use crate::menu_api::wizard_helpers::get_priority_tools_for_usecase;
use crate::menu_api::wizard_helpers::UseCase;
use crate::tools::{get_tool, tool_trait::ToolStatus};
use anyhow::Result;
use dialoguer::Select;
use std::collections::HashSet;

/// Wizard menu functions - migrated from wizard::menu_wizard
pub struct WizardMenuRunner;

impl WizardMenuRunner {
    /// Run the complete wizard workflow
    pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
        Self::models_wizard(i18n, state, hw)
    }

    /// Models wizard - choose use case then select model
    fn models_wizard(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
        loop {
            let use_cases = UseCase::all();
            let display_names: Vec<String> =
                use_cases.iter().map(|uc| uc.display_name(i18n)).collect();

            let back_option = i18n.t("menu.back");
            let mut all_items = vec![back_option];
            all_items.extend(display_names);

            let sel = Select::new()
                .with_prompt(i18n.t("wizard.usecase.choose"))
                .items(&all_items)
                .default(0)
                .interact_opt()?;

            match sel {
                Some(0) => return Ok(()),
                Some(s) if s <= use_cases.len() => {
                    if Self::handle_usecase_selection(i18n, state, hw, use_cases[s - 1])? {
                        return Ok(());
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    /// Handle use case selection - shows models and allows download selection
    fn handle_usecase_selection(
        i18n: &I18n,
        state: &mut WzllamaState,
        hw: &HardwareInfo,
        use_case: UseCase,
    ) -> Result<bool> {
        let local_models = ollama_api::get_models();
        let local_names: HashSet<String> = local_models.iter().map(|m| m.name.clone()).collect();
        let api_models = Self::get_models_from_llmfit(use_case);

        let api_ollama_models: Vec<ollama_api::OllamaModel> = if api_models.is_empty() {
            let search_query = match use_case {
                UseCase::General => "general",
                UseCase::Coding => "code",
                UseCase::Reasoning => "reasoning",
                UseCase::Chat => "chat",
                UseCase::Multimodal => "multimodal",
                UseCase::Embedding => "embedding",
            };

            localmax_models::fetch_models_by_search(search_query, 50)
                .unwrap_or_default()
                .into_iter()
                .map(|m| m.to_ollama_model())
                .collect()
        } else {
            api_models
                .into_iter()
                .map(|m| ollama_api::OllamaModel {
                    name: m.name.clone(),
                    model: m.name.clone(),
                    modified_at: None,
                    size: Some((m.memory_required_gb * 1024.0 * 1024.0 * 1024.0) as u64),
                    details: None,
                })
                .collect()
        };

        let model_names: Vec<String> = local_models.iter().map(|m| m.name.clone()).collect();
        let available: Vec<_> = api_ollama_models
            .iter()
            .filter(|m| !local_names.contains(&m.name))
            .collect();

        let mut all_model_choices: Vec<String> = model_names.clone();
        for model in &available {
            all_model_choices.push(format!("📥 {} (download)", model.name));
        }

        if let Some(ref model) = state.last_model {
            all_model_choices
                .push(i18n.t_with_vars("wizard.action.launch_with_current", &[("model", model)]));
        } else {
            all_model_choices.push(i18n.t("wizard.action.launch_with_current_no_model"));
        }

        all_model_choices.insert(0, i18n.t("menu.back"));

        let sel = Select::new()
            .with_prompt(i18n.t("wizard.usecase.choose_model"))
            .items(&all_model_choices)
            .default(0)
            .interact_opt()?;

        match sel {
            Some(0) => Ok(false),
            Some(s) if s <= model_names.len() => {
                let selected_model = &local_models[s - 1].name;
                state.last_model = Some(selected_model.clone());
                state::save(state)?;
                display::success(
                    &i18n.t_with_vars("wizard.model_selected", &[("model", selected_model)]),
                );
                Self::launch_tool_for_usecase(i18n, state, hw, use_case, selected_model)?;
                Ok(true)
            }
            Some(s) if s <= model_names.len() + available.len() => {
                let idx = s - model_names.len() - 1;
                let chosen_model = &available[idx];
                display::info(&format!(
                    "{}...",
                    i18n.t_with_vars("wizard.downloading", &[("model", &chosen_model.name)])
                ));
                if let Err(e) = ollama_api::pull_model(&chosen_model.name) {
                    display::warning(&format!(
                        "{}: {}",
                        i18n.t("models.localmaxxing_download_error"),
                        e
                    ));
                } else {
                    state.last_model = Some(chosen_model.name.clone());
                    state::save(state)?;
                    display::success(&i18n.t_with_vars(
                        "models.downloaded_success",
                        &[("model", &chosen_model.name)],
                    ));
                    Self::launch_tool_for_usecase(i18n, state, hw, use_case, &chosen_model.name)?;
                    return Ok(true);
                }
                Ok(false)
            }
            Some(s) if s == model_names.len() + available.len() + 1 => {
                if let Some(model) = state.last_model.clone() {
                    Self::launch_tool_for_usecase(i18n, state, hw, use_case, &model)?;
                } else {
                    display::warning(&i18n.t("tool.ollama.choose_model"));
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Get models from llmfit API filtered by use case
    fn get_models_from_llmfit(use_case: UseCase) -> Vec<llmfit_api::LLMFitModel> {
        let client = llmfit_api::LLMFitClient::new();
        if !client.is_running() {
            return vec![];
        }

        client
            .get_top_models(Some(20), None, Some(use_case.as_str()))
            .unwrap_or_default()
    }

    /// Launch tool for a given use case with selected model
    pub fn launch_tool_for_usecase(
        i18n: &I18n,
        state: &mut WzllamaState,
        _hw: &HardwareInfo,
        use_case: UseCase,
        model: &str,
    ) -> Result<()> {
        let tools = get_priority_tools_for_usecase(use_case, state);

        let installed_tools: Vec<String> = tools
            .into_iter()
            .filter(|tool_id| {
                let tool = get_tool(tool_id);
                tool.as_ref()
                    .map(|t| t.status(state) == ToolStatus::Installed)
                    .unwrap_or(false)
            })
            .collect();

        if installed_tools.is_empty() {
            display::warning(&i18n.t("wizard.no_tools_installed"));
            return Ok(());
        }

        if installed_tools.len() == 1 {
            let tool_id = installed_tools[0].clone();
            state.last_tool = Some(tool_id.clone());
            let tool = get_tool(&tool_id).unwrap();
            tool.launch(i18n, state, Some(model))?;
            return Ok(());
        }

        let tool_displays: Vec<String> = installed_tools
            .iter()
            .filter_map(|tool_id| {
                get_tool(tool_id).map(|t| format!("🔧 {} - {}", t.name(), t.description(i18n)))
            })
            .collect();

        let mut items: Vec<String> = vec![i18n.t("menu.back")];
        items.extend(tool_displays);

        let sel = Select::new()
            .with_prompt(i18n.t("wizard.select_tool"))
            .items(&items)
            .default(0)
            .interact_opt()?;

        match sel {
            Some(0) => {}
            Some(s) if s <= installed_tools.len() => {
                let tool_id = installed_tools[s - 1].clone();
                state.last_tool = Some(tool_id.clone());
                state::save(state)?;
                let tool = get_tool(&tool_id).unwrap();
                tool.launch(i18n, state, Some(model))?;
            }
            _ => {}
        }

        Ok(())
    }
}
