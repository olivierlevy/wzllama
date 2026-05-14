use anyhow::Result;
use dialoguer::{Select, Confirm};
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models};
use crate::display;
use crate::tools::ollama::OllamaTool;
use crate::wizard::configurator;

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    let _usage = state.last_usage.clone().unwrap_or_else(|| "mixed".into());
    
    // Choisir l'usage
    display::section(&i18n.t("menu.usage.title"));
    let usages = crate::config::templates::load_usages();
    let mut usage_items: Vec<(&String, &crate::config::templates::UsageSpec)> = usages.usages.iter().collect();
    usage_items.sort_by(|a, b| b.1.weights.get("default").unwrap_or(&0.0).partial_cmp(a.1.weights.get("default").unwrap_or(&0.0)).unwrap_or(std::cmp::Ordering::Equal));
    
    let mut items: Vec<String> = usage_items.iter().map(|(_, s)| i18n.t(&s.i18n_key)).collect();
    items.push(i18n.t("menu.back"));
    
    // Handle Escape
    let sel = match Select::new().with_prompt(i18n.t("menu.usage.choose")).items(&items).default(0).max_length(15).interact_opt()? {
        Some(s) => s,
        None => return Ok(()), // Escape pressed
    };
    
    if sel == usage_items.len() { return Ok(()); }
    
    let usage_type = &usage_items[sel].1.params.r#type;
    state.set_last_usage(usage_type);

    // Récupérer les modèles
    println!("   {}", i18n.t("install.ollama.searching"));
    let local = ollama_api::detect_url().and_then(|u| ollama_api::fetch_local_models(&u).ok()).unwrap_or_default();
    let remote = ollama_api::fetch_remote_catalog().unwrap_or_default();
    let all = ollama_api::merge_models(&local, &remote);
    
    let ranked = ollama_models::rank_models(
        &all.iter().map(|(m, _)| m.clone()).collect::<Vec<_>>(),
        usage_type, hw, 12
    );

    if ranked.is_empty() {
        display::warning(&i18n.t("install.ollama.no_compatible"));
        return Ok(());
    }

    // Affichage amélioré des modèles
    let mut model_items: Vec<String> = ranked.iter().map(|(m, s)| {
        let installed = all.iter().any(|(lm, dl)| *dl && lm.name == m.name);
        display::format_model(&m.name, m.size.unwrap_or(0), *s, installed)
    }).collect();
    model_items.push(i18n.t("menu.back"));

    display::section_title("🔍", "Modèles recommandés");
    let sel = match Select::new().with_prompt(i18n.t("install.ollama.choose")).items(&model_items).default(0).max_length(15).interact_opt()? {
        Some(s) => s,
        None => return Ok(()), // Escape pressed
    };
    
    if sel == ranked.len() { return Ok(()); }

    let (chosen, _) = &ranked[sel];

    // Avant le téléchargement
    if OllamaTool::is_running() {
        display::warning(&i18n.t("ollama.not_running"));
        if Confirm::new().with_prompt(i18n.t("ollama.start_now")).default(true).interact()? {
            OllamaTool::start()?;
        } else {
            return Ok(());
        }
    }

    // Télécharger si nécessaire
    let installed = local.iter().any(|m| m.name == chosen.name);
    if !installed {
        let confirm = Confirm::new().with_prompt(i18n.t_with_vars("config.download_confirm", &[("model", &chosen.name)])).default(true).interact()?;
        if confirm { ollama_api::pull_model(&chosen.name)?; }
    }

    // Configurer
    let task = ollama_models::TaskType::from_str(usage_type);
    let config = ollama_models::recommend_config(hw, &task, chosen, i18n);
    configurator::display_and_choose(i18n, state, &config, chosen, usage_type, hw)?;

    Ok(())
}