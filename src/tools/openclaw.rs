use anyhow::Result;
use colored::*;
use dialoguer::{Input};
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenClawTool;

impl Tool for OpenClawTool {
    fn id(&self) -> &str { "openclaw" }
    fn name(&self) -> &str { "OpenClaw" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.claude.description") }
    fn supports_fleets(&self) -> bool { true }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("openclaw") {
            ToolStatus::Installed
        } else {
            ToolStatus::NotInstalled { install_cmd: "npm install -g openclaw".into() }
        }
    }

    fn install(&self) -> Result<()> {
        shell::run("npm install -g openclaw")?;
        Ok(())
    }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>, fleet: Option<&str>) -> Result<()> {
        match fleet {
            Some(f) => println!("openclaw --profile {}", f),
            None => println!("openclaw"),
        }
        Ok(())
    }
}

// ─── Création de flotte ────────────────────────────

impl OpenClawTool {
    pub fn create_fleet(
        i18n: &I18n,
        usage_type: &str,
        orchestrator_name: &str,
        agents: &[(String, String, String, u32, f32, String)],
    ) -> Result<String> {
        let project_name: String = Input::new()
            .with_prompt(i18n.t("fleet.project_name"))
            .default(usage_type.to_string())
            .interact()?;

        println!("\n{}", "═".repeat(50).cyan());
        println!("{}", i18n.t("fleet.commands_to_run").bold());
        println!("{}", "═".repeat(50).cyan());
        println!();
        println!("   {}", i18n.t("fleet.copy_paste"));
        println!();

        println!("   # 0. Configurer Ollama comme provider");
        println!("   openclaw --profile {} config set plugins.entries.ollama.enabled true", project_name);
        println!("   openclaw --profile {} config set env.OLLAMA_API_KEY 'ollama-local'", project_name);
        println!();
        
        // 1. Gateway
        println!("   # {}", i18n.t("fleet.step_gateway"));
        println!("   openclaw --profile {} gateway install --force", project_name);
        println!("   openclaw --profile {} config set gateway.mode local", project_name);
        println!();
        
        // 2. Orchestrateur
        println!("   # {}", i18n.t("fleet.step_orch"));
        println!("   openclaw --profile {} config set agents.defaults.model.primary 'ollama/{}'", project_name, orchestrator_name);
        println!("   openclaw --profile {} config set agents.defaults.contextTokens 32768", project_name);
        println!();
        
        // 3. Agents
        let mut agents_json = String::from("[");
        for (name, role, model, ctx, _temp, _prompt) in agents {
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
        agents_json.pop(); // enlever la dernière virgule
        agents_json.push(']');

        println!("   # {}", i18n.t("fleet.step_agents"));
        println!("   openclaw --profile {} config set agents.list '{}' --json --merge", project_name, agents_json);
        println!();
        
        // 4. Validation
        println!("   # {}", i18n.t("fleet.step_validate"));
        println!("   openclaw --profile {} config set gateway.mode local", project_name);
        println!("   openclaw --profile {} doctor --fix", project_name);
        println!("   openclaw --profile {} gateway restart", project_name);
        println!();
        println!("   # {}", i18n.t("fleet.step_launch"));
        println!("   openclaw --profile {}", project_name);
        
        println!("\n{}", "═".repeat(50).cyan());

        Ok(project_name)
    }
}