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
        println!("   {}", i18n.t("install.installed_header"));
        
        let mut installed_items: Vec<String> = vec![];
        let mut available_items: Vec<(String, String)> = vec![]; // (id, label)

        // Docker
        let docker_ok = tools::docker::is_installed();
        let docker_running = docker_ok && tools::docker::is_running();
        let docker_label = format!("Docker {}", if docker_running { "(actif)" } else { "" });
        if docker_ok {
            installed_items.push(format!("✅ {}", docker_label));
        } else {
            available_items.push(("docker".into(), format!("📦 {}", docker_label)));
        }

        // Tous les outils du registry
        for tool in tools::get_available_tools(state, i18n) {
            let label = if tool.id == "open_webui" && tool.installed {
                format!("Open WebUI (http://localhost:3000)")
            } else {
                tool.name.clone()
            };
            if tool.installed {
                installed_items.push(format!("✅ {}", label));
            } else {
                available_items.push((tool.id.clone(), format!("📦 {}", label)));
            }
        }

        // Afficher les installés (non sélectionnables)
        for item in &installed_items {
            println!("   {}", item.dimmed());
        }

        if available_items.is_empty() {
            display::info(&i18n.t("install.all_installed"));
        } else {
            println!("\n   {}", i18n.t("install.available_header"));
            let display_items: Vec<String> = available_items.iter().map(|(_, label)| label.clone()).collect();
            let mut all_items = display_items.clone();
            all_items.push(i18n.t("menu.back"));

            let sel = Select::new()
                .with_prompt(i18n.t("install.choose_to_install"))
                .items(&all_items)
                .default(0)
                .interact()?;

            if sel == available_items.len() {
                return Ok(());
            }

            let (id, _) = &available_items[sel];

            // Installer l'outil sélectionné
            match id.as_str() {
                "docker" => {
                    if Confirm::new().with_prompt(i18n.t("install.docker.confirm")).default(true).interact()? {
                        tools::docker::install_linux(&system::detect_package_manager())?;
                        state.installed.docker = true;
                        crate::config::state::save(state)?;
                    }
                }
                _ => {
                    if let Some(t) = tools::get_tool(id) {
                        println!("   📥 {}", i18n.t("install.run_command"));
                        t.install()?; // Affiche la commande
                        if Confirm::new()
                            .with_prompt(i18n.t("install.execute_now"))
                            .default(true)
                            .interact()?
                        {
                            if let tools::tool_trait::ToolStatus::NotInstalled { ref install_cmd } = t.status() {
                                shell::run_live(install_cmd)?;
                                display::success(&i18n.t("install.completed"));
                                crate::config::state::mark_installed(id, state);
                                println!("\n   {}", i18n.t("install.launch_first_time").dimmed());
                                t.launch(i18n, state, None, None)?;
                                println!("\n   {}", i18n.t("install.relaunch_wzllama").bold());
                            }
                        }
                    }
                }
            }
        }

        // Option pour revenir
        if available_items.is_empty() {
            let items = vec![i18n.t("menu.back")];
            let sel = Select::new()
                .with_prompt("")
                .items(&items)
                .default(0)
                .interact()?;
            if sel == 0 { return Ok(()); }
        }
    }
}