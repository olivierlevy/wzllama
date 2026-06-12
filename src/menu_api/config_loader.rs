//! Configuration loaders for menu structures from external files

use anyhow::Result;
use std::path::Path;

use super::menu_tree::{MenuConfig, MenuTree};

/// Configuration file format with [menu] section
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct MenuFileConfig {
    menu: MenuConfig,
}

impl From<MenuFileConfig> for MenuConfig {
    fn from(file_config: MenuFileConfig) -> Self {
        file_config.menu
    }
}

/// Load menu configuration from a TOML file
pub fn load_from_toml<P: AsRef<Path>>(path: P) -> Result<MenuTree> {
    let content = std::fs::read_to_string(path.as_ref())?;
    let file_config: MenuFileConfig = toml::from_str(&content)?;
    let config: MenuConfig = file_config.into();
    Ok(MenuTree::from(config))
}

/// Load menu configuration from a JSON file
pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<MenuTree> {
    let content = std::fs::read_to_string(path.as_ref())?;
    let config: MenuConfig = serde_json::from_str(&content)?;
    Ok(MenuTree::from(config))
}

/// Build a menu tree from a MenuConfig
pub fn build_tree(config: MenuConfig) -> MenuTree {
    MenuTree::from(config)
}

/// Example menu configuration as TOML string (for documentation)
#[allow(dead_code)]
pub const EXAMPLE_TOML: &str = r#"
version = "1.0"
title = "wzllama Menu"
description = "Main navigation menu"

[[items]]
label = "Install"
children = [
    { label = "Ollama", action_id = "install_ollama" },
    { label = "Open WebUI", action_id = "install_open_webui" },
    { label = "Claude Code", action_id = "install_claude_code" },
]

[[items]]
label = "Launch"
children = [
    { label = "Ollama", action_id = "launch_ollama" },
    { label = "Chat", action_id = "launch_chat" },
]

[[items]]
label = "Models"
action_id = "manage_models"
"#;

/// Example menu configuration as JSON string
#[allow(dead_code)]
pub const EXAMPLE_JSON: &str = r#"
{
  "version": "1.0",
  "title": "wzllama Menu",
  "description": "Main navigation menu",
  "items": [
    {
      "label": "Install",
      "children": [
        { "label": "Ollama", "action_id": "install_ollama" },
        { "label": "Open WebUI", "action_id": "install_open_webui" }
      ]
    },
    {
      "label": "Launch",
      "children": [
        { "label": "Chat", "action_id": "launch_chat" }
      ]
    }
  ]
}
"#;
