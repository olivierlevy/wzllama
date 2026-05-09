use anyhow::Result;
use std::path::PathBuf;
use colored::*;
use dialoguer::{Select, Confirm, Input};
use crate::config::{self, I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models};
use crate::display;
use crate::wizard::fleet_templates::{self, FleetConfig, AgentTemplate};

pub fn run(
    i18n: &I18n, state: &mut WzllamaState,
    hw: &HardwareInfo, chosen: &ollama_api::OllamaModel,
    orchestrator_name: &str, usage_type: &str,
) -> Result<()> {
    display::header(&i18n.t("fleet.title"));

    let wizard_model = &chosen.model;
    let mut fleet = fleet_templates::get(usage_type, wizard_model, i18n);
    let capacity = ollama_models::calculate_fleet_capacity(hw, chosen);

    println!("\n{}", i18n.t("fleet.resources"));
    println!("   💾 RAM : {:.1} Go", capacity.ram_total_gb);
    println!("   🎮 VRAM : {:.1} Go", capacity.vram_total_gb);
    println!("   🎯 Orchestrateur : {} ({} tokens)", fleet.orchestrator.model.cyan(), fleet.orchestrator.num_ctx);
    println!("   🧠 Réflexion : {} ({} tokens)", wizard_model.cyan(), fleet.reflexion_agents.first().map(|a| a.num_ctx).unwrap_or(4096));
    println!("   🤖 Experts max : {} agents", capacity.max_experts_ram);

    if !Confirm::new().with_prompt(i18n.t("fleet.keep_orchestrator")).default(true).interact()? {
        println!("   {}", i18n.t("fleet.skipping_orchestrator"));
    }

    for (i, agent) in fleet.reflexion_agents.iter_mut().enumerate() {
        fleet_templates::edit_agent(i18n, agent, i, "réflexion")?;
    }
    for (i, agent) in fleet.expert_agents.iter_mut().enumerate() {
        fleet_templates::edit_agent(i18n, agent, i, "expert")?;
    }

    // Création
    display::section(&i18n.t("fleet.creating_fleet"));
    let mut created = vec![];

    for agent in fleet.reflexion_agents.iter().chain(fleet.expert_agents.iter()) {
        if agent.enabled {
            let modelfile = format!("FROM {}\nPARAMETER num_ctx {}\nPARAMETER temperature {:.1}\nSYSTEM \"{}\"",
                agent.model, agent.num_ctx, agent.temperature, agent.system_prompt);
            if ollama_api::create_model(&agent.name, &modelfile).is_ok() {
                created.push((agent.name.clone(), agent.role.clone()));
            }
        }
    }

    if created.is_empty() {
        display::warning(&i18n.t("fleet.nothing_created"));
        return Ok(());
    }

    // Générer openclaw.json
    let project_name: String = Input::new().with_prompt(i18n.t("fleet.project_name")).default(usage_type.to_string()).interact()?;
    let openclaw_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(format!(".openclaw-{}", project_name));
    std::fs::create_dir_all(&openclaw_dir)?;

    let mut agents_json = String::new();
    for agent in fleet.reflexion_agents.iter().chain(fleet.expert_agents.iter()) {
        if agent.enabled {
            let id = agent.name.strip_prefix("wzllama-reflexion-").or(agent.name.strip_prefix("wzllama-expert-")).unwrap_or(&agent.name);
            agents_json.push_str(&format!("      {{ \"id\": \"{}\", \"model\": {{ \"primary\": \"ollama/{}\" }}, \"identity\": {{ \"name\": \"{}\" }} }},\n", id, agent.name, agent.role));
        }
    }
    if agents_json.ends_with(",\n") { agents_json.pop(); agents_json.pop(); agents_json.push('\n'); }

    let config = format!(r#"{{
  "gateway": {{ "mode": "local" }},
  "agents": {{
    "defaults": {{ "model": {{ "primary": "ollama/{}" }} }},
    "list": [{}] 
  }}
}}"#, orchestrator_name, agents_json);

    std::fs::write(openclaw_dir.join("openclaw.json"), &config)?;

    println!("\n🦞 {}", i18n.t("fleet.openclaw_launch"));
    println!("   openclaw --profile {}", project_name.cyan());

    Ok(())
}