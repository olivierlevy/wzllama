//! Model picker utility for selecting Ollama models
//! Used by tool launchers when no default model is set

use anyhow::Result;
use dialoguer::Select;
use crate::config::I18n;
use crate::core::ollama_api;

/// Pick a model from locally available models via dialog menu
/// 
/// # Returns
/// * `Some(String)` - The selected model name
/// * `None` - User cancelled or no models available
pub fn pick_model(i18n: &I18n) -> Result<Option<String>> {
    if let Some(url) = ollama_api::detect_url() {
        let models = ollama_api::fetch_local_models(&url)?;
        if !models.is_empty() {
            let names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();
            let sel = Select::new()
                .with_prompt(i18n.t("ollama.select_model"))
                .items(&names)
                .default(0)
                .interact_opt()?;
            
            if let Some(idx) = sel {
                return Ok(Some(names[idx].to_string()));
            }
        }
    }
    Ok(None)
}