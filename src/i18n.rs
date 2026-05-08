use std::collections::HashMap;
use anyhow::{Result, Context};

pub struct I18n {
    map: HashMap<String, String>,
}

impl I18n {
    pub fn load(lang: &str) -> Result<Self> {
        let file_path = wzllama_dir().join("i18n").join(format!("{lang}.json"));
        let fallback = wzllama_dir().join("i18n").join("en.json");

        let content = if file_path.exists() {
            std::fs::read_to_string(&file_path)?
        } else {
            std::fs::read_to_string(&fallback)?
        };

        let map: HashMap<String, String> = serde_json::from_str(&content)
            .context(format!("Fichier i18n '{}' invalide", file_path.display()))?;

        Ok(Self { map })
    }

    pub fn t(&self, key: &str) -> &str {
        self.map.get(key).map(String::as_str).unwrap_or(key)
    }
}