use anyhow::Result;
use colored::*;
use dialoguer::{Select, Confirm};
use crate::config::{self, I18n, WzllamaState};
use crate::core::ollama_api;
use crate::display;

pub fn run(i18n: &I18n, state: &mut WzllamaState) -> Result<()> {
    let fleets = config::fleets::detect_openclaw_fleets();
    let models = ollama_api::list_wzllama_models();

    if fleets.is_empty() && models.is_empty() {
        display::info(&i18n.t("cleanup.nothing"));
        return Ok(());
    }

    display::section(&i18n.t("cleanup.title"));

    let mut items = vec![];
    if !fleets.is_empty() {
        items.push(i18n.t("cleanup.delete_all_fleets"));
        for (name, _) in &fleets { items.push(format!("  🗑️  Flotte : {}", name)); }
    }
    if !models.is_empty() {
        items.push(i18n.t("cleanup.delete_all_models"));
        for m in &models { items.push(format!("  🗑️  Modèle : {}", m)); }
    }
    items.push(i18n.t("menu.back"));

    let sel = Select::new().with_prompt(i18n.t("cleanup.choose")).items(&items).default(0).interact()?;

    if sel == items.len() - 1 { return Ok(()); }

    let confirm = Confirm::new().with_prompt(i18n.t("cleanup.confirm")).default(false).interact()?;
    if !confirm { return Ok(()); }

    if items[sel] == i18n.t("cleanup.delete_all_fleets") {
        for (name, _) in &fleets { config::fleets::delete_fleet(name, state)?; }
        display::success(&i18n.t("cleanup.fleets_deleted"));
    } else if items[sel] == i18n.t("cleanup.delete_all_models") {
        let count = models.len();
        for m in &models { let _ = ollama_api::delete_model(m); }
        display::success(&i18n.t_with_vars("cleanup.models_deleted", &[("count", &count.to_string())]));
    }

    Ok(())
}