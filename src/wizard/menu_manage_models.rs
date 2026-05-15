use anyhow::Result;
use dialoguer::{Select, Confirm};
use crate::config::{I18n, WzllamaState};
use crate::core::{hardware::HardwareInfo, ollama_api, ollama_models};
use crate::display;

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    loop {
        let models = ollama_api::get_models();
        
        if models.is_empty() {
            display::warning(&i18n.t("models.manage_empty"));
            return Ok(());
        }

        // Afficher les modèles avec leurs informations
        display::section(&i18n.t("models.manage_title"));
        
        let mut items: Vec<String> = models.iter().map(|m| {
            let installed = true; // All models from get_models() are installed
            display::format_model(&m.name, m.size.unwrap_or(0), 1.0, installed)
        }).collect();
        
        items.push(i18n.t("menu.back"));
        
        let sel = match Select::new()
            .with_prompt(&i18n.t("models.manage_select"))
            .items(&items)
            .default(0)
            .interact_opt()?
        {
            Some(s) => s,
            None => return Ok(()),
        };
        
        if sel == models.len() {
            return Ok(());
        }
        
        let selected_model = &models[sel];
        
        // Sous-menu d'actions pour le modèle sélectionné
        let mut actions = vec![
            i18n.t_with_vars("models.manage_selected", &[("model", &selected_model.name)]),
            i18n.t("models.manage_show_info"),
            i18n.t("models.manage_set_default"),
            i18n.t("models.manage_delete"),
        ];
        
        actions.push(i18n.t("menu.back"));
        
        let action_sel = match Select::new()
            .with_prompt(&i18n.t("models.manage_action"))
            .items(&actions)
            .default(0)
            .interact_opt()?
        {
            Some(s) => s,
            None => continue,
        };
        
        match action_sel {
            0 => {
                // Already selected, just show info
                show_model_info(i18n, selected_model, hw);
            }
            1 => {
                // Show model info
                show_model_info(i18n, selected_model, hw);
            }
            2 => {
                // Set as default model
                state.set_last_model(&selected_model.name);
                display::success(&i18n.t_with_vars("models.manage_selected", &[("model", &selected_model.name)]));
            }
            3 => {
                // Delete model
                if Confirm::new()
                    .with_prompt(&i18n.t_with_vars("models.manage_delete_confirm", &[("model", &selected_model.name)]))
                    .default(false)
                    .interact()?
                {
                    match ollama_api::delete_model(&selected_model.name) {
                        Ok(()) => display::success(&i18n.t("models.manage_deleted")),
                        Err(e) => display::error(&format!("Delete failed: {}", e)),
                    }
                }
            }
            _ => continue,
        }
    }
}

fn show_model_info(i18n: &I18n, model: &ollama_api::OllamaModel, hw: &HardwareInfo) {
    display::section(&format!("ℹ️ {}", model.name));
    
    println!("  {}: {}", i18n.t("models.info_size"), model.formatted_size());
    
    if let Some(ref modified) = model.modified_at {
        println!("  {}: {}", i18n.t("models.info_modified"), modified);
    }
    
    if let Some(ref details) = model.details {
        if let Some(ref family) = details.family {
            println!("  {}: {}", i18n.t("models.info_family"), family);
        }
        if let Some(ref param_size) = details.parameter_size {
            println!("  {}: {}", i18n.t("models.info_param_size"), param_size);
        }
    }
    
    // Check la compatibilité hardware
    println!();
    let score = ollama_models::score_model(model, "mixed", hw);
    if score > 0.0 {
        println!("  {}", i18n.t("models.hardware_ok"));
    } else {
        println!("  {}", i18n.t("models.hardware_warning"));
    }
}