use anyhow::Result;
use colored::*;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::tools;
use crate::wizard::menu_fleets;

pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    loop {
        let tools = tools::get_available_tools(state);
        let mut items: Vec<String> = tools.iter().map(|t| {
            let s = if t.installed { "✅" } else { "📦" };
            format!("{} {} - {}", s, t.name, t.description.dimmed())
        }).collect();
        items.push(i18n.t("menu.back"));

        let sel = Select::new().with_prompt(i18n.t("menu.tools.choose")).items(&items).default(0).interact()?;
        if sel == tools.len() { return Ok(()); }

        let tool = &tools[sel];
        
        if !tool.installed {
            // Installer d'abord
            if let Some(t) = tools::get_tool(&tool.id) {
                if let tools::tool_trait::ToolStatus::NotInstalled { ref install_cmd } = t.status() {
                    println!("   📥 {}", install_cmd);
                    t.install()?;
                    crate::config::state::mark_installed(&tool.id, state);
                }
            }
        }

        // Si l'outil supporte les flottes, rediriger vers le menu flottes
        if tool.supports_fleets {
            menu_fleets::run(i18n, state, hw)?;
            return Ok(());
        }

        // Lancer l'outil
        if let Some(t) = tools::get_tool(&tool.id) {
            let model = state.last_model.as_deref();
            t.launch(state, model, None)?;
            state.set_last_tool(&tool.id);
        }
    }
}