#![allow(dead_code)]

use anyhow::Result;
use colored::*;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models::{ModelConfig, TaskType}};
use crate::tools::openclaw::OpenClawTool;
use crate::wizard::fleet_creator;

pub fn display_and_choose(
    i18n: &I18n, state: &mut WzllamaState,
    config: &ModelConfig, model: &ollama_api::OllamaModel,
    usage_type: &str, hw: &HardwareInfo,
) -> Result<()> {
    println!("\n{}", i18n.t("config.title").bold());
    println!("   {}", i18n.t_with_vars("config.ctx", &[("ctx", &config.num_ctx.to_string())]));
    println!("   {}", i18n.t_with_vars("config.kv_cache", &[("kv", &config.kv_cache_type)]));
    println!("   {}", if config.flash_attention { i18n.t("config.flash_on") } else { i18n.t("config.flash_off") });
    println!("   {}", i18n.t_with_vars("config.temp", &[("temp", &format!("{:.1}", config.temperature))]));
    println!("\n   {}\n{}", i18n.t("config.modelfile"), config.generate_modelfile().dimmed());

    // Menu d'action
    let custom_name = format!("wzllama-{}", TaskType::parse_from_str(usage_type).to_str());
    
    let items = vec![
        i18n.t("config.action_chat"),
        i18n.t("config.action_create_model"),
        i18n.t("config.action_fleet"),
        i18n.t("config.action_back"),
    ];

    let sel = match Select::new().with_prompt(i18n.t("config.launch_choose")).items(&items).default(0).interact_opt()? {
        Some(s) => s,
        None => return Ok(()), // Escape pressed
    };

    match sel {
        0 => {
            println!("\n{}", config.env_vars_display().cyan());
            println!("ollama run {}", model.name.cyan());
            state.set_last_model(&model.name);
        }
        1 => {
            let _cmd = config.write_modelfile(&custom_name)?;
            ollama_api::create_model(&custom_name, &config.generate_modelfile())?;
            println!("   ✅ {}", i18n.t_with_vars("config.created", &[("name", &custom_name)]));
            state.set_last_model(&custom_name);
        }
        2 => {
            // Créer le modèle d'abord puis la flotte
            ollama_api::create_model(&custom_name, &config.generate_modelfile())?;
            let project_name = fleet_creator::run(i18n, state, hw, model, &custom_name, usage_type)?;
            // Lancement final
            let _ = OpenClawTool::run_fleet(i18n, &project_name);
        }
        _ => {}
    }
    Ok(())
}