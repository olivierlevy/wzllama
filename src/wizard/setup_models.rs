use anyhow::Result;
use colored::*;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models};
use crate::display;

pub fn ensure_first_models(i18n: &I18n, hw: &HardwareInfo, state: &mut WzllamaState) -> Result<()> {
    let local = ollama_api::detect_url()
        .and_then(|u| ollama_api::fetch_local_models(&u).ok())
        .unwrap_or_default();
    
    if !local.is_empty() {
        return Ok(());
    }

    display::header(&i18n.t("models.first_time"));
    display::info(&i18n.t("models.no_models_yet"));

    let remote = ollama_api::fetch_remote_catalog().unwrap_or_default();
    
    // 1. Meilleur modèle qualité (gros)
    let heavy = ollama_models::rank_models(&remote, "mixed", hw, 1);
    
    // 2. Modèle léger : forcer un petit modèle connu
    let light_model = "qwen2.5:1.5b".to_string();
    let light = vec![(ollama_api::OllamaModel {
        name: light_model.clone(),
        model: light_model.clone(),
        modified_at: None,
        size: Some(1_000_000_000),  // ~1 Go
        details: None,
    }, 0.8)];

    println!("\n   {}", i18n.t("models.recommended_pair"));

    for (label, ranked) in [("models.heavy", &heavy), ("models.light", &light)] {
        if let Some((model, _)) = ranked.first() {
            let size = display::format_size(model.size.unwrap_or(0));
            println!("   {} : {} ({})", i18n.t(label), model.name.bold(), size);
            if Confirm::new()
                .with_prompt(i18n.t_with_vars("models.install_this", &[("model", &model.name)]))
                .default(true)
                .interact()?
            {
                ollama_api::pull_model(&model.name)?;
            }
        }
    }
    
    // Après la boucle d'installation des modèles
    if let Some((heavy_model, _)) = heavy.first() {
        crate::config::state::set_last_model(&heavy_model.name, state);
    }

    // Mettre à jour la config env avec les modèles installés
    let mut env_config = crate::config::env::EnvConfig::load();
    if let Some((heavy_model, _)) = heavy.first() {
        env_config.models.code = heavy_model.name.clone();
        env_config.models.book = heavy_model.name.clone();
        env_config.models.chat = heavy_model.name.clone();
    }
    if let Some((light_model, _)) = light.first() {
        env_config.models.agent = light_model.name.clone();
    }
    env_config.save()?;

    display::success(&i18n.t("models.ready"));
    Ok(())
}