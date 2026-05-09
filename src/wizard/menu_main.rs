use anyhow::Result;
use colored::*;
use dialoguer::{Select, Confirm};
use crate::config::{self, I18n, WzllamaState};
use crate::core::{hardware::HardwareInfo, system, ollama_api};
use crate::tools;
use crate::display;
use crate::wizard::{menu_models, menu_tools, menu_install, menu_fleets, menu_cleanup};

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
    let sel = Select::new()
        .with_prompt("🌍 Langue / Language")
        .items(&items)
        .default(default)
        .interact()?;
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

    let sel = Select::new()
        .with_prompt("🌍 Langue / Language")
        .items(&all_items)
        .default(default)
        .interact()?;

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
    let mut current_i18n = i18n; // On va peut-être changer de langue en cours de route
    
    loop {
        let ram_avail = system::get_available_ram_gb();
        let vram_avail = system::get_available_vram_gb();
        let running = ollama_api::get_running_models();

        display::header(&current_i18n.t("menu.main.title"));
        println!("   💾 RAM : {:.1} / {:.1} Go libres", ram_avail, hw.ram_gb);
        if let Some(vram) = vram_avail {
            println!("   🎮 VRAM : {:.1} / {:.1} Go libres", vram, hw.total_vram_mb as f64 / 1024.0);
        }
        if !running.is_empty() {
            println!("   ⚡ Modèles chargés : {}", running.join(", ").dimmed());
        }

        let mut items = vec![
            current_i18n.t("menu.main.models"),
            current_i18n.t("menu.main.tools"),
        ];

        let fleets = config::fleets::detect_openclaw_fleets();
        if !fleets.is_empty() {
            items.push(current_i18n.t("menu.main.fleets"));
        }

        items.push(current_i18n.t("menu.main.install"));
        items.push(current_i18n.t("menu.main.cleanup"));
        items.push(current_i18n.t("menu.main.language"));
        items.push(current_i18n.t("menu.main.quit"));

        let choice = Select::new()
            .with_prompt(current_i18n.t("menu.main.choose"))
            .items(&items)
            .default(0)
            .interact()?;

        let has_fleets = !fleets.is_empty();
        
        match choice {
            0 => menu_models::run(current_i18n, state, hw)?,
            1 => menu_tools::run(current_i18n, state, hw)?,
            2 if has_fleets => menu_fleets::run(current_i18n, state, hw)?,
            n if n == 2 + has_fleets as usize => menu_install::run(current_i18n, state)?,
            n if n == 3 + has_fleets as usize => menu_cleanup::run(current_i18n, state)?,
            n if n == 4 + has_fleets as usize => {
                // Changer de langue
                let new_i18n = change_language(state)?;
                // On ne peut pas réassigner current_i18n directement car c'est une référence
                // On relance la boucle avec la nouvelle langue
                return run(&new_i18n, state, hw);
            }
            _ => break,
        }
    }
    Ok(())
}