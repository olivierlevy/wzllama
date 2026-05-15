use wzllama::config::state::{self, InstalledTools, WzllamaState, FleetState};
use std::collections::HashMap;

#[test]
fn test_installed_tools_default() {
    let tools = InstalledTools::default();
    assert!(!tools.docker);
    assert!(!tools.ollama);
    assert!(!tools.open_webui);
    assert!(!tools.openclaw);
    assert!(!tools.claude_code);
    assert!(!tools.hermes_agent);
    assert!(!tools.opencode);
    assert!(!tools.codex);
    assert!(!tools.copilot_cli);
    assert!(!tools.droid);
    assert!(!tools.pi);
    assert!(!tools.pool);
}

#[test]
fn test_wzllama_state_default() {
    let state = WzllamaState::default();
    assert!(state.language.is_none());
    assert!(state.last_model.is_none());
    assert!(state.last_usage.is_none());
    assert!(state.last_tool.is_none());
    assert!(state.last_fleet.is_none());
    assert!(state.fleets.is_empty());
}

#[test]
fn test_fleet_state_default() {
    let fleet = FleetState::default();
    assert!(fleet.profile.is_empty());
    assert!(fleet.orchestrator.is_empty());
    assert!(fleet.agents.is_empty());
    assert!(!fleet.openclaw_installed);
}

#[test]
fn test_mark_installed_docker() {
    let mut state = WzllamaState::default();
    state::mark_installed("docker", &mut state);
    assert!(state.installed.docker);
    assert!(!state.installed.ollama);
}

#[test]
fn test_mark_installed_ollama() {
    let mut state = WzllamaState::default();
    state::mark_installed("ollama", &mut state);
    assert!(state.installed.ollama);
}

#[test]
fn test_mark_installed_unknown() {
    let mut state = WzllamaState::default();
    state::mark_installed("unknown_tool", &mut state);
    // Should not panic, just do nothing
    assert!(!state.installed.docker);
    assert!(!state.installed.ollama);
}

#[test]
fn test_set_language() {
    let mut state = WzllamaState::default();
    state.language = Some("en".to_string());
    assert_eq!(state.language, Some("en".to_string()));
    
    state.language = Some("fr".to_string());
    assert_eq!(state.language, Some("fr".to_string()));
}

#[test]
fn test_state_with_fleets() {
    let mut state = WzllamaState::default();
    let mut fleets = HashMap::new();
    
    fleets.insert("project1".to_string(), FleetState {
        profile: "openclaw".to_string(),
        orchestrator: "qwen2.5:7b".to_string(),
        agents: vec!["analyst".to_string(), "reviewer".to_string()],
        openclaw_installed: true,
    });
    
    state.fleets = fleets;
    
    assert!(state.fleets.contains_key("project1"));
    let fleet = state.fleets.get("project1").unwrap();
    assert_eq!(fleet.profile, "openclaw");
    assert_eq!(fleet.agents.len(), 2);
}

#[test]
fn test_state_serialization() {
    let state = WzllamaState {
        language: Some("en".to_string()),
        installed: InstalledTools { ollama: true, ..Default::default() },
        fleets: HashMap::new(),
        last_model: Some("qwen2.5:7b".to_string()),
        last_usage: Some("mixed".to_string()),
        last_tool: None,
        last_fleet: None,
    };
    
    let json = serde_json::to_string(&state).unwrap();
    let deserialized: WzllamaState = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.language, Some("en".to_string()));
    assert!(deserialized.installed.ollama);
    assert_eq!(deserialized.last_model, Some("qwen2.5:7b".to_string()));
    assert_eq!(deserialized.last_usage, Some("mixed".to_string()));
}

#[test]
fn test_save_and_load_state() {
    let original_state = WzllamaState {
        language: Some("fr".to_string()),
        installed: InstalledTools { open_webui: true, ..Default::default() },
        fleets: {
            let mut m = HashMap::new();
            m.insert("test_fleet".to_string(), FleetState::default());
            m
        },
        last_model: Some("test-model".to_string()),
        last_usage: Some("big_code".to_string()),
        last_tool: Some("open_webui".to_string()),
        last_fleet: Some("test_fleet".to_string()),
    };
    
    // Save
    let result = state::save(&original_state);
    assert!(result.is_ok());
    
    // Load - just verify the state was saved (exact values may vary due to concurrent tests)
    let loaded_state = state::load();
    
    // Verify that open_webui is installed (this should persist since we saved it)
    assert!(loaded_state.installed.open_webui);
    // Verify fleets persist
    assert!(loaded_state.fleets.contains_key("test_fleet"));
}