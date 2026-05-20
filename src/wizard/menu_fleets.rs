use anyhow::Result;
use dialoguer::Select;
use crate::config::{self, I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, system};
use crate::display;
use crate::tools::openclaw::OpenClawTool;

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    // Affiche le header avec ressources comme le menu principal
    let ram_avail = system::get_available_ram_gb();
    let vram_avail = system::get_available_vram_gb();
    let running = ollama_api::get_running_models();
    display::clear_screen();
    display::header_with_resources(
        &i18n.t("menu.main.fleets"),
        hw.ram_gb, ram_avail, 
        hw.total_vram_mb as f64 / 1024.0, vram_avail, 
        &running,
        state.last_model.as_deref()
    );
    
    let fleets = config::fleets::detect_openclaw_fleets();
    state.fleets = fleets.clone();
    let _ = config::state::save(state);

    if fleets.is_empty() {
        display::warning(&i18n.t("fleet.none_found"));
        let create = dialoguer::Confirm::new()
            .with_prompt(i18n.t("fleet.create_new"))
            .default(true)
            .interact_opt()?
            .unwrap_or(false);
        if create {
            // Créer une flotte (nécessite un modèle)
            crate::wizard::menu_models::run(i18n, state, hw)?;
        }
        // Lance OpenClaw avec le modèle courant même si pas de flotte
        use crate::tools::tool_trait::Tool;
        let model = state.last_model.as_deref();
        OpenClawTool.launch(i18n, state, model)?;
        return Ok(());
    }

    // Afficher les ressources hardware
    println!("\n{}", i18n.t("fleet.resources"));
    crate::wizard::menu_main::display_hardware(hw, i18n);
    
    let mut items: Vec<String> = fleets.iter().map(|(name, f)| {
        let s = if f.openclaw_installed { "✅" } else { "📄" };
        format!("{} {} ({} agents)", s, name, f.agents.len())
    }).collect();
    items.push(i18n.t("fleet.create_new"));
    items.push(i18n.t("menu.back"));

    let sel = match Select::new().with_prompt(i18n.t("fleet.choose")).items(&items).default(0).max_length(15).interact_opt()? {
        Some(s) => s,
        None => return Ok(()), // Escape pressed
    };

    if sel == fleets.len() {
        crate::wizard::menu_models::run(i18n, state, hw)?;
    } else if sel == fleets.len() + 1 {
        return Ok(());
    } else {
        let (name, _) = fleets.iter().nth(sel).unwrap();
        state.set_last_fleet(name);
        println!("\n🦞 {}", i18n.t("fleet.launching"));
        let _ = OpenClawTool::run_fleet(i18n, name);
    }
    Ok(())
}