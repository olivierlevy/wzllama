use anyhow::Result;
use colored::*;
use dialoguer::{Select, Input};
use crate::config::I18n;

#[derive(Debug, Clone)]
pub struct FleetConfig {
    pub orchestrator: OrchestratorConfig,
    pub reflexion_agents: Vec<AgentTemplate>,
    pub expert_agents: Vec<AgentTemplate>,
}

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub model: String,
    pub num_ctx: u32,
    #[allow(dead_code)]
    pub system_prompt: String,
}

#[derive(Debug, Clone)]
pub struct AgentTemplate {
    pub name: String,
    pub role: String,
    pub model: String,
    pub num_ctx: u32,
    pub temperature: f32,
    pub system_prompt: String,
    pub enabled: bool,
}

pub fn get(usage_type: &str, wizard_model: &str, i18n: &I18n) -> FleetConfig {
    match usage_type {
        "code" => FleetConfig {
            orchestrator: OrchestratorConfig {
                model: "qwen2.5:7b".into(), num_ctx: 32768,
                system_prompt: i18n.t("fleet.template.orchestrator_code"),
            },
            reflexion_agents: vec![
                AgentTemplate { name: "wzllama-reflexion-arch".into(), role: i18n.t("fleet.template.reflexion_arch"), model: wizard_model.into(), num_ctx: 8192, temperature: 0.3, system_prompt: i18n.t("fleet.template.reflexion_arch_prompt"), enabled: true },
                AgentTemplate { name: "wzllama-reflexion-review".into(), role: i18n.t("fleet.template.reflexion_review"), model: wizard_model.into(), num_ctx: 8192, temperature: 0.4, system_prompt: i18n.t("fleet.template.reflexion_review_prompt"), enabled: true },
            ],
            expert_agents: vec![
                AgentTemplate { name: "wzllama-expert-lint".into(), role: i18n.t("fleet.template.expert_lint"), model: "qwen2.5:1.5b".into(), num_ctx: 2048, temperature: 0.1, system_prompt: i18n.t("fleet.template.expert_lint_prompt"), enabled: true },
                AgentTemplate { name: "wzllama-expert-doc".into(), role: i18n.t("fleet.template.expert_doc"), model: "qwen2.5:3b".into(), num_ctx: 4096, temperature: 0.4, system_prompt: i18n.t("fleet.template.expert_doc_prompt"), enabled: true },
                AgentTemplate { name: "wzllama-expert-test".into(), role: i18n.t("fleet.template.expert_test"), model: "qwen2.5:3b".into(), num_ctx: 4096, temperature: 0.5, system_prompt: i18n.t("fleet.template.expert_test_prompt"), enabled: true },
            ],
        },
        _ => FleetConfig {
            orchestrator: OrchestratorConfig {
                model: "qwen2.5:7b".into(), num_ctx: 32768,
                system_prompt: i18n.t("fleet.template.orchestrator_generic"),
            },
            reflexion_agents: vec![
                AgentTemplate { name: "wzllama-reflexion".into(), role: i18n.t("fleet.template.reflexion_generic"), model: wizard_model.into(), num_ctx: 8192, temperature: 0.5, system_prompt: i18n.t("fleet.template.reflexion_generic_prompt"), enabled: true },
            ],
            expert_agents: vec![
                AgentTemplate { name: "wzllama-expert-fast".into(), role: i18n.t("fleet.template.expert_fast"), model: "qwen2.5:1.5b".into(), num_ctx: 2048, temperature: 0.7, system_prompt: i18n.t("fleet.template.expert_fast_prompt"), enabled: true },
            ],
        },
    }
}

pub fn edit_agent(i18n: &I18n, agent: &mut AgentTemplate, index: usize, agent_type: &str) -> Result<()> {
    println!("\n{} {}/{} : {}", "─".repeat(40).dimmed(), index+1, agent_type, agent.role.bold());
    let items = vec![
        format!("{} {}", if agent.enabled { "✅" } else { "❌" }, i18n.t("fleet.edit.toggle")),
        i18n.t("fleet.edit.role"),
        i18n.t("fleet.edit.system_prompt"),
        i18n.t("fleet.edit.keep"),
    ];
    let sel = Select::new().with_prompt(i18n.t("fleet.edit.choose")).items(&items).default(3).interact()?;
    match sel {
        0 => agent.enabled = !agent.enabled,
        1 => { agent.role = Input::new().with_prompt(i18n.t("fleet.edit.new_role")).default(agent.role.clone()).interact()?; }
        2 => { agent.system_prompt = Input::new().with_prompt(i18n.t("fleet.edit.new_prompt")).default(agent.system_prompt.clone()).interact()?; }
        _ => {}
    }
    Ok(())
}