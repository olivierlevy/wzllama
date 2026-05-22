//! Dynamic menu builder that constructs MenuTree from wzllama state
//!
//! This module provides runtime menu construction based on:
//! - Installed tools
//! - Available models
//! - Hardware capabilities
//! - User preferences

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::tools;
use crate::menu_api::menu_tree::MenuMetadata;
use crate::menu_api::{MenuTree, MenuItem};

/// Builds a dynamic main menu from current state
pub fn build_main_menu(i18n: &I18n, state: &WzllamaState, _hw: &HardwareInfo) -> Result<MenuTree> {
    let mut items = vec![];
    
    // Resume option (if we have both last_tool and last_model)
    let has_resume = state.last_tool.is_some() && state.last_model.is_some();
    if has_resume {
        if let Some(ref last_tool) = state.last_tool {
            if let Some(tool) = tools::get_tool(last_tool) {
                items.push(MenuItem::leaf(&format!("▶ Reprendre {}", tool.name()))
                    .with_action("resume_last"));
            }
        }
    }
    
    // Main menu items
    items.push(MenuItem::leaf(&i18n.t("menu.main.wizard")).with_action("wizard"));
    items.push(MenuItem::leaf(&i18n.t("menu.main.models")).with_action("models"));
    items.push(MenuItem::leaf(&i18n.t("menu.main.scientific")).with_action("scientific"));
    items.push(MenuItem::leaf(&i18n.t("menu.main.tools")).with_action("tools"));
    items.push(MenuItem::leaf(&i18n.t("menu.main.cleanup")).with_action("cleanup"));
    items.push(MenuItem::leaf(&i18n.t("menu.main.config")).with_action("config"));
    items.push(MenuItem::leaf(&i18n.t("menu.main.language")).with_action("language"));
    items.push(MenuItem::leaf(&i18n.t("menu.main.quit")).with_action("quit"));
    
    let root = MenuItem::branch(&i18n.t("menu.main.title"))
        .add_submenus(items);
    
    Ok(MenuTree {
        root,
        metadata: MenuMetadata {
            title: Some(i18n.t("menu.main.title").to_string()),
            description: Some("Main menu for wzllama".to_string()),
            version: Some("1.0".to_string()),
        },
    })
}

/// Builds the wizard submenu with use cases
pub fn build_wizard_menu(i18n: &I18n) -> MenuTree {
    let items = vec![
        MenuItem::leaf(&i18n.t("wizard.usecase.general")).with_action("wizard_usecase_general"),
        MenuItem::leaf(&i18n.t("wizard.usecase.coding")).with_action("wizard_usecase_coding"),
        MenuItem::leaf(&i18n.t("wizard.usecase.reasoning")).with_action("wizard_usecase_reasoning"),
        MenuItem::leaf(&i18n.t("wizard.usecase.chat")).with_action("wizard_usecase_chat"),
        MenuItem::leaf(&i18n.t("wizard.usecase.multimodal")).with_action("wizard_usecase_multimodal"),
        MenuItem::leaf(&i18n.t("wizard.usecase.embedding")).with_action("wizard_usecase_embedding"),
    ];
    
    let root = MenuItem::branch(&i18n.t("wizard.title"))
        .add_submenu(MenuItem::branch(&i18n.t("wizard.subtitle")).add_submenus(items));
    
    MenuTree::new(&i18n.t("wizard.title"))
        .with_root(root)
}

/// Builds the models menu with local installed models
pub fn build_models_menu(i18n: &I18n, state: &WzllamaState) -> MenuTree {
    let local_models = crate::core::ollama_api::get_models();
    let last_model = state.last_model.clone();
    
    let mut items = vec![];
    
    for model in local_models {
        let default_marker = if Some(model.name.clone()) == last_model { " (default)" } else { "" };
        let label = format!("{} [{}]{}", model.name, model.details.as_ref().map(|d| d.parameter_size.as_deref().unwrap_or("")).unwrap_or(""), default_marker);
        items.push(MenuItem::leaf(&label).with_action(&format!("select_model_{}", model.name)));
    }
    
    // Add "Browse by organization" option
    items.push(MenuItem::leaf(&i18n.t("models.browse_by_org")).with_action("browse_models_org"));
    
    let root = MenuItem::branch(&i18n.t("menu.main.models"))
        .add_submenus(items);
    
    MenuTree {
        root,
        metadata: MenuMetadata {
            title: Some(i18n.t("menu.main.models").to_string()),
            description: Some("Choose an AI model".to_string()),
            version: Some("1.0".to_string()),
        },
    }
}