use anyhow::Result;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};

pub fn run(i18n: &I18n, state: &mut WzllamaState) -> Result<()> {
    loop {
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