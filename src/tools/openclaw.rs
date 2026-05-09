use anyhow::Result;
use std::path::PathBuf;
use colored::*;
use dialoguer::{Confirm, Input};
use crate::config::{self, I18n, WzllamaState};
use crate::core::{shell, ollama_api, ollama_models};
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenClawTool;

impl Tool for OpenClawTool {
    fn id(&self) -> &str { "openclaw" }
    fn name(&self) -> &str { "OpenClaw" }
    fn description(&self) -> &str { "Assistant IA personnel avec 100+ skills" }
    fn supports_fleets(&self) -> bool { true }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("openclaw") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "npm install -g openclaw".into() } }
    }

    fn install(&self) -> Result<()> {
        shell::run("npm install -g openclaw")?;
        Ok(())
    }

    fn launch(&self, _state: &WzllamaState, _model: Option<&str>, fleet: Option<&str>) -> Result<()> {
        match fleet {
            Some(f) => { shell::run(&format!("openclaw --profile {}", f))?; }
            None => { shell::run("openclaw")?; }
        }
        Ok(())
    }
}

impl OpenClawTool {
    /// Génère le openclaw.json et configure le gateway pour une flotte
    pub fn create_fleet_config(
        i18n: &I18n,
        usage_type: &str,
        orchestrator_name: &str,
        agents: &[(String, String)], // (name, role)
    ) -> Result<String> {
        // Demander le nom du projet
        let project_name: String = Input::new()
            .with_prompt(i18n.t("fleet.project_name"))
            .default(usage_type.to_string())
            .interact()?;

        // Créer le dossier ~/.openclaw-{projet}/
        let openclaw_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(format!(".openclaw-{}", project_name));
        std::fs::create_dir_all(&openclaw_dir)?;

        // Construire le JSON des agents
        let mut agents_json = String::new();
        for (name, role) in agents {
            let id = name
                .strip_prefix("wzllama-reflexion-")
                .or(name.strip_prefix("wzllama-expert-"))
                .unwrap_or(name);
            agents_json.push_str(&format!(
                "      {{ \"id\": \"{}\", \"model\": {{ \"primary\": \"ollama/{}\" }}, \"identity\": {{ \"name\": \"{}\" }} }},\n",
                id, name, role
            ));
        }
        if agents_json.ends_with(",\n") {
            agents_json.pop(); agents_json.pop(); agents_json.push('\n');
        }

        let config = format!(r#"{{
  "gateway": {{ "mode": "local" }},
  "agents": {{
    "defaults": {{ "model": {{ "primary": "ollama/{}" }} }},
    "list": [
{}
    ]
  }}
}}"#, orchestrator_name, agents_json);

        std::fs::write(openclaw_dir.join("openclaw.json"), &config)?;

        // Installer le gateway systemd
        println!("\n🔧 {}", i18n.t("fleet.install_gateway"));
        println!("   openclaw --profile {} gateway install", project_name.cyan());
        
        if Confirm::new()
            .with_prompt(i18n.t("fleet.install_gateway_now"))
            .default(true)
            .interact()?
        {
            shell::run(&format!("openclaw --profile {} gateway install --force", project_name))?;
            println!("   {}", i18n.t("fleet.gateway_installed").green());
        }

        Ok(project_name)
    }
}