//! Wizard menu for selecting what to do with models
//! Integrates with llmfit API for use-case based model recommendations

use anyhow::Result;
use dialoguer::Select;
use std::collections::HashSet;
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, llmfit_api, localmax_models};
use crate::display;
use crate::tools::{self, tool_trait::ToolStatus};
use crate::menu_api::{MenuTree, MenuItem};
use super::menu_header;

/// Create the wizard menu tree structure
pub fn build_menu_tree() -> MenuTree {
    let root = MenuItem::branch("wizard")
        .add_submenu(MenuItem::leaf("↩️ Retour"))
        .add_submenu(MenuItem::leaf("📋 Général").with_action("usecase_general"))
        .add_submenu(MenuItem::leaf("💻 Coding").with_action("usecase_coding"))
        .add_submenu(MenuItem::leaf("🧠 Reasoning").with_action("usecase_reasoning"))
        .add_submenu(MenuItem::leaf("💬 Chat").with_action("usecase_chat"))
        .add_submenu(MenuItem::leaf("🎨 Multimodal").with_action("usecase_multimodal"))
        .add_submenu(MenuItem::leaf("🔢 Embedding").with_action("usecase_embedding"));
    
    MenuTree::new("wizard").with_root(root)
}

/// Use cases for model filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseCase {
    General,
    Coding,
    Reasoning,
    Chat,
    Multimodal,
    Embedding,
}

impl UseCase {
    pub fn all() -> Vec<Self> {
        vec![
            UseCase::General,
            UseCase::Coding,
            UseCase::Reasoning,
            UseCase::Chat,
            UseCase::Multimodal,
            UseCase::Embedding,
        ]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            UseCase::General => "general",
            UseCase::Coding => "coding",
            UseCase::Reasoning => "reasoning",
            UseCase::Chat => "chat",
            UseCase::Multimodal => "multimodal",
            UseCase::Embedding => "embedding",
        }
    }
    
    pub fn display_name(&self, i18n: &I18n) -> String {
        match self {
            UseCase::General => i18n.t("wizard.usecase.general"),
            UseCase::Coding => i18n.t("wizard.usecase.coding"),
            UseCase::Reasoning => i18n.t("wizard.usecase.reasoning"),
            UseCase::Chat => i18n.t("wizard.usecase.chat"),
            UseCase::Multimodal => i18n.t("wizard.usecase.multimodal"),
            UseCase::Embedding => i18n.t("wizard.usecase.embedding"),
        }
    }
}

/// Get priority tools for a use case (installed ones ranked by relevance)
pub fn get_priority_tools_for_usecase(use_case: UseCase, state: &WzllamaState) -> Vec<String> {
    let mut tool_ids = vec![];
    
    match use_case {
        UseCase::Coding => {
            if state.installed.claude_code { tool_ids.push("claude_code".to_string()); }
            if state.installed.opencode { tool_ids.push("opencode".to_string()); }
            if state.installed.droid { tool_ids.push("droid".to_string()); }
            if state.installed.codex { tool_ids.push("codex".to_string()); }
        }
        UseCase::Reasoning => {
            if state.installed.openclaw { tool_ids.push("openclaw".to_string()); }
            if state.installed.hermes_agent { tool_ids.push("hermes_agent".to_string()); }
        }
        UseCase::Chat => {
            if state.installed.goose { tool_ids.push("goose".to_string()); }
            if state.installed.pool { tool_ids.push("pool".to_string()); }
            if state.installed.pi { tool_ids.push("pi".to_string()); }
        }
        UseCase::Multimodal => {
            if state.installed.openclaw { tool_ids.push("openclaw".to_string()); }
            if state.installed.goose { tool_ids.push("goose".to_string()); }
        }
        UseCase::General | UseCase::Embedding => {
            if state.installed.openclaw { tool_ids.push("openclaw".to_string()); }
            if state.installed.goose { tool_ids.push("goose".to_string()); }
        }
    }
    
    // Add ollama as last resort (always available)
    tool_ids.push("ollama".to_string());
    
    tool_ids
}

/// Main wizard menu - goes directly to models use case selection
pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    models_wizard(i18n, state, hw)
}

/// Models wizard - choose use case then select model
fn models_wizard(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    loop {
        // Affiche le header avec ressources comme le menu principal
        menu_header::render(
            i18n,
            "wizard.title",
            true,
            state.last_model.as_deref(),
            hw.ram_gb,
            hw.total_vram_mb as f64 / 1024.0
        );
        
        let use_cases = UseCase::all();
        let display_names: Vec<String> = use_cases.iter()
            .map(|uc| uc.display_name(i18n))
            .collect();
        
        // Retour en premier item (selon TODO.md ligne 72)
        let back_option = i18n.t("menu.back");
        let mut all_items = vec![back_option];
        all_items.extend(display_names);
        
        let sel = Select::new()
            .with_prompt(i18n.t("wizard.usecase.choose"))
            .items(&all_items)
            .default(0)
            .interact_opt()?;
        
        match sel {
            Some(0) => return Ok(()),  // Retour en position 0
            Some(s) if s <= use_cases.len() => {
                if handle_usecase_selection(i18n, state, hw, use_cases[s - 1])? {  // -1 car Retour est en position 0
                    return Ok(()); // User selected a model and set it as default
                }
                // Otherwise continue the loop to show use case menu again
            }
            _ => return Ok(()),
        }
    }
}

/// Handle use case selection - shows models and allows download selection
pub fn handle_usecase_selection(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo, use_case: UseCase) -> Result<bool> {
    // Get local models first
    let local_models = ollama_api::get_models();
    let local_names: HashSet<String> = local_models.iter().map(|m| m.name.clone()).collect();
    
    // Try llmfit API first, fall back to localmaxxing
    let api_models = get_models_from_llmfit(use_case);
    
    // Convert to OllamaModel format for consistent display
    let api_ollama_models: Vec<ollama_api::OllamaModel> = if api_models.is_empty() {
        // Fallback to localmaxxing
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
        api_models.into_iter().map(|m| ollama_api::OllamaModel {
            name: m.name.clone(),
            model: m.name.clone(),
            modified_at: None,
            size: Some((m.memory_required_gb * 1024.0 * 1024.0 * 1024.0) as u64),
            details: None,
        }).collect()
    };
    
    // Allow selecting an installed model or downloading a new one
    let model_names: Vec<String> = local_models.iter().map(|m| m.name.clone()).collect();
    
    // Add available models for download
    let available: Vec<_> = api_ollama_models.iter()
        .filter(|m| !local_names.contains(&m.name))
        .collect();
    
    let mut all_model_choices: Vec<String> = model_names.clone();
    for model in &available {
        all_model_choices.push(format!("📥 {} (download)", model.name));
    }
    
    // Add action options
    if let Some(ref model) = state.last_model {
        all_model_choices.push(i18n.t_with_vars("wizard.action.launch_with_current", &[("model", model)]));
    } else {
        all_model_choices.push(i18n.t("wizard.action.launch_with_current_no_model"));
    }
    // Retour en premier item (selon TODO.md ligne 72) - on l'ajoute au début
    all_model_choices.insert(0, i18n.t("menu.back"));
    
    let sel = Select::new()
        .with_prompt(i18n.t("wizard.usecase.choose_model"))
        .items(&all_model_choices)
        .default(0)
        .interact_opt()?;
    
    match sel {
        Some(0) => Ok(false),  // Retour en position 0
        Some(s) if s <= model_names.len() => {
            // User selected an installed model - set as default
            let selected_model = &local_models[s - 1].name;  // -1 car Retour est en position 0
            state.last_model = Some(selected_model.clone());
            crate::config::state::save(state)?;
            display::success(&i18n.t_with_vars("wizard.model_selected", &[("model", selected_model)]));
            
            // Now show tools to launch with this model
            launch_tool_for_usecase(i18n, state, hw, use_case, selected_model)?;
            Ok(true)
        }
        Some(s) if s <= model_names.len() + available.len() => {
            // User selected a model to download (index +1 car Retour en position 0)
            let idx = s - model_names.len() - 1;
            let chosen_model = &available[idx];
            display::info(&format!("{}...", i18n.t_with_vars("wizard.downloading", &[("model", &chosen_model.name)])));
            if let Err(e) = ollama_api::pull_model(&chosen_model.name) {
                display::warning(&format!("{}: {}", i18n.t("models.localmaxxing_download_error"), e));
            } else {
                state.last_model = Some(chosen_model.name.clone());
                crate::config::state::save(state)?;
                display::success(&i18n.t_with_vars("models.downloaded_success", &[("model", &chosen_model.name)]));
                launch_tool_for_usecase(i18n, state, hw, use_case, &chosen_model.name)?;
                return Ok(true);
            }
            Ok(false)
        }
        Some(s) if s == model_names.len() + available.len() + 1 => {
            // Select tool for current use case (no model change)
            if let Some(ref model) = state.last_model {
                let model_name = model.clone();
                launch_tool_for_usecase(i18n, state, hw, use_case, &model_name)?;
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
    
    client.get_top_models(
        Some(20),
        None,
        Some(use_case.as_str())
    ).unwrap_or_default()
}

/// Launch tool for a given use case with selected model
pub fn launch_tool_for_usecase(i18n: &I18n, state: &mut WzllamaState, _hw: &HardwareInfo, use_case: UseCase, model: &str) -> Result<()> {
    let tools = get_priority_tools_for_usecase(use_case, state);
    
    // Filter to only installed tools
    let installed_tools: Vec<String> = tools.into_iter()
        .filter(|tool_id| {
            let tool = tools::get_tool(tool_id);
            tool.as_ref().map(|t| t.status(state) == ToolStatus::Installed).unwrap_or(false)
        })
        .collect();
    
    if installed_tools.is_empty() {
        display::warning(&i18n.t("wizard.no_tools_installed"));
        return Ok(());
    }
    
    if installed_tools.len() == 1 {
        // Only one tool available, launch it directly
        display::info(&i18n.t("wizard.one_tool_installed"));
        let tool_id = installed_tools[0].clone();
        state.last_tool = Some(tool_id.clone());
        let tool = tools::get_tool(&tool_id).unwrap();
        tool.launch(i18n, state, Some(model))?;
        return Ok(());
    }
    
    // Show tool selection menu
    let tool_displays: Vec<String> = installed_tools.iter()
        .filter_map(|tool_id| {
            tools::get_tool(tool_id).map(|t| {
                format!("🔧 {} - {}", t.name(), t.description(i18n))
            })
        })
        .collect();
    
    // Retour en premier item (selon TODO.md ligne 72)
    let mut items: Vec<String> = vec![i18n.t("menu.back")];
    items.extend(tool_displays);
    
    let sel = Select::new()
        .with_prompt(i18n.t("wizard.select_tool"))
        .items(&items)
        .default(0)
        .interact_opt()?;
    
    match sel {
        Some(0) => {}  // Retour en position 0
        Some(s) if s <= installed_tools.len() => {
            let tool_id = installed_tools[s - 1].clone();  // -1 car Retour est en position 0
            state.last_tool = Some(tool_id.clone());
            crate::config::state::save(state)?;
            let tool = tools::get_tool(&tool_id).unwrap();
            tool.launch(i18n, state, Some(model))?;
        }
        _ => {}
    }
    
    Ok(())
}