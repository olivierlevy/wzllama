pub mod configurator;
pub mod estimator;
pub mod fleet_creator;
pub mod fleet_templates;
pub mod menu_cleanup;
pub mod menu_fleets;
pub mod menu_install;
pub mod menu_main;
pub mod menu_models;
pub mod menu_tools;

use anyhow::Result;
use colored::Colorize;
use crate::config::{I18n, WzllamaState};
use crate::display;
use crate::core::HardwareInfo;

pub fn run() -> Result<()> {
    let mut state = WzllamaState::load();

    // Langue (passe le menu si déjà choisie)
    let i18n = menu_main::select_language(&mut state)?;

    // Détection matérielle
    let hardware = crate::core::hardware::detect();
    display::header(&i18n.t("app.title"));
    display::section(&i18n.t("system.detecting"));
    menu_main::display_hardware(&hardware, &i18n);

    // Menu principal navigable
    menu_main::run(&i18n, &mut state, &hardware)?;

    println!("\n{}", i18n.t("app.goodbye").bold().green());
    Ok(())
}
