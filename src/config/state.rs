#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::paths;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct InstalledTools {
    #[serde(default)]
    pub docker: bool,
    #[serde(default)]
    pub ollama: bool,
    #[serde(default)]
    pub open_webui: bool,
    #[serde(default)]
    pub openclaw: bool,
    #[serde(default)]
    pub claude_code: bool,
    #[serde(default)]
    pub hermes_agent: bool,
    #[serde(default)]
    pub opencode: bool,
    #[serde(default)]
    pub codex: bool,
    #[serde(default)]
    pub copilot_cli: bool,
    #[serde(default)]
    pub droid: bool,
    #[serde(default)]
    pub pi: bool,
    #[serde(default)]
    pub pool: bool,
    #[serde(default)]
    pub obsidian: bool,
    #[serde(default)]
    pub goose: bool,
    #[serde(default)]
    pub llmfit: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct FleetState {
    pub profile: String,
    pub orchestrator: String,
    pub agents: Vec<String>,
    #[serde(default)]
    pub openclaw_installed: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct WzllamaState {
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub installed: InstalledTools,
    #[serde(default)]
    pub fleets: HashMap<String, FleetState>,
    #[serde(default)]
    pub last_model: Option<String>,
    #[serde(default)]
    pub last_usage: Option<String>,
    #[serde(default)]
    pub last_tool: Option<String>,
    #[serde(default)]
    pub last_fleet: Option<String>,
}

pub fn load() -> WzllamaState {
    let path = paths::state_file();
    if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_default())
            .unwrap_or_default()
    } else {
        WzllamaState::default()
    }
}

pub fn save(state: &WzllamaState) -> Result<()> {
    std::fs::write(paths::state_file(), serde_json::to_string_pretty(state)?)?;
    Ok(())
}

pub fn mark_installed(tool: &str, state: &mut WzllamaState) {
    match tool {
        "docker" => state.installed.docker = true,
        "ollama" => state.installed.ollama = true,
        "open_webui" => state.installed.open_webui = true,
        "openclaw" => state.installed.openclaw = true,
        "claude_code" => state.installed.claude_code = true,
        "hermes_agent" => state.installed.hermes_agent = true,
        "opencode" => state.installed.opencode = true,
        "codex" => state.installed.codex = true,
        "copilot_cli" => state.installed.copilot_cli = true,
        "droid" => state.installed.droid = true,
        "pi" => state.installed.pi = true,
        "pool" => state.installed.pool = true,
        "obsidian" => state.installed.obsidian = true,
        "goose" => state.installed.goose = true,
        "llmfit" => state.installed.llmfit = true,
        _ => {}
    }
    let _ = save(state);
}

pub fn set_language(lang: &str, state: &mut WzllamaState) {
    state.language = Some(lang.to_string());
    let _ = save(state);
}

pub fn set_last_tool(tool: &str, state: &mut WzllamaState) {
    state.last_tool = Some(tool.to_string());
    let _ = save(state);
}

pub fn set_last_fleet(fleet: &str, state: &mut WzllamaState) {
    state.last_fleet = Some(fleet.to_string());
    let _ = save(state);
}

pub fn set_last_model(model: &str, state: &mut WzllamaState) {
    state.last_model = Some(model.to_string());
    let _ = save(state);
}

pub fn set_last_usage(usage: &str, state: &mut WzllamaState) {
    state.last_usage = Some(usage.to_string());
    let _ = save(state);
}

pub fn load_language() -> String {
    // Priority: WZLLAMA_LANG env var > state > system detection > fr
    if let Ok(lang) = std::env::var("WZLLAMA_LANG") {
        return lang;
    }
    
    let state = load();
    state.language.unwrap_or_else(|| "fr".into())
}

impl WzllamaState {
    pub fn load() -> Self {
        load()
    }
    
    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        save(self)
    }
    
    pub fn set_last_model(&mut self, model: &str) {
        set_last_model(model, self);
    }
    
    pub fn set_last_usage(&mut self, usage: &str) {
        set_last_usage(usage, self);
    }
    
    pub fn set_last_tool(&mut self, tool: &str) {
        set_last_tool(tool, self);
    }
    
    pub fn set_last_fleet(&mut self, fleet: &str) {
        set_last_fleet(fleet, self);
    }
}
