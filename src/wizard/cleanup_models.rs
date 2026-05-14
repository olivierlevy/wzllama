use anyhow::Result;
use dialoguer::{Select, Confirm};
use crate::config::I18n;
use crate::core::ollama_api;
use crate::display;

pub fn run(i18n: &I18n, _state: &mut crate::config::WzllamaState) -> Result<()> {
    loop {
        let models = ollama_api::list_wzllama_models();
        
        if models.is_empty() {
            display::info(&i18n.t("cleanup.no_models"));
            return Ok(());
        }

        let mut items: Vec<String> = models.iter().map(|m| format!("🗑️  {}", m)).collect();
        items.push(i18n.t("cleanup.delete_all_models"));
        items.push(i18n.t("menu.back"));

        let sel = Select::new()
            .with_prompt(i18n.t("cleanup.choose_model"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact()?;;

        if sel == models.len() + 1 { return Ok(()); }

        if sel == models.len() {
            if Confirm::new().with_prompt(i18n.t("cleanup.confirm")).default(false).interact()? {
                let count = models.len();
                for m in &models { let _ = ollama_api::delete_model(m); }
                display::success(&i18n.t_with_vars("cleanup.models_deleted", &[("count", &count.to_string())]));
            }
        } else {
            let name = &models[sel];
            if Confirm::new().with_prompt(i18n.t("cleanup.confirm")).default(false).interact()? {
                ollama_api::delete_model(name)?;
                display::success(&i18n.t_with_vars("cleanup.model_deleted", &[("name", name)]));
            }
        }
    }
}