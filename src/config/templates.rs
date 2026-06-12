use crate::config::paths;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// Types pour usages.yaml
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsageParams {
    pub r#type: String,
    #[serde(default)]
    pub pages_per_chunk: Option<u32>,
    #[serde(default)]
    pub loc_per_chunk: Option<u32>,
    #[serde(default)]
    pub context_tokens: Option<u32>,
    #[serde(default)]
    pub max_tokens_per_call: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsageSpec {
    pub i18n_key: String,
    #[serde(default)]
    pub description_key: Option<String>,
    pub weights: HashMap<String, f32>,
    pub params: UsageParams,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsagesConfig {
    pub usages: HashMap<String, UsageSpec>,
}

pub fn ensure() -> Result<()> {
    let user_cfg = paths::config_dir();
    let user_i18n = paths::i18n_dir();
    fs::create_dir_all(&user_cfg)?;
    fs::create_dir_all(&user_i18n)?;

    let exe_dir = std::env::current_exe()?
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let cfg_source = if exe_dir.join("config").exists() {
        exe_dir.join("config")
    } else {
        std::path::PathBuf::from("config")
    };

    for file in &["default_usages.yaml", "default_config.yaml"] {
        let src = cfg_source.join(file);
        let dest_name = file.strip_prefix("default_").unwrap_or(file);
        let dest = user_cfg.join(dest_name);
        if !dest.exists() && src.exists() {
            fs::copy(&src, &dest)?;
        }
    }

    let i18n_source = cfg_source.join("i18n");
    if i18n_source.exists() {
        for entry in fs::read_dir(&i18n_source)? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|e| e == "json") {
                let dest = user_i18n.join(entry.file_name());
                if !dest.exists() {
                    fs::copy(entry.path(), &dest)?;
                }
            }
        }
    }
    Ok(())
}

pub fn load_usages() -> UsagesConfig {
    let path = paths::config_dir().join("usages.yaml");
    match fs::read_to_string(&path) {
        Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|_| default_usages()),
        Err(_) => default_usages(),
    }
}

pub fn default_usages() -> UsagesConfig {
    let mut usages = HashMap::new();
    usages.insert(
        "big_book".into(),
        UsageSpec {
            i18n_key: "usage.big_book.label".into(),
            description_key: Some("usage.big_book.description".into()),
            weights: {
                let mut w = HashMap::new();
                w.insert("default".into(), 0.7);
                w
            },
            params: UsageParams {
                r#type: "book".into(),
                pages_per_chunk: Some(20),
                loc_per_chunk: None,
                context_tokens: Some(8192),
                max_tokens_per_call: None,
            },
        },
    );
    usages.insert(
        "big_code".into(),
        UsageSpec {
            i18n_key: "usage.big_code.label".into(),
            description_key: Some("usage.big_code.description".into()),
            weights: {
                let mut w = HashMap::new();
                w.insert("default".into(), 0.6);
                w
            },
            params: UsageParams {
                r#type: "code".into(),
                pages_per_chunk: None,
                loc_per_chunk: Some(500),
                context_tokens: Some(4096),
                max_tokens_per_call: None,
            },
        },
    );
    usages.insert(
        "fast_agents".into(),
        UsageSpec {
            i18n_key: "usage.fast_agents.label".into(),
            description_key: Some("usage.fast_agents.description".into()),
            weights: {
                let mut w = HashMap::new();
                w.insert("default".into(), 0.9);
                w
            },
            params: UsageParams {
                r#type: "agents".into(),
                pages_per_chunk: None,
                loc_per_chunk: None,
                context_tokens: Some(2048),
                max_tokens_per_call: Some(1024),
            },
        },
    );
    usages.insert(
        "mixed".into(),
        UsageSpec {
            i18n_key: "usage.mixed.label".into(),
            description_key: Some("usage.mixed.description".into()),
            weights: {
                let mut w = HashMap::new();
                w.insert("default".into(), 0.5);
                w
            },
            params: UsageParams {
                r#type: "mixed".into(),
                pages_per_chunk: None,
                loc_per_chunk: None,
                context_tokens: Some(4096),
                max_tokens_per_call: None,
            },
        },
    );
    UsagesConfig { usages }
}

pub fn validate_all() -> Result<()> {
    println!("Validation des templates...\n");
    let usages = load_usages();
    println!("✓ usages.yaml chargé ({} entrées)", usages.usages.len());
    let languages = crate::config::i18n::get_available_languages();
    for lang in &languages {
        match crate::config::i18n::load(&lang.code) {
            Ok(_) => println!("✓ i18n/{} ({})", lang.code, lang.name),
            Err(e) => println!("✗ i18n/{} : {}", lang.code, e),
        }
    }
    Ok(())
}

pub fn reset_all() -> Result<()> {
    let cfg = paths::config_dir();
    for file in &["usages.yaml", "config.yaml"] {
        let p = cfg.join(file);
        if p.exists() {
            fs::rename(&p, p.with_extension("yaml.bak"))?;
        }
    }
    for entry in fs::read_dir(paths::i18n_dir())? {
        let p = entry?.path();
        if p.extension().is_some_and(|e| e == "json") && p.exists() {
            fs::rename(&p, p.with_extension("json.bak"))?;
        }
    }
    ensure()
}
