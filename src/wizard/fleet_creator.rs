use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, Input};
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models};
use crate::display;
use crate::tools::openclaw::OpenClawTool;
use crate::wizard::fleet_templates;

pub fn run(
    i18n: &I18n,
    _state: &mut WzllamaState,
    hw: &HardwareInfo,
    chosen: &ollama_api::OllamaModel,
    orchestrator_name: &str,
    usage_type: &str,
) -> Result<()> {
    display::header(&i18n.t("fleet.title"));

    let wizard_model = &chosen.model;
    let mut fleet = fleet_templates::get(usage_type, wizard_model, i18n);
    let capacity = ollama_models::calculate_fleet_capacity(hw, chosen);

    println!("\n{}", i18n.t("fleet.resources"));
    println!("   💾 RAM : {:.1} Go", capacity.ram_total_gb);
    println!("   🎮 VRAM : {:.1} Go", capacity.vram_total_gb);
    println!("   🎯 {} : {} ({} tokens)", i18n.t("fleet.orchestrator"), fleet.orchestrator.model.cyan(), fleet.orchestrator.num_ctx);
    println!("   🧠 {} : {} ({} tokens)", i18n.t("fleet.reflexion"), wizard_model.cyan(), fleet.reflexion_agents.first().map(|a| a.num_ctx).unwrap_or(4096));
    println!("   🤖 {} : {}", i18n.t("fleet.experts_max"), capacity.max_experts_ram);

    if !Confirm::new().with_prompt(i18n.t("fleet.keep_orchestrator")).default(true).interact()? {
        println!("   {}", i18n.t("fleet.skipping_orchestrator"));
    }

    for (i, agent) in fleet.reflexion_agents.iter_mut().enumerate() {
        fleet_templates::edit_agent(i18n, agent, i, &i18n.t("fleet.type_reflexion"))?;
    }
    for (i, agent) in fleet.expert_agents.iter_mut().enumerate() {
        fleet_templates::edit_agent(i18n, agent, i, &i18n.t("fleet.type_expert"))?;
    }

    // Ajouter agents personnalisés
    println!("\n{}", i18n.t("fleet.add_more"));
    let mut add_more = Confirm::new().with_prompt(i18n.t("fleet.add_more_confirm")).default(false).interact()?;
    while add_more {
        let role: String = Input::new().with_prompt(i18n.t("fleet.custom_role")).interact()?;
        let prompt: String = Input::new().with_prompt(i18n.t("fleet.custom_prompt")).interact()?;
        fleet.expert_agents.push(fleet_templates::AgentTemplate {
            name: format!("wzllama-expert-custom-{}", fleet.expert_agents.len() + 1),
            role, model: "qwen2.5:3b".into(), num_ctx: 4096, temperature: 0.5,
            system_prompt: prompt, enabled: true,
        });
        add_more = Confirm::new().with_prompt(i18n.t("fleet.add_another")).default(false).interact()?;
    }

    // Création des agents
    display::section(&i18n.t("fleet.creating_fleet"));
    let mut created_agents: Vec<(String, String)> = vec![];

    for agent in fleet.reflexion_agents.iter().chain(fleet.expert_agents.iter()) {
        if agent.enabled {
            println!("   🤖 {}", agent.role);
            let modelfile = format!(
                "FROM {}\nPARAMETER num_ctx {}\nPARAMETER temperature {:.1}\nSYSTEM \"{}\"",
                agent.model, agent.num_ctx, agent.temperature, agent.system_prompt
            );
            if ollama_api::create_model(&agent.name, &modelfile).is_ok() {
                println!("   ✅ {}", agent.name.cyan());
                created_agents.push((agent.name.clone(), agent.role.clone()));
            }
        }
    }

    if created_agents.is_empty() {
        display::warning(&i18n.t("fleet.nothing_created"));
        return Ok(());
    }

    // Déléguer la création openclaw.json à OpenClawTool
    let project_name = OpenClawTool::create_fleet_config(
        i18n, usage_type, orchestrator_name, &created_agents,
    )?;

    println!("\n🦞 {}", i18n.t("fleet.openclaw_launch"));
    println!("   openclaw --profile {}", project_name.cyan());

    Ok(())
}