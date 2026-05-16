use anyhow::Result;
use dialoguer::{Select, Confirm};
use colored::Colorize;
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models};
use crate::display;
use crate::tools::ollama::OllamaTool;

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    // Récupérer les modèles locaux
    let local = ollama_api::detect_url().and_then(|u| ollama_api::fetch_local_models(&u).ok()).unwrap_or_default();
    
    // Si des modèles sont déjà installés, proposer de les use
    if !local.is_empty() {
        return run_with_installed_models(i18n, state, hw, local);
    }
    
    // Sinon, proposer les derniers modèles du catalogue
    run_catalog_selection(i18n, state, hw)
}

fn run_with_installed_models(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo, local: Vec<ollama_api::OllamaModel>) -> Result<()> {
    display::section(&i18n.t("menu.main.models"));
    
    let mut items: Vec<String> = local.iter().map(|m| {
        display::format_model(&m.name, m.size.unwrap_or(0), 1.0, true)
    }).collect();
    
    items.push(i18n.t("menu.models.install_new"));
    items.push(i18n.t("menu.back"));
    
    let sel = match Select::new()
        .with_prompt(&i18n.t("menu.models.choose"))
        .items(&items)
        .default(0)
        .interact_opt()?
    {
        Some(s) => s,
        None => return Ok(()),
    };
    
    if sel == local.len() {
        // Installer un nouveau modèle
        return run_catalog_selection(i18n, state, hw);
    }
    
    if sel == items.len() - 1 {
        return Ok(());
    }
    
    // Utiliser le modèle sélectionné
    let chosen = &local[sel];
    state.set_last_model(&chosen.name);
    display::success(&i18n.t_with_vars("models.manage_selected", &[("model", &chosen.name)]));
    Ok(())
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
    
    let display_items: Vec<String> = model_items.iter().map(|(s, _, _)| s.clone()).collect();
    let mut all_items = display_items.clone();
    all_items.push(i18n.t("models.catalog_custom"));
    all_items.push(i18n.t("menu.back"));
    
    let sel = match Select::new()
        .with_prompt(&i18n.t("models.catalog_select"))
        .items(&all_items)
        .default(0)
        .max_length(15)
        .interact_opt()?
    {
        Some(s) => s,
        None => return Ok(()),
    };
    
    if sel == model_items.len() {
        // Custom model input
        return prompt_custom_model(i18n, state, hw);
    }
    
    if sel == all_items.len() - 1 {
        return Ok(());
    }
    
    let (_, chosen, score) = model_items[sel];
    
    // Show detailed model information before asking to install
    show_model_details(i18n, chosen, hw);
    
    if score < 0.2 {
        if !Confirm::new()
            .with_prompt(&i18n.t("models.catalog_not_compatible"))
            .default(false)
            .interact()?
        {
            return Ok(());
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
    
    Ok(())
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