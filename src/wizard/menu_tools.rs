use anyhow::Result;
use colored::*;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::core::shell;
use crate::tools::{self, docker, tool_trait::ToolStatus, open_webui::OpenWebUITool};
use crate::display;

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
    
    // Import nécessaires pour le header
    use crate::core::{ollama_api, system};
    
    loop {
        // Affiche le header avec ressources comme le menu principal
        let ram_avail = system::get_available_ram_gb();
        let vram_avail = system::get_available_vram_gb();
        let running = ollama_api::get_running_models();
        display::clear_screen();
        display::header_with_resources(
            &i18n.t("menu.main.tools"),
            hw.ram_gb, ram_avail, 
            hw.total_vram_mb as f64 / 1024.0, vram_avail, 
            &running,
            state.last_model.as_deref()
        );
        
        let tools = tools::get_available_tools(state, i18n);
        let mut items: Vec<String> = tools.iter().map(|t| {
            let icon = if t.installed { "✅" } else { "📦" };
            let agentic = if t.supports_agentic { "🤖" } else { "" };
            let agentic_tag = if t.supports_agentic { " [agentic]".to_string() } else { String::new() };
            let desc = t.description.clone();
            format!("{} {}{} - {}{}", icon, agentic, t.name, desc.dimmed(), agentic_tag)
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
                let model = state.last_model.as_deref();
                tool.launch(i18n, state, model)?;
                state.set_last_tool(&tool_info.id);
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
                        let model = state.last_model.as_deref();
                        tool.launch(i18n, state, model)?;
                        state.set_last_tool(&tool_info.id);
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

            let model = state.last_model.as_deref();
            tool.launch(i18n, state, model)?;
            state.set_last_tool(&tool_info.id);
        } else {
            // Installer
            println!("   📥 {}", i18n.t("install.run_command"));

            tool.install(i18n)?;
            display::success(&i18n.t("install.completed"));
            crate::config::state::mark_installed(&tool_info.id, state);
            *state = crate::config::state::load();

            println!("\n   {}", i18n.t("install.launch_first_time").dimmed());
            let model = state.last_model.as_deref();
            tool.launch(i18n, state, model)?;
            println!("\n   {}", i18n.t("install.relaunch_wzllama").bold());
        }
    }
}