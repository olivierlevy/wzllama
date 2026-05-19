use anyhow::Result;
use colored::*;
use dialoguer::{Select, Confirm};
use crate::config::{I18n, WzllamaState};
use crate::core::{hardware::HardwareInfo, ollama_api, localmax_models};
use crate::display;

/// Choisir le type d'usage pour obtenir des modèles adaptés (appelé depuis le menu principal)
pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    use std::io::Write;
    
    display::section(&i18n.t("usage.choose_title"));
    
    // Types d'usage disponibles
    let usage_types = [
        (i18n.t("usage.code.label"), i18n.t("usage.code.description"), "code"),
        (i18n.t("usage.book.label"), i18n.t("usage.book.description"), "book"),
        (i18n.t("usage.agent.label"), i18n.t("usage.agent.description"), "agent"),
        (i18n.t("usage.mixed.label"), i18n.t("usage.mixed.description"), "general"),
    ];
    
    let display_items: Vec<String> = usage_types.iter()
        .map(|(label, desc, _)| format!("{} {}", label, desc))
        .collect();
    
    let mut all_items = display_items.clone();
    all_items.push(i18n.t("menu.back"));
    
    let sel = match Select::new()
        .with_prompt(i18n.t("usage.choose"))
        .items(&all_items)
        .default(0)
        .interact_opt()?
    {
        Some(s) => s,
        None => return Ok(()),
    };
    
    if sel == all_items.len() - 1 {
        return Ok(());
    }
    
    let search_query = usage_types[sel].2;
    
    // Display loading
    println!("{}", i18n.t("models.localmaxxing_loading"));
    std::io::stdout().flush()?;
    
    // Fetch models by search query
    let models = match localmax_models::fetch_models_by_search(search_query, 50) {
        Ok(m) if !m.is_empty() => {
            eprintln!("DEBUG [usage]: Got {} models from API/cache for query '{}'", m.len(), search_query);
            m
        },
        Ok(_) => {
            eprintln!("DEBUG [usage]: API/cache returned empty for query '{}', using fallback", search_query);
            // Try fallback before giving up
            localmax_models::get_popular_models()
        }
        Err(e) => {
            eprintln!("DEBUG [usage]: API/cache error for query '{}': {}", search_query, e);
            // API failed, use fallback
            localmax_models::get_popular_models()
        }
    };
    
    if models.is_empty() {
        display::warning(&i18n.t("models.localmaxxing_empty"));
        println!();
        display::info(&i18n.t("press_enter_to_continue"));
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        return Ok(());
    }
    
    // Récupérer les modèles locaux installés
    let local_models = ollama_api::get_models();
    let local_names: std::collections::HashSet<&str> = local_models.iter().map(|m| m.name.as_str()).collect();
    
    // Group models by organization and sort by params descending
    let mut groups: std::collections::HashMap<String, Vec<localmax_models::LocalMaxModel>> = std::collections::HashMap::new();
    for model in models {
        let org = model.organization.clone();
        groups.entry(org).or_default().push(model);
    }
    
    // Build display items with organization headers
    let mut model_items: Vec<(String, localmax_models::LocalMaxModel, String, bool)> = vec![]; // (display, model, org, is_installed)
    
    // Sort organizations by their best model's params
    let mut orgs: Vec<_> = groups.iter().collect();
    orgs.sort_by(|a, b| {
        let a_best = a.1.iter().map(|m| m.params.unwrap_or(0.0)).fold(0.0, f64::max);
        let b_best = b.1.iter().map(|m| m.params.unwrap_or(0.0)).fold(0.0, f64::max);
        b_best.partial_cmp(&a_best).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // First, collect installed models separately
    let mut installed_items: Vec<(String, localmax_models::LocalMaxModel, String)> = vec![];
    let mut available_items: Vec<(String, localmax_models::LocalMaxModel, String)> = vec![];
    
    for (org, models) in orgs {
        // Sort models in this org by params desc
        let mut sorted = models.clone();
        sorted.sort_by(|a, b| {
            let a_params = a.params.unwrap_or(0.0);
            let b_params = b.params.unwrap_or(0.0);
            b_params.partial_cmp(&a_params).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for model in sorted {
            let display_name = model.display_name.as_ref().unwrap_or(&model.hf_id);
            // Format params as standard sizes: 7b, 14b, 30b, 72b, etc.
            let params = model.params.map_or(String::new(), |p| {
                let rounded = (p / 7.0).round() * 7.0; // Round to nearest 7B
                if (rounded - 7.0).abs() < 0.1 { "7b".to_string() }
                else if (rounded - 14.0).abs() < 0.1 { "14b".to_string() }
                else if (rounded - 30.0).abs() < 0.1 || (rounded - 32.0).abs() < 0.1 { "30b".to_string() }
                else if (rounded - 70.0).abs() < 0.1 || (rounded - 72.0).abs() < 0.1 { "72b".to_string() }
                else { format!("{:.0}b", rounded) }
            });
            let ollama_name = model.to_ollama_name();
            
            let fallback_indicator = if model.is_direct_ollama_mapping() {
                String::new()
            } else {
                format!(" → Ollama: {}", ollama_name).yellow().to_string()
            };
            
            // Check if installed - simple approach like menu_models.rs
            let is_installed = local_names.contains(ollama_name.as_str());
            let status = if is_installed { " ✅ (installed)".green().to_string() } else { String::new() };
            let display = format!("{} [{}] {}{} {}", display_name, params, org, status, fallback_indicator);
            
            if is_installed {
                installed_items.push((display, model, org.clone()));
            } else {
                available_items.push((display, model, org.clone()));
            }
        }
    }
    
    // Combine: installed first, then available
    model_items.extend(installed_items.into_iter().map(|(d, m, o)| (d, m, o, true)));
    model_items.extend(available_items.into_iter().map(|(d, m, o)| (d, m, o, false)));
    
    let display_items: Vec<String> = model_items.iter().map(|(d, _, _, _)| d.clone()).collect();
    let mut all_items = display_items.clone();
    all_items.push(i18n.t("menu.back"));
    
    let sel = match Select::new()
        .with_prompt(i18n.t("models.localmaxxing_select"))
        .items(&all_items)
        .default(0)
        .max_length(20)
        .interact_opt()?
    {
        Some(s) => s,
        None => return Ok(()),
    };
    
    if sel == all_items.len() - 1 {
        return Ok(());
    }
    
    let chosen = &model_items[sel].1;
    let model_name = chosen.to_ollama_name();
    let is_installed = model_items[sel].3;
    
    // Show model details
    let model = chosen.to_ollama_model();
    show_model_details(i18n, &model, hw);
    
    if is_installed {
        // Model already installed - offer to set as default
        display::info(&i18n.t("models.manage_already_installed"));
        if Confirm::new()
            .with_prompt(i18n.t_with_vars("models.set_as_default", &[("model", &model_name)]))
            .default(true)
            .interact()?
        {
            state.set_last_model(&model_name);
            display::success(&i18n.t_with_vars("models.manage_selected", &[("model", &model_name)]));
        }
    } else {
        let confirm = Confirm::new()
            .with_prompt(i18n.t_with_vars("config.download_confirm", &[("model", &model_name)]))
            .default(false)
            .interact()?;
        
        if confirm {
            if let Err(e) = ollama_api::pull_model(&model_name) {
                display::warning(&format!("{}: {}", i18n.t("models.localmaxxing_download_error"), e));
            } else {
                state.set_last_model(&model_name);
                display::success(&i18n.t_with_vars("models.downloaded_success", &[("model", &model_name)]));
            }
        }
    }
    
    Ok(())
}

fn show_model_details(i18n: &I18n, model: &ollama_api::OllamaModel, hw: &HardwareInfo) {
    println!("\n{}", "─".repeat(50).dimmed());
    println!("  {} {}", "📦".cyan(), model.name.bold());
    if let Some(ref details) = model.details {
        if let Some(ref family) = details.family {
            println!("  {} {}", "Famille:".dimmed(), family);
        }
        if let Some(ref ps) = details.parameter_size {
            println!("  {} {}", "Paramètres:".dimmed(), ps);
        }
    }
    println!("  {} {:.1} Go", i18n.t("system.ram").dimmed(), hw.ram_gb);
    println!();
}