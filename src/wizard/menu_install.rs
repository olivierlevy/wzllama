use anyhow::Result;
use colored::*;
use dialoguer::{Select, Confirm};
use crate::config::{I18n, WzllamaState};
use crate::core::{shell, system};
use crate::tools;
use crate::display;

pub fn run(i18n: &I18n, state: &mut WzllamaState) -> Result<()> {
    loop {
        display::section(&i18n.t("install.title"));

        let mut items = vec![];
        
        // Docker
        let docker_ok = tools::docker::is_installed();
        let docker_running = docker_ok && tools::docker::is_running();
        items.push(format!("{} Docker {}", if docker_ok { "✅" } else { "❌" }, if docker_running { "(actif)" } else { "" }));

        // Open WebUI
        let webui_ok = shell::run("sudo docker ps --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok();
        items.push(format!("{} Open WebUI {}", if webui_ok { "✅" } else { "❌" }, if webui_ok { "http://localhost:3000" } else { "" }));

        // Ollama
        items.push(format!("{} Ollama", if shell::is_installed("ollama") { "✅" } else { "❌" }));

        // Tous les outils
        for tool in tools::get_available_tools(state) {
            items.push(format!("{} {}", if tool.installed { "✅" } else { "❌" }, tool.name));
        }

        items.push(i18n.t("menu.back"));

        let sel = Select::new().with_prompt(i18n.t("install.manage")).items(&items).default(0).interact()?;

        if sel == items.len() - 1 { return Ok(()); }

        match sel {
            0 => {
                if !docker_ok {
                    if Confirm::new().with_prompt(i18n.t("install.docker.confirm")).default(true).interact()? {
                        tools::docker::install_linux(&system::detect_package_manager())?;
                        state.installed.docker = true;
                        crate::config::state::save(state)?;
                    }
                } else if !docker_running {
                    tools::docker::start()?;
                }
            }
            1 if !webui_ok => {
                if let Some(t) = tools::get_tool("open_webui") { t.install()?; state.installed.open_webui = true; crate::config::state::save(state)?; }
            }
            2 if !shell::is_installed("ollama") => {
                if let Some(t) = tools::get_tool("ollama") { t.install()?; state.installed.ollama = true; crate::config::state::save(state)?; }
            }
            _ => {
                // Outils (index décalé de 3)
                let tools_list = tools::get_available_tools(state);
                if let Some(tool_info) = tools_list.get(sel - 3) {
                    if !tool_info.installed {
                        if let Some(t) = tools::get_tool(&tool_info.id) { t.install()?; crate::config::state::mark_installed(&tool_info.id, state); }
                    }
                }
            }
        }
    }
}