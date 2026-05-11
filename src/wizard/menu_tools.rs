use anyhow::Result;
use colored::*;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::tools::{self, docker};
use crate::display;

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    loop {
        let tools = tools::get_available_tools(state, i18n);
        let mut items: Vec<String> = tools.iter().map(|t| {
            let icon = if t.installed { "✅" } else { "📦" };
            format!("{} {} - {}", icon, t.name, t.description.dimmed())
        }).collect();
        items.push(i18n.t("menu.back"));

        let sel = Select::new()
            .with_prompt(i18n.t("menu.tools.choose"))
            .items(&items)
            .default(0)
            .interact()?;

        if sel == tools.len() { return Ok(()); }

        let tool_info = &tools[sel];
        let tool = match tools::get_tool(&tool_info.id) {
            Some(t) => t,
            None => continue,
        };

        if tool.requires_docker() {
            if let Err(_) = docker::ensure_ready(i18n) {
                continue;
            }
        }

        if tool.requires_docker() {
            if let Err(_) = docker::ensure_ready(i18n) {
                continue;
            }
        }
        if tool_info.installed {
            // Lancer l'outil
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

            tool.install()?;
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