use anyhow::Result;
use colored::*;
use dialoguer::{Select, Confirm};
use crate::config::{self, I18n, WzllamaState};
use crate::core::{hardware::HardwareInfo, system, ollama_api};
use crate::tools;
use crate::display;
use crate::wizard::{menu_models, menu_tools, menu_install, menu_fleets, menu_cleanup};

pub fn select_language(state: &mut WzllamaState) -> Result<I18n> {
    if let Some(ref lang) = state.language {
        let i18n = config::i18n::load(lang)?;
        println!("🌍 {} : {}", i18n.t("menu.language.current"), i18n.meta.name);
        let change = Confirm::new().with_prompt(i18n.t("menu.language.change")).default(false).interact()?;
        if !change { return Ok(i18n); }
    }

    let languages = config::i18n::get_available_languages();
    let system_lang = config::i18n::detect_system_language();
    let default = languages.iter().position(|l| l.code == system_lang).unwrap_or(0);
    let items: Vec<String> = languages.iter().map(|l| format!("{} ({})", l.name, l.code)).collect();

    let sel = Select::new().with_prompt("🌍 Langue").items(&items).default(default).interact()?;
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
    loop {
        let ram_avail = system::get_available_ram_gb();
        let vram_avail = system::get_available_vram_gb();
        let running = ollama_api::get_running_models();

        display::header(&i18n.t("menu.main.title"));
        println!("   💾 RAM : {:.1} / {:.1} Go libres", ram_avail, hw.ram_gb);
        if let Some(vram) = vram_avail {
            println!("   🎮 VRAM : {:.1} / {:.1} Go libres", vram, hw.total_vram_mb as f64 / 1024.0);
        }
        if !running.is_empty() {
            println!("   ⚡ Modèles chargés : {}", running.join(", ").dimmed());
            println!("   💡 ollama stop <nom> pour libérer de la VRAM");
        }

        let mut items = vec![
            i18n.t("menu.main.models"),
            i18n.t("menu.main.tools"),
        ];

        // Flottes si OpenClaw détecté
        let fleets = config::fleets::detect_openclaw_fleets();
        if !fleets.is_empty() {
            items.push(i18n.t("menu.main.fleets"));
        }

        items.push(i18n.t("menu.main.install"));
        items.push(i18n.t("menu.main.cleanup"));
        items.push(i18n.t("menu.main.language"));
        items.push(i18n.t("menu.main.quit"));

        let choice = Select::new()
            .with_prompt(i18n.t("menu.main.choose"))
            .items(&items)
            .default(0)
            .interact()?;

        let fleets_offset = if fleets.is_empty() { 0 } else { 1 };
        // items indices: models(0), tools(1), fleets(2 si present), install, cleanup, language, quit
        
        // Correction: utiliser des valeurs constantes calculées
        let install_idx = if fleets.is_empty() { 2 } else { 3 };
        let cleanup_idx = if fleets.is_empty() { 3 } else { 4 };
        let language_idx = if fleets.is_empty() { 4 } else { 5 };
        let quit_idx = if fleets.is_empty() { 5 } else { 6 };

        match choice {
            0 => menu_models::run(i18n, state, hw)?,
            1 => menu_tools::run(i18n, state, hw)?,
            idx if idx == 2 && !fleets.is_empty() => menu_fleets::run(i18n, state, hw)?,
            idx if idx == install_idx => menu_install::run(i18n, state)?,
            idx if idx == cleanup_idx => menu_cleanup::run(i18n, state)?,
            idx if idx == language_idx => { select_language(state)?; }
            idx if idx == quit_idx => break,
            _ => break,
        }
    }
    Ok(())
}