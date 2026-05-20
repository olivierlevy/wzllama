use anyhow::Result;
use colored::*;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::core::shell;
use crate::tools::{self, docker, tool_trait::ToolStatus, open_webui::OpenWebUITool};
use crate::display;
use super::menu_header;

fn sync_tools_state(state: &mut WzllamaState) {
    state.installed.docker = docker::is_installed();
    state.installed.ollama = shell::is_installed_quiet("ollama");
    state.installed.open_webui = shell::run_quiet("docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui || sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok();
    state.installed.openclaw = shell::is_installed_quiet("openclaw");
    state.installed.claude_code = shell::is_installed_quiet("claude");
    state.installed.hermes_agent = shell::is_installed_quiet("hermes");
    state.installed.opencode = shell::is_installed_quiet("opencode");
    state.installed.codex = shell::is_installed_with_local_bin("codex");
    state.installed.droid = shell::is_installed_quiet("droid");
    state.installed.pi = shell::is_installed_with_local_bin("pi");
    state.installed.pool = shell::is_installed_quiet("pool");
    
    // Obsidian - check flatpak first, then binary
    state.installed.obsidian = if shell::run("flatpak --version").is_ok() {
        shell::run_quiet("flatpak info md.obsidian.Obsidian").is_ok()
    } else {
        shell::is_installed_quiet("obsidian") || std::path::Path::new("/app/bin/obsidian").exists()
    };
}

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    // Synchroniser l'état réel des outils avec Docker
    sync_tools_state(state);
    crate::config::state::save(state)?;
    
    loop {
        // Affiche le header avec ressources comme le menu principal
        menu_header::render(
            i18n, 
            "menu.main.tools", 
            true,
            state.last_model.as_deref(),
            hw.ram_gb, 
            hw.total_vram_mb as f64 / 1024.0
        );
        
        let tools = tools::get_available_tools(state, i18n);
        let mut items: Vec<String> = tools.iter().map(|t| {
            let tool_dyn = tools::get_tool(&t.id);
            let supports_agentic = tool_dyn.as_ref().map(|x| x.supports_agentic()).unwrap_or(false);
            let icon = if supports_agentic { "🤖" } else if t.installed { "✅" } else { "📦" };
            let agentic_tag = if supports_agentic { " [agentic]".to_string() } else { String::new() };
            format!("{} {} - {}{}", icon, t.name, t.description.dimmed(), agentic_tag)
        }).collect();
        items.push(i18n.t("menu.back"));

        let max_items = display::menu_max_items(items.len(), 10);
        let sel = Select::new()
            .with_prompt(i18n.t("menu.tools.choose"))
            .items(&items)
            .max_length(max_items)
            .default(0)
            .interact_opt();

        // Handle Escape (Interrupted)
        let sel = match sel {
            Ok(Some(s)) => s,
            Ok(None) | Err(_) => {
                // Echap pressed - return to parent menu (or quit if main)
                return Ok(());
            }
        };

        if sel == tools.len() { return Ok(()); }

        let tool_info = &tools[sel];
        let tool = match tools::get_tool(&tool_info.id) {
            Some(t) => t,
            None => continue,
        };

        if tool.requires_docker() {
            docker::ensure_ready(i18n)?;
            // Réévaluer le statut après le démarrage de Docker
            let current_status = tool.status(state);
            // Si l'outil n'était pas marqué comme installé mais le conteneur existe
            if !tool_info.installed && current_status == ToolStatus::Installed {
                // L'outil est maintenant installé, le lancer
                state.set_last_tool(&tool_info.id);
                crate::config::state::save(state)?;
                let model = state.last_model.as_deref();
                tool.launch(i18n, state, model)?;
                continue;
            }
        }
        
        if tool_info.installed {
            // Pour Open WebUI, proposer mise à jour
            if tool_info.id == "open_webui" {
                let items = vec![
                    i18n.t("menu.tools.launch"),
                    i18n.t("tool.openwebui.update"),
                ];
                let sel = Select::new()
                    .with_prompt(i18n.t("menu.tools.choose"))
                    .items(&items)
                    .default(0)
                    .interact_opt()?;
                
                match sel {
                    Some(0) => {
                        state.set_last_tool(&tool_info.id); // Avant launch!
                        crate::config::state::save(state)?;  // Sauvegarder avant l'exec
                        let model = state.last_model.as_deref();
                        tool.launch(i18n, state, model)?;
                    }
                    Some(1) => {
                        OpenWebUITool::update(i18n)?;
                    }
                    _ => {} // Escape or cancel
                }
                continue;
            }
            
            if tool.supports_fleets() {
                crate::wizard::menu_fleets::run(i18n, state, hw)?;
                return Ok(());
            }

            // Sauvegarder l'outil AVANT launch (car exec remplace le processus)
            state.set_last_tool(&tool_info.id);
            crate::config::state::save(state)?;  // Sauvegarder avant l'exec
            let model = state.last_model.as_deref();
            tool.launch(i18n, state, model)?;
        } else {
            // Installer
            println!("   📥 {}", i18n.t("install.run_command"));

            tool.install(i18n)?;
            display::success(&i18n.t("install.completed"));
            crate::config::state::mark_installed(&tool_info.id, state);
            *state = crate::config::state::load();

            // Sauvegarder l'outil nouvellement installé
            state.set_last_tool(&tool_info.id);
            crate::config::state::save(state)?;

            println!("\n   {}", i18n.t("install.launch_first_time").dimmed());
            let model = state.last_model.as_deref();
            tool.launch(i18n, state, model)?;
            println!("\n   {}", i18n.t("install.relaunch_wzllama").bold());
        }
    }
}