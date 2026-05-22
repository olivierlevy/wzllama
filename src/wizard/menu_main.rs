//! Main menu - migrated to use menu_api
//!
//! This module now delegates to the menu_api system for menu structure
//! while preserving the actual business logic in the wizard modules.

use anyhow::Result;
use crate::config::{self, I18n, WzllamaState};
use crate::core::hardware::HardwareInfo;
use crate::menu_api::MainMenuRunner;

/// Select language on first run (kept for initialization)
pub fn select_language(state: &mut WzllamaState) -> Result<I18n> {
    // Si une langue est déjà enregistrée, la charger directement sans menu
    if let Some(ref lang) = state.language {
        let i18n = config::i18n::load(lang)?;
        return Ok(i18n);
    }

    // Premier lancement : détecter la langue système et l'utiliser
    let languages = config::i18n::get_available_languages();
    let system_lang = config::i18n::detect_system_language();
    let selected = languages.iter().position(|l| l.code == system_lang).unwrap_or(0);

    let i18n = config::i18n::load(&languages[selected].code)?;
    config::state::set_language(&languages[selected].code, state);
    Ok(i18n)
}

/// Main menu entry point - uses menu_api system
pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    crate::tools::ollama::OllamaTool::ensure_running(i18n)?;
    crate::tools::llmfit::LLMFitTool::ensure_running(i18n)?;
    crate::wizard::setup_models::ensure_first_models(i18n, hw, state)?;
    
    // Use the menu_api runner
    let mut runner = MainMenuRunner::new(i18n, state, hw);
    runner.run()
}