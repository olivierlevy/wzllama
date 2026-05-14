use anyhow::Result;
use dialoguer::{Select, Confirm};
use crate::config::{self, I18n, WzllamaState};
use crate::display;

pub fn run(i18n: &I18n, state: &mut WzllamaState) -> Result<()> {
    loop {
        let fleets = config::fleets::detect_openclaw_fleets();
        
        if fleets.is_empty() {
            display::info(&i18n.t("cleanup.no_fleets"));
            return Ok(());
        }

        let mut items: Vec<String> = fleets.keys().map(|n| format!("🗑️  {}", n)).collect();
        items.push(i18n.t("cleanup.delete_all_fleets"));
        items.push(i18n.t("menu.back"));

        let sel = Select::new()
            .with_prompt(i18n.t("cleanup.choose_fleet"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact()?;;

        if sel == fleets.len() + 1 { return Ok(()); }

        if sel == fleets.len() {
            // Supprimer tout
            if Confirm::new().with_prompt(i18n.t("cleanup.confirm")).default(false).interact()? {
                for name in fleets.keys() {
                    config::fleets::delete_fleet(name, state)?;
                }
                display::success(&i18n.t("cleanup.fleets_deleted"));
            }
        } else {
            let name = fleets.keys().nth(sel).unwrap();
            if Confirm::new().with_prompt(i18n.t("cleanup.confirm")).default(false).interact()? {
                config::fleets::delete_fleet(name, state)?;
                display::success(&i18n.t_with_vars("cleanup.fleet_deleted", &[("name", name)]));
            }
        }
    }
}