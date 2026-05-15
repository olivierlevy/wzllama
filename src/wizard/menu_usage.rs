use anyhow::Result;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};
use crate::core::hardware::HardwareInfo;
use crate::config::state::UsageType;

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

        let default = match state.last_usage.as_str() {
            "big_book" => 0,
            "big_code" => 1,
            "fast_agents" => 2,
            "mixed" => 3,
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

        match choice {
            0 => {
                state.last_usage = "big_book".to_string();
                crate::config::state::save(state.clone())?;
            }
            1 => {
                state.last_usage = "big_code".to_string();
                crate::config::state::save(state.clone())?;
            }
            2 => {
                state.last_usage = "fast_agents".to_string();
                crate::config::state::save(state.clone())?;
            }
            3 => {
                state.last_usage = "mixed".to_string();
                crate::config::state::save(state.clone())?;
            }
            4 => break, // Back
            _ => break,
        }

        // Exit loop after selection (return to main menu)
        break;
    }
    
    Ok(())
}