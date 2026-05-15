use anyhow::Result;
use dialoguer::{Select, Confirm};
use colored::Colorize;
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models};
use crate::display;
use crate::tools::ollama::OllamaTool;

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    // Récupérer les modèles locaux
    let local = ollama_api::detect_url().and_then(|u| ollama_api::fetch_local_models(&u).ok()).unwrap_or_default();
    
    // Si des modèles sont déjà installés, proposer de les utiliser
    if !local.is_empty() {
        return run_with_installed_models(i18n, state, hw, local);
    }
    
    // Sinon, proposer les derniers modèles du catalogue
    run_catalog_selection(i18n, state, hw)
}

fn run_with_installed_models(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo, local: Vec<ollama_api::OllamaModel>) -> Result<()> {
    display::section(&i18n.t("menu.main.models"));
    
    let mut items: Vec<String> = local.iter().map(|m| {
        display::format_model(&m.name, m.size.unwrap_or(0), 1.0, true)
    }).collect();
    
    items.push(i18n.t("menu.models.install_new"));
    items.push(i18n.t("menu.back"));
    
    let sel = match Select::new()
        .with_prompt(&i18n.t("menu.models.choose"))
        .items(&items)
        .default(0)
        .interact_opt()?
    {
        Some(s) => s,
        None => return Ok(()),
    };
    
    if sel == local.len() {
        // Installer un nouveau modèle
        return run_catalog_selection(i18n, state, hw);
    }
    
    if sel == items.len() - 1 {
        return Ok(());
    }
    
    // Utiliser le modèle sélectionné
    let chosen = &local[sel];
    state.set_last_model(&chosen.name);
    display::success(&i18n.t_with_vars("models.manage_selected", &[("model", &chosen.name)]));
    Ok(())
}

fn run_catalog_selection(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    println!("   {}", i18n.t("install.ollama.searching"));
    
    let remote = ollama_api::fetch_remote_catalog().unwrap_or_default();
    
    if remote.is_empty() {
        display::warning(&i18n.t("install.ollama.no_compatible"));
        return Ok(());
    }

    // Afficher les modèles avec leur compatibilité hardware
    display::section(&i18n.t("models.catalog_title"));
    
    let mut model_items: Vec<(String, &ollama_api::OllamaModel, bool)> = remote.iter()
        .map(|m| {
            let score = ollama_models::score_model(m, "mixed", hw);
            let compatible = score > 0.0;
            let status = if compatible { "✅".green() } else { "⚠️".yellow() };
            let size_str = m.formatted_size();
            (format!("{} {} ({})", status, m.name, size_str), m, compatible)
        })
        .collect();
    
    // Trier par compatibilité puis par taille décroissante
    model_items.sort_by(|a, b| {
        match (b.2, a.2) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.1.size.unwrap_or(0).cmp(&a.1.size.unwrap_or(0)),
        }
    });
    
    let display_items: Vec<String> = model_items.iter().map(|(s, _, _)| s.clone()).collect();
    let mut all_items = display_items.clone();
    all_items.push(i18n.t("menu.back"));
    
    let sel = match Select::new()
        .with_prompt(&i18n.t("models.catalog_select"))
        .items(&all_items)
        .default(0)
        .max_length(15)
        .interact_opt()?
    {
        Some(s) => s,
        None => return Ok(()),
    };
    
    if sel == model_items.len() {
        return Ok(());
    }
    
    let (_, chosen, compatible) = model_items[sel];
    
    if !compatible {
        if !Confirm::new()
            .with_prompt(&i18n.t("models.catalog_not_compatible"))
            .default(false)
            .interact()?
        {
            return Ok(());
        }
    }

    // Avant le téléchargement
    if !ollama_api::detect_url().is_some() {
        display::warning(&i18n.t("ollama.not_running"));
        if Confirm::new().with_prompt(i18n.t("ollama.start_now")).default(true).interact()? {
            OllamaTool::start()?;
        } else {
            return Ok(());
        }
    }

    // Télécharger le modèle
    let confirm = Confirm::new()
        .with_prompt(&i18n.t_with_vars("config.download_confirm", &[("model", &chosen.name)]))
        .default(true)
        .interact()?;
    
    if confirm {
        ollama_api::pull_model(&chosen.name)?;
        state.set_last_model(&chosen.name);
    }
    
    Ok(())
}