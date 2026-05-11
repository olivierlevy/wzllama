use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, Input};
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenClawTool;

impl Tool for OpenClawTool {
    fn id(&self) -> &str { "openclaw" }
    fn name(&self) -> &str { "OpenClaw" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.openclaw.description") }
    fn supports_fleets(&self) -> bool { true }
    fn status(&self) -> ToolStatus {
        if shell::is_installed("openclaw") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self) -> Result<()> {
        shell::run_live("npm install -g openclaw")?;
        Ok(())
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                //FIXME essayer openclaw --profile si ça ne marche pas via ollama
                display::comment(&i18n.t_with_vars("tool.openclaw.run_profile", &[("profile", &m)]));
                display::comment(&format!("openclaw --profile {}", m));
                display::run(&i18n.t_with_vars("tool.openclaw.run_model", &[("model", &m)]));
                let cmd: String = format!("ollama launch openclaw --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::comment(&i18n.t("tool.openclaw.no_model"));
                println!("ollama launch openclaw");
            }
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
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.openclaw.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run("openclaw uninstall --all --yes --non-interactive 2>/dev/null");
        let _ = shell::run("sudo npm uninstall -g openclaw 2>/dev/null");
        // Nettoyer les résidus
        let _ = shell::run("rm -f ~/.local/bin/openclaw 2>/dev/null");
        let _ = shell::run("rm -rf ~/.openclaw* 2>/dev/null");
        let _ = shell::run("systemctl --user disable openclaw-gateway-* 2>/dev/null");
        display::success(&i18n.t("tool.openclaw.uninstalled"));
        Ok(())
    }
}