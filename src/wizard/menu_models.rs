use anyhow::Result;
use dialoguer::{Select, Confirm};
use colored::Colorize;
use std::collections::HashMap;
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models, localmax_models::{self, LocalMaxModel}, cache};
use crate::display;

/// Enter alternate screen buffer (keeps content fixed)
fn enter_alternate_screen() {
    print!("\x1b[?1049h");
    use std::io::Write;
    std::io::stdout().flush().ok();
}

/// Exit alternate screen buffer
fn exit_alternate_screen() {
    print!("\x1b[?1049l");
    use std::io::Write;
    std::io::stdout().flush().ok();
}

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

/// Convert an OllamaModel to a LocalMaxModel by finding matching entry in models list
/// or creating a minimal one for local-only models
fn ollama_to_localmax_model(
    ollama_model: &ollama_api::OllamaModel,
    models: &[LocalMaxModel],
) -> LocalMaxModel {
    let ollama_name = &ollama_model.name;
    
    // Try to find matching model in localmaxxing database
    let matching_model = models.iter().find(|m| {
        // Check if hf_id directly matches (already ollama name)
        if m.hf_id == *ollama_name {
            return true;
        }
        // Check if ollama_name conversion matches
        let converted_name = m.to_ollama_name();
        converted_name == *ollama_name
    });
    
    match matching_model {
        Some(m) => m.clone(),
        None => {
            // Create a minimal LocalMaxModel for local-only models
            LocalMaxModel {
                hf_id: ollama_name.clone(),
                display_name: Some(ollama_name.clone()),
                organization: "local".to_string(),
                ..Default::default()
            }
        }
    }
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
    
    // Récupérer les modèles locaux (just to check if any exist)
    let _local = ollama_api::detect_url().and_then(|u| ollama_api::fetch_local_models(&u).ok()).unwrap_or_default();
    
    // Fetch localmaxxing models (uses cache)
    display::section(&i18n.t("models.localmaxxing_title"));
    
    // Try localmaxxing API first - uses daily cache in fetch_models_by_search
    // Never use fallback models, only cached data from localmaxxing
    let models = localmax_models::fetch_models_by_search("performance", 100)
        .unwrap_or_else(|_| Vec::new());
    
    if models.is_empty() {
        display::warning(&i18n.t("models.localmaxxing_empty"));
        exit_alternate_screen();
        return Ok(());
    }

    // Enter alternate screen buffer for clean redraw
    enter_alternate_screen();
    use std::io::Write;
    std::io::stdout().flush().ok();
    
    'outer: loop {
        // Refresh local models to detect newly installed models
        let local = ollama_api::get_models();
        let local_names: std::collections::HashSet<&str> = local.iter().map(|m| m.name.as_str()).collect();
        
        // Clear screen for clean redraw (alternate screen buffer)
        print!("\x1b[2J\x1b[H");
        use std::io::Write;
        std::io::stdout().flush().ok();
        
        // Rebuild installed models list
        let mut installed_items: Vec<(String, LocalMaxModel)> = vec![];
        for ollama_model in &local {
            let local_model = ollama_to_localmax_model(ollama_model, &models);
            let params = ollama_model.name.split(':')
                .nth(1)
                .map(|s| s.to_string())
                .unwrap_or_else(|| localmax_models::extract_param_size(&local_model.hf_id));
            
            let hw_compat = local_model.hardware_compatibility(hw);
            let name_colored = match hw_compat {
                "🟢" => ollama_model.name.green().to_string(),
                "🟡" => ollama_model.name.yellow().to_string(),
                "🟠" => {
                    let c = colored::Color::TrueColor { r: 245, g: 158, b: 11 };
                    (&ollama_model.name as &str).color(c).to_string()
                },
                _ => (&ollama_model.name as &str).red().to_string(),
            };
            
            let display = format!(
                "✅ {} [{}] {} (installed)",
                name_colored,
                params,
                local_model.organization
            );
            installed_items.push((display, local_model));
        }
        
        installed_items.sort_by(|a, b| {
            let a_priority = match a.1.hardware_compatibility(hw) {
                "🟢" => 0, "🟡" => 1, "🟠" => 2, _ => 3,
            };
            let b_priority = match b.1.hardware_compatibility(hw) {
                "🟢" => 0, "🟡" => 1, "🟠" => 2, _ => 3,
            };
            a_priority.cmp(&b_priority)
        });
        
        // Rebuild organization groups
        let mut groups: HashMap<String, Vec<LocalMaxModel>> = HashMap::new();
        for model in &models {
            let ollama_name = model.to_ollama_name();
            let is_installed = local_names.contains(ollama_name.as_str());
            if !is_installed {
                let org = model.organization.clone();
                groups.entry(org).or_default().push(model.clone());
            }
        }
        
        let mut orgs: Vec<_> = groups.iter().collect();
        orgs.sort_by(|a, b| {
            let a_popularity: u32 = a.1.iter().map(|m| m._count.as_ref().map_or(0, |c| c.benchmark_runs)).sum();
            let b_popularity: u32 = b.1.iter().map(|m| m._count.as_ref().map_or(0, |c| c.benchmark_runs)).sum();
            b_popularity.cmp(&a_popularity)
        });
        
        // Build main menu items
        let mut main_items = vec![];
        for (display, model) in &installed_items {
            main_items.push((display.clone(), Some(model.clone())));
        }
        
        let installed_count = installed_items.len();
        main_items.push((format!("─── {} ───", i18n.t("models.localmaxxing_by_org")), None));
        
        for (org, org_models) in &orgs {
            let count = org_models.len();
            let display = format!("🏢 {} ({})", org, i18n.t_with_vars("models.localmaxxing_org_count", &[("count", &count.to_string())]));
            main_items.push((display, None));
        }
        
        main_items.push((i18n.t("menu.back"), None));
        
        let display_items: Vec<String> = main_items.iter().map(|(d, _)| d.clone()).collect();
        
        let sel = match Select::new()
            .with_prompt(i18n.t("menu.select"))
            .items(&display_items)
            .default(0)
            .max_length(20)
            .interact_opt()?
        {
            Some(s) => s,
            None => {
                exit_alternate_screen();
                return Ok(());
            }
        };
        
        // Handle selection
        if sel == display_items.len() - 1 {
            exit_alternate_screen();
            return Ok(());
        }
        
        let (_, model_opt) = &main_items[sel];
        
        if let Some(model) = model_opt {
            handle_model_selection(i18n, state, hw, model, &local_names)?;
            continue 'outer;
        }
        
        let org_index = sel - (installed_count + 1);
        
        if org_index >= orgs.len() {
            continue 'outer;
        }
        
        let (org, org_models) = orgs[org_index];
        
        let mut sorted_models = org_models.clone();
        sorted_models.sort_by(|a, b| {
            let a_priority = match a.hardware_compatibility(hw) {
                "🟢" => 0, "🟡" => 1, "🟠" => 2, _ => 3,
            };
            let b_priority = match b.hardware_compatibility(hw) {
                "🟢" => 0, "🟡" => 1, "🟠" => 2, _ => 3,
            };
            match a_priority.cmp(&b_priority) {
                std::cmp::Ordering::Equal => {
                    let a_pop = a._count.as_ref().map_or(0, |c| c.benchmark_runs);
                    let b_pop = b._count.as_ref().map_or(0, |c| c.benchmark_runs);
                    b_pop.cmp(&a_pop)
                }
                other => other,
            }
        });
        
        show_org_models_menu(i18n, state, hw, &sorted_models, org)?;
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
            .with_prompt(i18n.t_with_vars("config.download_confirm", &[("model", &model_name)]))
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
) -> Result<()> {
    display::section(&format!("🏢 {} models", org_name));
    
    loop {
        // Refresh local models to detect newly installed models
        let local = ollama_api::get_models();
        let local_names: std::collections::HashSet<&str> = local.iter().map(|m| m.name.as_str()).collect();
        
        // Build display items
        let mut model_items: Vec<(String, LocalMaxModel)> = vec![];
        
        for model in models {
            let display_name = model.display_name.as_ref().unwrap_or(&model.hf_id);
            let params = model.params.map_or(String::new(), |p| {
                let rounded = (p / 7.0).round() * 7.0;
                if (rounded - 7.0).abs() < 0.1 { "7b".to_string() }
                else if (rounded - 14.0).abs() < 0.1 { "14b".to_string() }
                else if (rounded - 30.0).abs() < 0.1 || (rounded - 32.0).abs() < 0.1 { "30b".to_string() }
                else if (rounded - 70.0).abs() < 0.1 || (rounded - 72.0).abs() < 0.1 { "72b".to_string() }
                else { format!("{:.0}b", rounded) }
            });
            let ollama_name = model.to_ollama_name();
            
            // Check if installed using fresh local_names
            let is_installed = local_names.contains(ollama_name.as_str());
            let icon = if is_installed { "✅" } else { "📥" };
            
            let fallback_indicator = if is_installed {
                String::new()
            } else {
                format!(" → {}", ollama_name).yellow().to_string()
            };
            
            let hw_compat = model.hardware_compatibility(hw);
            
            // Color the display name based on hardware compatibility
            let display_name_colored = match hw_compat {
                "🟢" => display_name.green().to_string(),
                "🟡" => display_name.yellow().to_string(),
                "🟠" => {
                    let c = colored::Color::TrueColor { r: 245, g: 158, b: 11 };
                    (display_name as &str).color(c).to_string()
                },
                _ => (display_name as &str).red().to_string(),  // 🔴
            };
            
            let display = format!("{} {} [{}]{}", icon, display_name_colored, params, fallback_indicator);
            model_items.push((display, model.clone()));
        }
        
        let display_items: Vec<String> = model_items.iter().map(|(d, _)| d.clone()).collect();
        let mut all_items = vec![];
        all_items.extend(display_items);
        all_items.push(i18n.t("menu.back"));
        
        let sel = match Select::new()
            .with_prompt(i18n.t("menu.select"))
            .items(&all_items)
            .default(0)
            .max_length(20)
            .interact_opt()?
        {
            Some(s) => s,
            None => return Ok(()),
        };
        
        if sel == all_items.len() - 1 {
            // Back button - return to parent (organization list)
            return Ok(());
        }
        
        let chosen = &model_items[sel].1;
        handle_model_selection(i18n, state, hw, chosen, &local_names)?;
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
            .with_prompt(i18n.t("models.manage_action"))
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
                if let Some(m) = model {
                    show_installed_model_info(i18n, m, hw);
                }
            }
            1 => {
                // Show model info
                if let Some(m) = model {
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
                    .with_prompt(i18n.t_with_vars("models.manage_delete_confirm", &[("model", model_name)]))
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
            println!("  Template: {}", short_template.dimmed());
        }
        
        if let Some(ref info) = details.model_info {
            if let Some(arch) = info.get("architecture").and_then(|a| a.as_str()) {
                println!("  Arch: {}", arch);
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