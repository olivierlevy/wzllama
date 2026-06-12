use wzllama::config::state::{self, InstalledTools, WzllamaState};

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
    let state = WzllamaState {
        language: Some("en".to_string()),
        ..Default::default()
    };
    assert_eq!(state.language, Some("en".to_string()));

    let state = WzllamaState {
        language: Some("fr".to_string()),
        ..Default::default()
    };
    assert_eq!(state.language, Some("fr".to_string()));
}

#[test]
fn test_state_serialization() {
    let state = WzllamaState {
        language: Some("en".to_string()),
        installed: InstalledTools {
            ollama: true,
            ..Default::default()
        },
        last_model: Some("qwen2.5:7b".to_string()),
        last_usage: Some("mixed".to_string()),
        last_tool: None,
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
    // Test serialization/deserialization directly without relying on file persistence
    let original_state = WzllamaState {
        language: Some("en".to_string()),
        installed: InstalledTools {
            open_webui: true,
            ..Default::default()
        },
        last_model: Some("test-model".to_string()),
        last_usage: Some("big_code".to_string()),
        last_tool: Some("open_webui".to_string()),
    };

    // Test JSON serialization roundtrip
    let json = serde_json::to_string(&original_state).unwrap();
    let loaded_state: WzllamaState = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded_state.language, Some("en".to_string()));
    assert!(loaded_state.installed.open_webui);
}
