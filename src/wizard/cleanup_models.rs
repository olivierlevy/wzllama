use crate::config::I18n;
use crate::core::ollama_api;
use crate::display;
use anyhow::Result;
use dialoguer::{Confirm, Select};

pub fn run(i18n: &I18n, _state: &mut crate::config::WzllamaState) -> Result<()> {
    loop {
        // Liste TOUS les modèles locaux avec leurs tailles
        let all_models = ollama_api::get_models();

        if all_models.is_empty() {
            display::info(&i18n.t("cleanup.no_models"));
            return Ok(());
        }

        // Retour en premier item (selon TODO.md ligne 72)
        let mut items: Vec<String> = vec![i18n.t("menu.back")];
        items.extend(
            all_models
                .iter()
                .map(|m| format!("🗑️  {} ({})", m.name, m.formatted_size())),
        );
        items.push(i18n.t("cleanup.delete_all_models"));

        let sel = match Select::new()
            .with_prompt(i18n.t("cleanup.choose_model"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact_opt()?
        {
            Some(s) => s,
            None => return Ok(()), // Escape pressed
        };

        // Retour en position 0
        if sel == 0 {
            return Ok(());
        }

        // Supprimer tous les modèles (dernier item)
        if sel == items.len() - 1 {
            if Confirm::new()
                .with_prompt(i18n.t("cleanup.confirm"))
                .default(false)
                .interact()?
            {
                let count = all_models.len();
                for m in &all_models {
                    let _ = ollama_api::delete_model(&m.name);
                }
                display::success(
                    &i18n.t_with_vars("cleanup.models_deleted", &[("count", &count.to_string())]),
                );
            }
        } else {
            // Un modèle spécifique (après Retour en position 0)
            let model_idx = sel - 1;
            let model = &all_models[model_idx];
            if Confirm::new()
                .with_prompt(i18n.t("cleanup.confirm"))
                .default(false)
                .interact()?
            {
                ollama_api::delete_model(&model.name)?;
                display::success(
                    &i18n.t_with_vars("cleanup.model_deleted", &[("name", &model.name)]),
                );
            }
        }
    }
}
