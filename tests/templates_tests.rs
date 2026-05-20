use wzllama::config::templates::{self, UsageParams, UsageSpec, UsagesConfig};
use std::collections::HashMap;

#[test]
fn test_usage_params_creation() {
    let params = UsageParams {
        r#type: "book".into(),
        pages_per_chunk: Some(20),
        loc_per_chunk: None,
        context_tokens: Some(8192),
        max_tokens_per_call: None,
    };
    
    assert_eq!(params.r#type, "book");
    assert_eq!(params.pages_per_chunk, Some(20));
    assert_eq!(params.loc_per_chunk, None);
    assert_eq!(params.context_tokens, Some(8192));
}

#[test]
fn test_usage_spec_creation() {
    let mut weights = HashMap::new();
    weights.insert("default".into(), 0.7);
    
    let spec = UsageSpec {
        i18n_key: "usage.test.label".into(),
        description_key: Some("usage.test.description".into()),
        weights,
        params: UsageParams {
            r#type: "test".into(),
            pages_per_chunk: None,
            loc_per_chunk: None,
            context_tokens: Some(4096),
            max_tokens_per_call: None,
        },
    };
    
    assert_eq!(spec.i18n_key, "usage.test.label");
    assert_eq!(spec.description_key, Some("usage.test.description".into()));
    assert!(!spec.weights.is_empty());
}

#[test]
fn test_usages_config_default() {
    let config = UsagesConfig { usages: HashMap::new() };
    assert!(config.usages.is_empty());
}

#[test]
fn test_default_usages_structure() {
    let config = templates::load_usages();
    
    // Vérifie que les usages par défaut existent
    assert!(config.usages.contains_key("big_book"));
    assert!(config.usages.contains_key("big_code"));
    assert!(config.usages.contains_key("fast_agents"));
    assert!(config.usages.contains_key("mixed"));
    
    // Vérifie les propriétés de big_book
    let big_book = config.usages.get("big_book").unwrap();
    assert_eq!(big_book.i18n_key, "usage.big_book.label");
    assert_eq!(big_book.params.r#type, "book");
}

#[test]
fn test_default_usages_big_book() {
    let config = templates::load_usages();
    
    let big_book = config.usages.get("big_book").unwrap();
    assert_eq!(big_book.params.context_tokens, Some(8192));
    assert_eq!(big_book.params.pages_per_chunk, Some(20));
}

#[test]
fn test_default_usages_big_code() {
    let config = templates::load_usages();
    
    let big_code = config.usages.get("big_code").unwrap();
    assert_eq!(big_code.params.context_tokens, Some(4096));
    assert_eq!(big_code.params.loc_per_chunk, Some(500));
}

#[test]
fn test_default_usages_fast_agents() {
    let config = templates::load_usages();
    
    let fast_agents = config.usages.get("fast_agents").unwrap();
    assert_eq!(fast_agents.params.context_tokens, Some(2048));
    assert_eq!(fast_agents.params.max_tokens_per_call, Some(1024));
}

#[test]
fn test_default_usages_weights() {
    let config = templates::load_usages();
    
    // Vérifie que tous les usages ont un poids par défaut
    for spec in config.usages.values() {
        assert!(spec.weights.contains_key("default"));
    }
    
    let big_book = config.usages.get("big_book").unwrap();
    assert_eq!(big_book.weights.get("default"), Some(&0.7));
}

#[test]
fn test_usages_config_serialization() {
    let mut usages = HashMap::new();
    usages.insert("test_usage".into(), UsageSpec {
        i18n_key: "test.label".into(),
        description_key: None,
        weights: { let mut w = HashMap::new(); w.insert("default".into(), 0.5); w },
        params: UsageParams {
            r#type: "test".into(),
            pages_per_chunk: Some(10),
            loc_per_chunk: None,
            context_tokens: Some(2048),
            max_tokens_per_call: None,
        },
    });
    
    let config = UsagesConfig { usages };
    let yaml = serde_yaml::to_string(&config).unwrap();
    let deserialized: UsagesConfig = serde_yaml::from_str(&yaml).unwrap();
    
    assert!(deserialized.usages.contains_key("test_usage"));
}