use anyhow::Result;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, system};
use crate::display;

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    loop {
        // Affiche le header avec ressources comme le menu principal
        let ram_avail = system::get_available_ram_gb();
        let vram_avail = system::get_available_vram_gb();
        let running = ollama_api::get_running_models();
        display::clear_screen();
        display::header_with_resources(
            &i18n.t("menu.main.cleanup"),
            hw.ram_gb, ram_avail, 
            hw.total_vram_mb as f64 / 1024.0, vram_avail, 
            &running,
            state.last_model.as_deref()
        );
        
        let items = vec![
            i18n.t("cleanup.menu_tools"),
            i18n.t("cleanup.menu_fleets"),
            i18n.t("cleanup.menu_models"),
            i18n.t("menu.back"),
        ];

        let sel = match Select::new()
            .with_prompt(i18n.t("cleanup.choose"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact_opt()? {
            Some(s) => s,
            None => return Ok(()), // Escape pressed
        };

        match sel {
            0 => super::cleanup_tools::run(i18n, state)?,
            1 => super::cleanup_fleets::run(i18n, state)?,
            2 => super::cleanup_models::run(i18n, state)?,
            _ => return Ok(()),
        }
    }
}