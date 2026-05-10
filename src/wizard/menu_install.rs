use anyhow::Result;
use colored::*;
use dialoguer::{Select, Confirm};
use crate::config::{I18n, WzllamaState};
use crate::core::{shell, system};
use crate::tools;
use crate::display;

pub fn run(i18n: &I18n, state: &mut WzllamaState) -> Result<()> {
    let mut state = WzllamaState::load();
    // Synchroniser l'état avec la réalité
    sync_state_with_reality(&mut state);
    loop {
        display::section(&i18n.t("install.title"));

        let mut items: Vec<(String, String, bool, Option<String>)> = vec![]; // (id, label, installed, url)

        // Docker
        let docker_ok = tools::docker::is_installed();
        let docker_running = docker_ok && tools::docker::is_running();
        items.push(("docker".into(), format!("Docker {}", if docker_running { "(actif)" } else { "" }), docker_ok, None));

        // Outils du registry (tous, y compris ollama et open_webui)
        for tool in tools::get_available_tools(&state, &i18n) {
            let label = if tool.id == "open_webui" && tool.installed {
                format!("{} (http://localhost:3000)", tool.name)
            } else {
                tool.name.clone()
            };
            items.push((tool.id.clone(), label, tool.installed, None));
        }

        let display_items: Vec<String> = items.iter().map(|(_, label, installed, _)| {
            format!("{} {}", if *installed { "✅" } else { "❌" }, label)
        }).collect();
        let mut all_items = display_items.clone();
        all_items.push(i18n.t("menu.back"));

        let sel = Select::new()
            .with_prompt(i18n.t("install.manage"))
            .items(&all_items)
            .default(0)
            .interact()?;

        if sel == items.len() { return Ok(()); }

        let (id, _, installed, _) = &items[sel];

        match id.as_str() {
            "docker" if !*installed => {
                if Confirm::new().with_prompt(i18n.t("install.docker.confirm")).default(true).interact()? {
                    tools::docker::install_linux(&system::detect_package_manager())?;
                    state.installed.docker = true;
                    crate::config::state::save(&state)?;
                }
            }
            "docker" if !docker_running => {
                tools::docker::start()?;
            }
            _ if !*installed => {
                if let Some(t) = tools::get_tool(id) {
                    println!("   📥 {}", i18n.t("install.run_command"));
                    t.install()?;  // Affiche env vars + commande
                    
                    if Confirm::new()
                        .with_prompt(i18n.t("install.execute_now"))
                        .default(true)
                        .interact()?
                    {
                        if let tools::tool_trait::ToolStatus::NotInstalled { ref install_cmd } = t.status() {
                            shell::run(install_cmd)?;
                            display::success(&i18n.t("install.completed"));
                            crate::config::state::mark_installed(t.id(), &mut state);
                            
                            // Afficher les commandes de lancement
                            println!("\n   {}", i18n.t("install.launch_first_time").dimmed());
                            t.launch(i18n, &state, None, None)?;
                            println!("\n   {}", i18n.t("install.relaunch_wzllama").bold());
                        }
                    }
                }
            }
            _ => {
                display::info(&format!("{} déjà installé", id));
            }
        }
    }
}



fn sync_state_with_reality(state: &mut WzllamaState) {    
    state.installed.docker = shell::is_installed("docker");
    state.installed.ollama = shell::is_installed("ollama");
    state.installed.open_webui = shell::run(
        "sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui"
    ).is_ok();
    state.installed.openclaw = shell::is_installed("openclaw");
    state.installed.claude_code = shell::is_installed("claude");
    state.installed.hermes_agent = shell::is_installed("hermes");
    state.installed.opencode = shell::is_installed("opencode");
    state.installed.codex = shell::is_installed("codex");
    state.installed.copilot_cli = shell::is_installed("gh");
    state.installed.droid = shell::is_installed("droid");
    state.installed.pi = shell::is_installed("pi");
    state.installed.pool = shell::is_installed("pool");
    
    let _ = crate::config::state::save(state);
}