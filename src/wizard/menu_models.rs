use anyhow::Result;
use dialoguer::{Select, Confirm};
use colored::Colorize;
use std::collections::HashMap;
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models, localmax_models::{self, LocalMaxModel}, cache};
use crate::display;

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
    
    // Separate installed models from all models
    let installed_models: Vec<_> = models.iter()
        .filter(|m| {
            let is_direct = m.is_direct_ollama_mapping();
            let ollama_name = m.to_ollama_name();
            is_direct && local_names.contains(ollama_name.as_str())
        })
        .cloned()
        .collect();
    
    // Build organization groups for non-installed models
    let mut groups: HashMap<String, Vec<LocalMaxModel>> = HashMap::new();
    for model in &models {
        let is_direct = model.is_direct_ollama_mapping();
        let ollama_name = model.to_ollama_name();
        let is_installed = is_direct && local_names.contains(ollama_name.as_str());
        if !is_installed {
            let org = model.organization.clone();
            groups.entry(org).or_default().push(model.clone());
        }
    }
    
    // Sort organizations by popularity (total benchmark runs)
    let mut orgs: Vec<_> = groups.iter().collect();
    orgs.sort_by(|a, b| {
        let a_popularity: u32 = a.1.iter().map(|m| m._count.as_ref().map_or(0, |c| c.benchmark_runs)).sum();
        let b_popularity: u32 = b.1.iter().map(|m| m._count.as_ref().map_or(0, |c| c.benchmark_runs)).sum();
        b_popularity.cmp(&a_popularity)
    });
    
    // Build main menu items
    let mut main_items = vec![];
    
    // Add legend for hardware compatibility indicators at the top (visible on first page)
    main_items.push((format!("─── 🟢=Excellent 🟡=OK 🟠=Low 🔴=Not recommended ───"), None));
    
    // Add installed models section if any (sorted by popularity)
    if !installed_models.is_empty() {
        let mut sorted_installed = installed_models.clone();
        sorted_installed.sort_by(|a, b| {
            let a_pop = a._count.as_ref().map_or(0, |c| c.benchmark_runs);
            let b_pop = b._count.as_ref().map_or(0, |c| c.benchmark_runs);
            b_pop.cmp(&a_pop)
        });
        for model in &sorted_installed {
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
            let hw_compat = model.hardware_compatibility(hw);
            let display = format!("✅ {} [{}] {} (installed) {}", display_name, params, model.organization, hw_compat);
            main_items.push((display, Some(model.clone())));
        }
    }
    
    // Add separator and organization submenu
    main_items.push((format!("─── {} ───", i18n.t("models.localmaxxing_by_org")), None));
    
    // Add organization submenu items
    for (org, org_models) in &orgs {
        let count = org_models.len();
        let display = format!("🏢 {} ({})", org, i18n.t_with_vars("models.localmaxxing_org_count", &[("count", &count.to_string())]));
        main_items.push((display, None)); // None means it's a submenu header
    }
    
    main_items.push((i18n.t("menu.back"), None));
    
    'outer: loop {
    let display_items: Vec<String> = main_items.iter().map(|(d, _)| d.clone()).collect();
    
    let sel = match Select::new()
        .with_prompt(&i18n.t("menu.select"))
        .items(&display_items)
        .default(0)
        .max_length(20)
        .interact_opt()?
    {
        Some(s) => s,
        None => return Ok(()),
    };
    
    // Handle selection
    if sel == display_items.len() - 1 {
        // Back button - exit to main menu
        return Ok(());
    }
    
    let (_, model_opt) = &main_items[sel];
    
    // If it's an installed model, handle it directly
    if let Some(model) = model_opt {
        handle_model_selection(i18n, state, hw, model, &local_names)?;
        continue 'outer;
    }
    
    // Otherwise it's an organization submenu - show models from that org
    let header_idx = if !installed_models.is_empty() { installed_models.len() } else { 0 };
    let org_index = sel - header_idx - 1; // -1 for the separator line
    
    if org_index as usize >= orgs.len() {
        continue 'outer;
    }
    
    let (org, org_models) = orgs[org_index as usize].clone();
    
    // Sort models in this organization by popularity (benchmark runs)
    let mut sorted_models = org_models.clone();
    sorted_models.sort_by(|a, b| {
        let a_pop = a._count.as_ref().map_or(0, |c| c.benchmark_runs);
        let b_pop = b._count.as_ref().map_or(0, |c| c.benchmark_runs);
        b_pop.cmp(&a_pop)
    });
    
    // Show organization models submenu
    show_org_models_menu(i18n, state, hw, &sorted_models, org, &local_names)?;
    }
}

/// Handle model selection - determine if installed or needs download
fn handle_model_selection(
    i18n: &I18n,
    state: &mut WzllamaState,
    hw: &HardwareInfo,
    model: &LocalMaxModel,
    local_names: &std::collections::HashSet<&str>,
) -> Result<()> {
    let model_name = model.to_ollama_name();
    
    // Check if model is installed
    if local_names.contains(model_name.as_str()) {
        // Model already installed - show actions submenu
        run_model_actions_menu(i18n, state, hw, &model_name)?;
    } else {
        // Model not installed - show details and offer to download
        let ollama_model = model.to_ollama_model();
        show_model_details(i18n, &ollama_model, hw);
        
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

/// Show models from a specific organization
fn show_org_models_menu(
    i18n: &I18n,
    state: &mut WzllamaState,
    hw: &HardwareInfo,
    models: &[LocalMaxModel],
    org_name: &str,
    local_names: &std::collections::HashSet<&str>,
) -> Result<()> {
    display::section(&format!("🏢 {} models", org_name));
    
    'org_loop: loop {
        // Build display items
        let mut model_items: Vec<(String, LocalMaxModel)> = vec![];
        
        for model in models {
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
                format!(" → {}", ollama_name).yellow().to_string()
            };
            
            let hw_compat = model.hardware_compatibility(hw);
            let display = format!("{} {} [{}]{} {}", icon, display_name, params, fallback_indicator, hw_compat);
            model_items.push((display, model.clone()));
        }
        
        let display_items: Vec<String> = model_items.iter().map(|(d, _)| d.clone()).collect();
        let mut all_items = vec![];
        
        // Add legend at the top of org submenu
        all_items.push("─── 🟢=Excellent 🟡=OK 🟠=Low 🔴=Not recommended ──".to_string());
        
        all_items.extend(display_items);
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
        
        // Skip legend row (index 0) and back button (last index)
        if sel == 0 {
            continue 'org_loop; // Skip legend, show menu again
        }
        
        if sel == all_items.len() - 1 {
            // Back button - return to parent (organization list)
            return Ok(());
        }
        
        // Adjust index for legend row (-1 to skip legend)
        let chosen = &model_items[sel - 1].1;
        handle_model_selection(i18n, state, hw, chosen, local_names)?;
        // After handling, continue the loop to show the org menu again
    }
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