use anyhow::Result;
use colored::*;
use dialoguer::Select;
use crate::config::{self, I18n, WzllamaState};
use crate::core::{hardware::HardwareInfo, system, ollama_api};
use crate::display;
use crate::tools::{llmfit::LLMFitTool, ollama::OllamaTool};
use crate::tools;
use crate::wizard::menu_cleanup;
use crate::wizard::menu_config;
use crate::wizard::menu_fleets;
use crate::wizard::menu_models;
use crate::wizard::menu_scientific;
use crate::wizard::menu_tools;
use crate::wizard::menu_wizard;
use crate::wizard::setup_models;

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

pub fn select_language(state: &mut WzllamaState) -> Result<I18n> {
    // Si une langue est déjà enregistrée, la charger directement sans menu
    if let Some(ref lang) = state.language {
        let i18n = config::i18n::load(lang)?;
        return Ok(i18n);
    }

    // Premier lancement : choisir la langue
    let languages = config::i18n::get_available_languages();
    let system_lang = config::i18n::detect_system_language();
    let default = languages.iter().position(|l| l.code == system_lang).unwrap_or(0);
    let items: Vec<String> = languages.iter().map(|l| format!("{} ({})", l.name, l.code)).collect();

    println!("{}", "═".repeat(50).cyan());
    let sel = match Select::new()
        .with_prompt("🌍 Langue / Language")
        .items(&items)
        .default(default)
        .interact_opt()? {
        Some(s) => s,
        None => return Err(anyhow::anyhow!("Language selection cancelled")),
    };
    let i18n = config::i18n::load(&languages[sel].code)?;
    config::state::set_language(&languages[sel].code, state);
    Ok(i18n)
}

pub fn change_language(state: &mut WzllamaState) -> Result<I18n> {
    let languages = config::i18n::get_available_languages();
    let current = state.language.clone().unwrap_or_else(|| "fr".into());
    let default = languages.iter().position(|l| l.code == current).unwrap_or(0);
    let items: Vec<String> = languages.iter().map(|l| format!("{} ({})", l.name, l.code)).collect();
    
    let mut all_items = items.clone();
    all_items.push("↩️  Retour".to_string());

    let sel = match Select::new()
        .with_prompt("🌍 Langue / Language")
        .items(&all_items)
        .default(default)
        .interact_opt()? {
        Some(s) => s,
        None => return config::i18n::load(&current), // Escape - return current language
    };

    if sel == items.len() {
        // Retour : recharger la langue actuelle
        return config::i18n::load(&current);
    }

    let i18n = config::i18n::load(&languages[sel].code)?;
    config::state::set_language(&languages[sel].code, state);
    Ok(i18n)
}

pub fn display_hardware(hw: &HardwareInfo, i18n: &I18n) {
    println!("  {}: {}", i18n.t("system.os").dimmed(), hw.os.bold());
    println!("  {}: {:.1} Go", i18n.t("system.ram").dimmed(), hw.ram_gb);
    if hw.has_gpu() {
        for (i, gpu) in hw.gpus.iter().enumerate() {
            println!("  {} #{}: {} ({}: {} Mo)", i18n.t("system.gpu").dimmed(), i+1, gpu.name, i18n.t("system.vram").dimmed(), gpu.vram_mb);
        }
    } else {
        println!("  {}: {}", i18n.t("system.gpu").dimmed(), i18n.t("system.no_gpu").yellow());
    }
}

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    let current_i18n = i18n;
    OllamaTool::ensure_running(current_i18n)?;
    LLMFitTool::ensure_running(current_i18n)?;
    setup_models::ensure_first_models(current_i18n, hw, state)?;
    
    // Enter alternate screen buffer for fixed interface
    enter_alternate_screen();
    use std::io::Write;
    std::io::stdout().flush().ok();
    
    let (term_width, term_height) = display::get_terminal_size();
    let compact = term_height < 25 || term_width < 70;
    
    loop {
        // Redraw header each iteration (appears fixed due to alternate screen)
        print!("\x1b[2J\x1b[H"); // Clear screen, cursor home
        
        let ram_avail = system::get_available_ram_gb();
        let vram_avail = system::get_available_vram_gb();
        let running = ollama_api::get_running_models();
        
        if compact {
            display::section(&current_i18n.t("menu.main.title"));
            println!("   💾 {:.1}/{:.1} Go | 🎮 {:.1}/{:.1} Go", 
                ram_avail, hw.ram_gb,
                vram_avail.unwrap_or(0.0), hw.total_vram_mb as f64 / 1024.0);
        } else {
            display::header(&current_i18n.t("menu.main.title"));
            display::resources_with_bars(hw.ram_gb, ram_avail, 
                hw.total_vram_mb as f64 / 1024.0, vram_avail, &running, state.last_model.as_deref());
            
            if hw.has_gpu() {
                for (i, gpu) in hw.gpus.iter().enumerate() {
                    println!("   {} #{}: {}", "🎮".dimmed(), i+1, gpu.name.dimmed());
                }
            }
        }

        let mut items = vec![];

        // Resume option: insert at position 0 if we have both last_tool and last_model
        let has_resume = state.last_tool.is_some() && state.last_model.is_some();
        if has_resume {
            if let Some(ref last_tool) = state.last_tool {
                if let Some(tool) = tools::get_tool(last_tool) {
                    let tool_name = tool.name();
                    let resume_label = current_i18n.t_with_vars("menu.main.resume", &[("tool", tool_name), ("model", state.last_model.as_ref().unwrap())]);
                    items.push(resume_label);
                }
            }
        }
        
        items.push(current_i18n.t("menu.main.wizard"));
        items.push(current_i18n.t("menu.main.models"));
        items.push(current_i18n.t("menu.main.scientific"));
        items.push(current_i18n.t("menu.main.tools"));

        let fleets = config::fleets::detect_openclaw_fleets();
        let has_fleets = !fleets.is_empty();
        if has_fleets {
            items.push(current_i18n.t("menu.main.fleets"));
        }

        items.push(current_i18n.t("menu.main.cleanup"));
        items.push(current_i18n.t("menu.main.config")); 
        items.push(current_i18n.t("menu.main.language"));
        items.push(current_i18n.t("menu.main.quit"));

        let reserved = if compact { 5 } else { 15 };
        
        let choice = match Select::new()
            .with_prompt(current_i18n.t("menu.main.choose"))
            .items(&items)
            .default(0)
            .max_length(display::menu_max_items(items.len(), reserved))
            .interact_opt()? {
            Some(c) => c,
            None => break, // Escape pressed - quit from main menu
        };

        // Menu indices calculation
        // Without resume: wizard(0), models(1), scientific(2), tools(3), [fleets(4)], cleanup(4/5), config(5/6), language(6/7), quit(7/8)
        // With resume: resume(0), wizard(1), models(2), scientific(3), tools(4), [fleets(5)], cleanup(5/6), config(6/7), language(7/8), quit(8/9)
        let base_offset = has_resume as usize;
        let wizard_idx = base_offset;
        let models_idx = 1 + base_offset;
        let scientific_idx = 2 + base_offset;
        let tools_idx = 3 + base_offset;
        let fleets_idx = 4 + base_offset;
        // cleanup is at position 4 without fleets, or 5 with fleets
        let cleanup_idx = 4 + base_offset + has_fleets as usize;
        let config_idx = 5 + base_offset + has_fleets as usize;
        let language_idx = 6 + base_offset + has_fleets as usize;
        let quit_idx = 7 + base_offset + has_fleets as usize;
        
        match choice {
            n if has_resume && n == 0 => {
                // Resume last tool with last model
                if let (Some(ref last_tool), Some(ref last_model)) = (&state.last_tool, &state.last_model) {
                    if let Some(tool) = tools::get_tool(last_tool) {
                        tool.launch(current_i18n, state, Some(last_model))?;
                    }
                }
            }
            n if n == wizard_idx => menu_wizard::run(current_i18n, state, hw)?,
            n if n == models_idx => menu_models::run(current_i18n, state, hw)?,
            n if n == scientific_idx => menu_scientific::run(current_i18n, state, hw)?,
            n if n == tools_idx => menu_tools::run(current_i18n, state, hw)?,
            n if has_fleets && n == fleets_idx => menu_fleets::run(current_i18n, state, hw)?,
            n if n == cleanup_idx => menu_cleanup::run(current_i18n, state, hw)?,
            n if n == config_idx => menu_config::run(current_i18n, state, hw)?,
            n if n == language_idx => {
                let new_i18n = change_language(state)?;
                return run(&new_i18n, state, hw);
            }
            n if n == quit_idx => break,
            _ => break,
        }
    }
    exit_alternate_screen();
    Ok(())
}