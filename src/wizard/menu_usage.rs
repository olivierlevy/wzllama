use anyhow::Result;
use colored::*;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};
use crate::core::hardware::HardwareInfo;

pub fn run(i18n: &I18n, state: &mut WzllamaState, _hw: &HardwareInfo) -> Result<()> {
    loop {
        // Display header
        println!("\n{}", "═".repeat(50).cyan());
        println!("  🤖 {}", i18n.t("menu.usage.title"));
        println!("{}\n", "═".repeat(50).cyan());

        // Build menu items with i18n
        let items = vec![
            format!("{}\n   {}", i18n.t("usage.big_book.label"), i18n.t("usage.big_book.description")),
            format!("{}\n   {}", i18n.t("usage.big_code.label"), i18n.t("usage.big_code.description")),
            format!("{}\n   {}", i18n.t("usage.fast_agents.label"), i18n.t("usage.fast_agents.description")),
            format!("{}\n   {}", i18n.t("usage.mixed.label"), i18n.t("usage.mixed.description")),
            i18n.t("menu.back").to_string(),
        ];

        let default = match state.last_usage.as_deref() {
            Some("big_book") => 0,
            Some("big_code") => 1,
            Some("fast_agents") => 2,
            Some("mixed") => 3,
            _ => 0,
        };

        let choice = match Select::new()
            .with_prompt(i18n.t("menu.usage.choose"))
            .items(&items)
            .default(default)
            .interact_opt()? {
            Some(c) => c,
            None => break,
        };

        let usage = match choice {
            0 => "big_book",
            1 => "big_code",
            2 => "fast_agents",
            3 => "mixed",
            4 => break, // Back
            _ => break,
        };

        state.last_usage = Some(usage.to_string());
        crate::config::state::save(state)?;

        // Exit loop after selection (return to main menu)
        break;
    }
    
    Ok(())
}