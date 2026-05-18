#![allow(dead_code)]

use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, Input};
use crate::config::{I18n, WzllamaState};
use crate::core::{HardwareInfo, ollama_api, ollama_models, shell};
use crate::display;
use crate::wizard::fleet_templates;

pub fn run(
    i18n: &I18n,
    _state: &mut WzllamaState,
    hw: &HardwareInfo,
    chosen: &ollama_api::OllamaModel,
    orchestrator_name: &str,
    usage_type: &str,
) -> Result<String> {
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
    let mut agents_data: Vec<(String, String, String, u32, f32, String)> = vec![];

    for agent in fleet.reflexion_agents.iter().chain(fleet.expert_agents.iter()) {
        if agent.enabled {
            println!("   🤖 {}", agent.role);
            let modelfile = format!(
                "FROM {}\nPARAMETER num_ctx {}\nPARAMETER temperature {:.1}\nSYSTEM \"{}\"",
                agent.model, agent.num_ctx, agent.temperature, agent.system_prompt
            );
            if ollama_api::create_model(&agent.name, &modelfile).is_ok() {
                println!("   ✅ {}", agent.name.cyan());
                agents_data.push((
                    agent.name.clone(),
                    agent.role.clone(),
                    agent.model.clone(),
                    agent.num_ctx,
                    agent.temperature,
                    agent.system_prompt.clone(),
                ));
            }
        }
    }

    if agents_data.is_empty() {
        display::warning(&i18n.t("fleet.nothing_created"));
        anyhow::bail!(i18n.t("fleet.nothing_created")); 
    }

    // Déléguer à OpenClaw
    let project_name = create_fleet(i18n, usage_type, orchestrator_name, &agents_data)?;
    Ok(project_name)
}

pub fn ask_project_name(i18n: &I18n, usage_type: &str) -> Result<String> {
    let err_no_space = i18n.t("fleet.project_name_no_space").to_string();
    let err_valid_char = i18n.t("fleet.project_name_valid_chars").to_string();
    let project_name: String = Input::new()
        .with_prompt(i18n.t("fleet.project_name"))
        .default(usage_type.to_string())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.contains(char::is_whitespace) {
                Err(&err_no_space)
            } else if !input.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                Err(&err_valid_char)
            } else {
                Ok(())
            }
        })
        .interact()?;
    Ok(project_name)
}

// ─── Création de flotte ────────────────────────────
pub fn create_fleet(
    i18n: &I18n,
    usage_type: &str,
    orchestrator_name: &str,
    agents: &[(String, String, String, u32, f32, String)],
) -> Result<String> {
    let project_name: String = ask_project_name(i18n, usage_type)?;

    println!("\n{}", "═".repeat(50).cyan());
    println!("{}", i18n.t("fleet.commands_to_run").bold());
    println!("{}", "═".repeat(50).cyan());
    println!();
    println!("   {}", i18n.t("fleet.copy_paste"));
    println!();

    // 0. Onboard Ollama
    println!("\n{}", "═".repeat(50).cyan());
    display::comment(&i18n.t("fleet.step_onboard").bold());
    let workspace_arg = get_workspace_arg(&project_name)?;
    let onboard_cmd = format!(
        "openclaw --profile {} onboard {} --auth-choice ollama --non-interactive --accept-risk --skip-health --skip-bootstrap --skip-channels --skip-skills --skip-ui",
        project_name, workspace_arg
    );
    shell::run_cmd(&onboard_cmd)?;
    println!();

    // 1. Gateway
    display::comment(&i18n.t("fleet.step_gateway").bold());
    shell::run_cmd(&format!("openclaw --profile {} gateway install --force", project_name))?;
    shell::run_cmd(&format!("openclaw --profile {} config set gateway.mode local", project_name))?;
    println!();

    // 2. Orchestrateur
    display::comment(&i18n.t("fleet.step_orch").bold());
    shell::run_cmd(&format!("openclaw --profile {} config set agents.defaults.model.primary 'ollama/{}'", project_name, orchestrator_name))?;
    shell::run_cmd(&format!("openclaw --profile {} config set agents.defaults.contextTokens 32768", project_name))?;
    println!();

    // 3. Agents (construction du JSON comme avant)
    let mut agents_json = String::from("[");
    for (name, role, _model, ctx, _temp, _prompt) in agents {
        let id = role.to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
            .trim_matches('-')
            .to_string();
        let id = if id.is_empty() { format!("agent-{}", agents_json.len()) } else { id };
        agents_json.push_str(&format!(
            r#"{{"id":"{}","model":{{"primary":"ollama/{}"}},"contextTokens":{},"identity":{{"name":"{}"}}}},"#,
            id, name, ctx, role
        ));
    }
    agents_json.pop();
    agents_json.push(']');

    display::comment(&i18n.t("fleet.step_agents").bold());
    shell::run_cmd(&format!(
        "openclaw --profile {} config set agents.list '{}' --json --merge",
        project_name, agents_json
    ))?;
    println!();

    // 4. Validation et redémarrage
    display::comment(&i18n.t("fleet.step_validate").bold());
    shell::run_cmd(&format!("openclaw --profile {} doctor --fix", project_name))?;
    shell::run_cmd(&format!("openclaw --profile {} gateway restart", project_name))?;
    println!();

    Ok(project_name)
}

fn get_workspace_arg(project_name:&str) -> Result<String> {
    // Répertoire courant
    let current_dir = std::env::current_dir()?;
    // Nom du répertoire courant (dernier segment)
    let current_dir_name = current_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    // Condition
    let workspace_arg = if project_name == current_dir_name {
        // Ajoute --workspace avec le chemin complet du répertoire courant
        format!(" --workspace \"{}\"", current_dir.display())
    } else {
        String::new()
    };
    Ok(workspace_arg)
}