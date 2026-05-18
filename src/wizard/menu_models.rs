use anyhow::Result;
use dialoguer::{Select, Confirm};
use colored::Colorize;
use std::collections::HashMap;
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models, localmax_models::{self, LocalMaxModel}, cache};
use crate::display;
use crate::tools::ollama::OllamaTool;

fn is_cache_from_today() -> bool {
    let home = dirs::home_dir().unwrap_or_default();
    let cache_path = home.join(".wzllama/cache/localmax_tree.json");
    if let Ok(metadata) = std::fs::metadata(&cache_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(age) = std::time::SystemTime::now().duration_since(modified) {
                // Cache is valid for 7 days
                return age < std::time::Duration::from_secs(7 * 24 * 3600);
            }
        }
    }
    false
}

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    // Only refresh cache if it doesn't exist or is older than 7 days
    let cache_from_today = is_cache_from_today();
    if !cache_from_today {
        match cache::update_daily_models_cache() {
            Ok(()) => {}
            Err(e) => display::warning(&format!("Cache refresh failed: {}", e)),
        }
    }
    
    // Récupérer les modèles locaux
    let local = ollama_api::detect_url().and_then(|u| ollama_api::fetch_local_models(&u).ok()).unwrap_or_default();
    let local_names: std::collections::HashSet<&str> = local.iter().map(|m| m.name.as_str()).collect();
    
    // Fetch localmaxxing models (uses cache)
    display::section(&i18n.t("models.localmaxxing_title"));
    
    // Try localmaxxing API first - uses daily cache in fetch_models_by_search
    // Never use fallback models, only cached data from localmaxxing
    let models = localmax_models::fetch_models_by_search("performance", 100)
        .unwrap_or_else(|_| Vec::new());
    
    if models.is_empty() {
        display::warning(&i18n.t("models.localmaxxing_empty"));
        return Ok(());
    }
    
    // Group models by organization and build display items
    let mut groups: HashMap<String, Vec<LocalMaxModel>> = HashMap::new();
    for model in models {
        let org = model.organization.clone();
        groups.entry(org).or_default().push(model);
    }
    
    // Build display items with download icons for non-installed models
    let mut model_items: Vec<(String, LocalMaxModel)> = vec![];
    
    // Sort organizations by their best model's params
    let mut orgs: Vec<_> = groups.iter().collect();
    orgs.sort_by(|a, b| {
        let a_best = a.1.iter().map(|m| m.params.unwrap_or(0.0)).fold(0.0, f64::max);
        let b_best = b.1.iter().map(|m| m.params.unwrap_or(0.0)).fold(0.0, f64::max);
        b_best.partial_cmp(&a_best).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    for (org, models) in orgs {
        let mut sorted = models.clone();
        sorted.sort_by(|a, b| {
            let a_params = a.params.unwrap_or(0.0);
            let b_params = b.params.unwrap_or(0.0);
            b_params.partial_cmp(&a_params).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for model in sorted {
            let display_name = model.display_name.as_ref().unwrap_or(&model.hf_id);
            let params = model.params.map_or(String::new(), |p| {
                let rounded = (p / 7.0).round() * 7.0;
                if (rounded - 7.0).abs() < 0.1 { "7b".to_string() }
                else if (rounded - 14.0).abs() < 0.1 { "14b".to_string() }
                else if (rounded - 30.0).abs() < 0.1 { "30b".to_string() }
                else if (rounded - 32.0).abs() < 0.1 { "30b".to_string() }
                else if (rounded - 72.0).abs() < 0.1 { "72b".to_string() }
                else if (rounded - 70.0).abs() < 0.1 { "72b".to_string() }
                else { format!("{:.0}b", rounded) }
            });
            let ollama_name = model.to_ollama_name();
            
            // Only show installed icon if direct mapping AND actually installed
            let is_direct = model.is_direct_ollama_mapping();
            let is_installed = is_direct && local_names.contains(ollama_name.as_str());
            let icon = if is_installed { "✅" } else { "📥" };
            
            let fallback_indicator = if is_direct {
                String::new()
            } else {
                format!(" → Ollama: {}", ollama_name).yellow().to_string()
            };
            
            let display = format!("{} {} [{}] {} {}", icon, display_name, params, org, fallback_indicator);
            model_items.push((display, model));
        }
    }
    
    let display_items: Vec<String> = model_items.iter().map(|(d, _)| d.clone()).collect();
    let mut all_items = display_items.clone();
    all_items.push(i18n.t("menu.back"));
    
    let sel = match Select::new()
        .with_prompt(&i18n.t("menu.select"))
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
    
    // Check if model is installed
    if local_names.contains(model_name.as_str()) {
        // Model already installed - show actions submenu
        run_model_actions_menu(i18n, state, hw, &model_name)?;
    } else {
        // Model not installed - show details and offer to download
        let model = chosen.to_ollama_model();
        show_model_details(i18n, &model, hw);
        
        let confirm = Confirm::new()
            .with_prompt(&i18n.t_with_vars("config.download_confirm", &[("model", &model_name)]))
            .default(true)
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

/// Sous-menu d'actions pour un modèle installé
fn run_model_actions_menu(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo, model_name: &str) -> Result<()> {
    loop {
        // Get model details
        let models = ollama_api::get_models();
        let model = models.iter().find(|m| m.name == model_name);
        
        let mut actions = vec![
            i18n.t_with_vars("models.manage_selected", &[("model", model_name)]),
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
            None => break,
        };
        
        match action_sel {
            0 => {
                // Already selected, just show info
                if let Some(ref m) = model {
                    show_installed_model_info(i18n, m, hw);
                }
            }
            1 => {
                // Show model info
                if let Some(ref m) = model {
                    show_installed_model_info(i18n, m, hw);
                }
            }
            2 => {
                // Set as default model
                state.set_last_model(model_name);
                display::success(&i18n.t_with_vars("models.manage_selected", &[("model", model_name)]));
            }
            3 => {
                // Delete model
                if Confirm::new()
                    .with_prompt(&i18n.t_with_vars("models.manage_delete_confirm", &[("model", model_name)]))
                    .default(false)
                    .interact()?
                {
                    match ollama_api::delete_model(model_name) {
                        Ok(()) => {
                            display::success(&i18n.t("models.manage_deleted"));
                            return Ok(()); // Exit after deletion
                        }
                        Err(e) => display::error(&format!("Delete failed: {}", e)),
                    }
                }
            }
            _ => break,
        }
    }
    Ok(())
}

fn show_installed_model_info(i18n: &I18n, model: &ollama_api::OllamaModel, hw: &HardwareInfo) {
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

fn run_catalog_selection(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    println!("   {}", i18n.t("install.ollama.searching"));
    
    let remote = ollama_api::fetch_full_catalog().unwrap_or_default();
    
    if remote.is_empty() {
        display::warning(&i18n.t("install.ollama.no_compatible"));
        return Ok(());
    }

    // Separate cloud models from regular models
    let (cloud_models, local_models): (Vec<_>, Vec<_>) = remote.iter()
        .filter(|m| extract_size(&m.name) > 0)
        .partition(|m| ollama_models::is_cloud_model(m));
    
    // Ask user which category they want
    if !cloud_models.is_empty() {
        display::section(&i18n.t("models.catalog_title"));
        
        let mut category_items = vec![i18n.t("models.local_models")];
        if !cloud_models.is_empty() {
            category_items.push(i18n.t("models.cloud_models"));
        }
        category_items.push(i18n.t("menu.back"));
        
        let cat_sel = match Select::new()
            .with_prompt(&i18n.t("models.choose_category"))
            .items(&category_items)
            .default(0)
            .interact_opt()?
        {
            Some(s) => s,
            None => return Ok(()),
        };
        
        if cat_sel == category_items.len() - 1 {
            return Ok(());
        }
        
        if cat_sel == 1 && !cloud_models.is_empty() {
            return run_cloud_models_selection(i18n, state, hw, cloud_models);
        }
    }
    
    run_local_models_selection(i18n, state, hw, local_models)
}

fn run_local_models_selection(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo, models: Vec<&ollama_api::OllamaModel>) -> Result<()> {
    display::section(&i18n.t("models.catalog_title"));
    
    let mut model_items: Vec<(String, &ollama_api::OllamaModel, f32)> = models.into_iter()
        .map(|m| {
            let score = ollama_models::score_model(m, "mixed", hw);
            let (status, emoji) = match score {
                s if s >= 0.8 => ("Excellent fit".green(), "🚀"),
                s if s >= 0.5 => ("Good fit".green(), "✅"),
                s if s >= 0.2 => ("Fits with constraints".yellow(), "⚠️"),
                _ => ("May not fit".red(), "❌"),
            };
            let size_str = m.formatted_size();
            let hw_str = format_hardware_compatibility(m, hw);
            (format!("{} {} ({}) {} {}", emoji, m.name, size_str, status, hw_str), m, score)
        })
        .collect();
    
    // Trier par score de compatibilité décroissant
    model_items.sort_by(|a, b| {
        b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Pagination - 15 modèles par page
    let page_size = 15;
    let total_pages = (model_items.len() + page_size - 1) / page_size;
    let mut current_page = 0;
    
    loop {
        let start = current_page * page_size;
        let end = (start + page_size).min(model_items.len());
        let page_items = &model_items[start..end];
        
        let display_items: Vec<String> = page_items.iter().map(|(s, _, _)| s.clone()).collect();
        let mut all_items = display_items.clone();
        all_items.push(i18n.t("models.catalog_custom"));
        all_items.push(i18n.t("menu.back"));
        
        // Ajouter les contrôles de pagination
        let has_prev = total_pages > 1 && current_page > 0;
        let has_next = total_pages > 1 && current_page < total_pages - 1;
        
        if has_prev {
            all_items.push(i18n.t("menu.previous_page"));
        }
        if has_next {
            all_items.push(i18n.t("menu.next_page"));
        }
        
        let sel = match Select::new()
            .with_prompt(&format!("{} [Page {}/{}]", i18n.t("models.catalog_select"), current_page + 1, total_pages))
            .items(&all_items)
            .default(0)
            .max_length(20)
            .interact_opt()?
        {
            Some(s) => s,
            None => return Ok(()),
        };
        
        // Gérer la pagination en utilisant la longueur finale d'all_items
        let total_items = all_items.len();
        
        // prev est l'avant-dernier élément si les deux boutons sont présents, ou le dernier sinon
        // next est le dernier élément si présent
        let prev_btn_idx = if has_prev && has_next { total_items - 2 } else if has_prev { total_items - 1 } else { 0 };
        let next_btn_idx = if has_next { total_items - 1 } else { 0 };
        
        if has_prev && sel == prev_btn_idx {
            current_page -= 1;
            continue;
        }
        if has_next && sel == next_btn_idx {
            current_page += 1;
            continue;
        }
        
        // Vérifier si c'est le bouton custom (juste avant back)
        let custom_idx = page_items.len();
        if sel == custom_idx {
            return prompt_custom_model(i18n, state, hw);
        }
        
        // Vérifier si c'est le bouton back
        let back_idx = custom_idx + 1;
        if sel == back_idx {
            return Ok(());
        }
        
        // Si le sélecteur est hors limites, continuer la boucle
        if sel >= page_items.len() {
            continue;
        }
        
        let (_, chosen, score) = page_items[sel];
        
        // Show detailed model information before asking to install
        show_model_details(i18n, chosen, hw);
        
        if score < 0.2 {
            if !Confirm::new()
                .with_prompt(&i18n.t("models.catalog_not_compatible"))
                .default(false)
                .interact()?
            {
                // Return to the loop instead of exiting
                continue;
            }
        }

        // Before download
        if !ollama_api::detect_url().is_some() {
            display::warning(&i18n.t("ollama.not_running"));
            if Confirm::new().with_prompt(i18n.t("ollama.start_now")).default(true).interact()? {
                OllamaTool::start()?;
            } else {
                return Ok(());
            }
        }

        // Download le modèle
        let confirm = Confirm::new()
            .with_prompt(&i18n.t_with_vars("config.download_confirm", &[("model", &chosen.name)]))
            .default(true)
            .interact()?;
        
        if confirm {
            ollama_api::pull_model(&chosen.name)?;
            state.set_last_model(&chosen.name);
        }
        
        return Ok(());
    }
}

fn run_cloud_models_selection(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo, models: Vec<&ollama_api::OllamaModel>) -> Result<()> {
    display::section(&i18n.t("models.cloud_title"));
    
    let mut model_items: Vec<(String, &ollama_api::OllamaModel, f32)> = models.into_iter()
        .map(|m| {
            let score = ollama_models::score_model(m, "mixed", hw);
            let size_str = m.formatted_size();
            (format!("☁️ {} ({})", m.name, size_str), m, score)
        })
        .collect();
    
    model_items.sort_by(|a, b| {
        b.1.size.unwrap_or(0).cmp(&a.1.size.unwrap_or(0))
    });
    
    let display_items: Vec<String> = model_items.iter().map(|(s, _, _)| s.clone()).collect();
    let mut all_items = display_items.clone();
    all_items.push(i18n.t("menu.back"));
    
    let sel = match Select::new()
        .with_prompt(&i18n.t("models.cloud_select"))
        .items(&all_items)
        .default(0)
        .max_length(15)
        .interact_opt()?
    {
        Some(s) => s,
        None => return Ok(()),
    };
    
    if sel == model_items.len() {
        return Ok(());
    }
    
    let (_, chosen, _) = model_items[sel];
    
    // Show detailed model information
    show_model_details(i18n, chosen, hw);
    
    display::warning(&i18n.t("models.cloud_notice"));
    
    // Cloud models don't need local ollama running
    let confirm = Confirm::new()
        .with_prompt(&i18n.t_with_vars("models.cloud_install_confirm", &[("model", &chosen.name)]))
        .default(true)
        .interact()?;
    
    if confirm {
        // For cloud models, just set as default - they run remotely
        state.set_last_model(&chosen.name);
        display::success(&i18n.t("models.cloud_ready"));
    }
    
    Ok(())
}

/// Extract model size in billions of parameters from name
fn extract_size(name: &str) -> u32 {
    for part in name.split([':', '-', '/']) {
        if let Some(size) = part.strip_suffix('b') {
            if let Ok(n) = size.parse::<f32>() {
                return (n * 10.0).round() as u32 / 10;
            }
            if let Ok(n) = size.parse::<u32>() { return n; }
        }
    }
    0
}

/// Format hardware compatibility info for display
fn format_hardware_compatibility(model: &ollama_api::OllamaModel, hw: &HardwareInfo) -> String {
    let vram_gb = hw.total_vram_mb as f64 / 1024.0;
    let ram_gb = hw.ram_gb;
    let size_gb = model.size.unwrap_or(0) as f64 / 1_073_741_824.0;
    let has_gpu = hw.has_gpu();
    
    if !has_gpu {
        if size_gb > ram_gb {
            format!("[RAM: {:.0}GB/{:.0}GB]", ram_gb, size_gb)
        } else {
            String::new()
        }
    } else if size_gb <= vram_gb {
        // Fits in VRAM - optimal
        String::new()
    } else if size_gb <= ram_gb {
        // Fits in RAM but not VRAM - will be slower
        format!("[VRAM: {:.0}GB/{:.0}GB - RAM fallback]", vram_gb, size_gb)
    } else {
        format!("[VRAM: {:.0}GB/{:.0}GB]", vram_gb, size_gb)
    }
}

/// Show detailed model information using /api/show
fn show_model_details(i18n: &I18n, model: &ollama_api::OllamaModel, hw: &HardwareInfo) {
    display::section(&format!("🤖 {}", model.name));
    
    println!("  {}: {}", i18n.t("models.info_size"), model.formatted_size());
    
    // Disk space check
    let model_gb = model.size.unwrap_or(0) as f64 / 1_073_741_824.0;
    if model_gb > hw.available_disk_gb {
        println!("  {} ⚠️ {} {:.0}GB needed, {:.0}GB available", 
            i18n.t("models.info_disk"), "⚠️".yellow(), model_gb, hw.available_disk_gb);
    } else {
        println!("  {}: {:.0}GB available", i18n.t("models.info_disk"), hw.available_disk_gb);
    }
    
    // Try to get detailed info from /api/show
    if let Ok(details) = ollama_api::get_model_details(&model.name) {
        if let Some(family) = details.details.as_ref().and_then(|d| d.family.as_ref()) {
            println!("  {}: {}", i18n.t("models.info_family"), family);
        }
        
        if let Some(param_size) = details.details.as_ref().and_then(|d| d.parameter_size.as_ref()) {
            println!("  {}: {}", i18n.t("models.info_param_size"), param_size);
        }
        
        if let Some(ref license) = details.license {
            let short_license = if license.len() > 50 { 
                format!("{}...", &license[..47]) 
            } else { 
                license.clone() 
            };
            println!("  {}: {}", i18n.t("models.info_license"), short_license.dimmed());
        }
        
        if let Some(ref template) = details.template {
            let short_template = if template.len() > 60 { 
                format!("{}...", &template[..57]) 
            } else { 
                template.clone() 
            };
            println!("  {}: {}", "Template", short_template.dimmed());
        }
        
        if let Some(ref info) = details.model_info {
            if let Some(arch) = info.get("architecture").and_then(|a| a.as_str()) {
                println!("  {}: {}", "Arch", arch);
            }
        }
    } else {
        println!("  {}", "Detailed info unavailable".dimmed());
    }
    
    // Hardware compatibility check
    println!();
    let score = ollama_models::score_model(model, "mixed", hw);
    let has_gpu = hw.has_gpu();
    let vram_gb = hw.total_vram_mb as f64 / 1024.0;
    let size_gb = model.size.unwrap_or(0) as f64 / 1_073_741_824.0;
    
    match score {
        s if s >= 0.8 => println!("  {} 🚀 {}", "Hardware:".green(), "Excellent fit for your system".green()),
        s if s >= 0.5 => {
            if has_gpu && size_gb > vram_gb {
                println!("  {} ✅ {}", "Hardware:".green(), "Good fit (RAM fallback - slower)".yellow());
            } else {
                println!("  {} ✅ {}", "Hardware:".green(), "Good fit".green());
            }
        },
        s if s >= 0.2 => {
            if has_gpu && size_gb > vram_gb {
                println!("  {} ⚠️ {}", "Hardware:".yellow(), "Fits in RAM only - slower performance".yellow());
            } else {
                println!("  {} ⚠️ {}", "Hardware:".yellow(), "Fits with constraints".yellow());
            }
        },
        _ => println!("  {} ❌ {}", "Hardware:".red(), "May not fit in memory".red()),
    }
}

/// Prompt user for a custom model name
fn prompt_custom_model(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    use dialoguer::Input;
    
    let model_name: String = Input::new()
        .with_prompt(&i18n.t("models.catalog_custom_prompt"))
        .interact()?;
    
    let model_name = model_name.trim();
    if model_name.is_empty() {
        return Ok(());
    }
    
    // Create a fake model for display purposes
    let custom_model = ollama_api::OllamaModel {
        name: model_name.to_string(),
        model: model_name.to_string(),
        modified_at: None,
        size: None, // Size will be fetched from API when installing
        details: None,
    };
    
    // Show model info
    show_model_details(i18n, &custom_model, hw);
    
    // Install
    let confirm = Confirm::new()
        .with_prompt(&i18n.t_with_vars("config.download_confirm", &[("model", &custom_model.name)]))
        .default(true)
        .interact()?;
    
    if confirm {
        // Try to pull the model
        if let Err(e) = ollama_api::pull_model(&custom_model.name) {
            display::warning(&format!("{}: {}", i18n.t("tool.install_failed"), e));
        } else {
            state.set_last_model(&custom_model.name);
        }
    }
    
    Ok(())
}