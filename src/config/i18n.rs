use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::paths;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LanguageMeta {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub name_en: Option<String>,
    #[serde(default = "default_direction")]
    pub direction: String,
}

fn default_direction() -> String { "ltr".to_string() }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct I18nFile {
    #[serde(rename = "_language")]
    pub language: LanguageMeta,
    #[serde(flatten)]
    pub translations: HashMap<String, serde_json::Value>,
}

pub struct I18n {
    #[allow(dead_code)]
    pub meta: LanguageMeta,
    #[allow(dead_code)]
    pub map: HashMap<String, String>,
}

impl Default for I18n {
    fn default() -> Self {
        Self {
            meta: LanguageMeta { 
                code: "en".into(), 
                name: "English".into(), 
                name_en: Some("English".into()), 
                direction: "ltr".into() 
            },
            map: HashMap::new(),
        }
    }
}

impl I18n {
    pub fn t(&self, key: &str) -> String {
        self.map.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    pub fn t_with_vars(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let mut text = self.t(key);
        for (var, value) in vars {
            text = text.replace(&format!("{{{}}}", var), value);
        }
        text
    }
}

pub fn get_available_languages() -> Vec<LanguageMeta> {
    let i18n_path = paths::i18n_dir();
    let mut languages = Vec::new();
    
    // Chercher dans ~/.wzllama/i18n d'abord
    if let Ok(entries) = std::fs::read_dir(&i18n_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(file) = serde_json::from_str::<I18nFile>(&content) {
                        languages.push(file.language);
                    }
                }
            }
        }
    }
    
    // Si pas trouvé, chercher dans le répertoire embarqué du projet (config/i18n)
    if languages.is_empty() {
        if let Ok(entries) = std::fs::read_dir("config/i18n") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(file) = serde_json::from_str::<I18nFile>(&content) {
                            languages.push(file.language);
                        }
                    }
                }
            }
        }
    }
    
    if languages.is_empty() {
        languages.push(LanguageMeta {
            code: "fr".into(), name: "Français".into(),
            name_en: Some("French".into()), direction: "ltr".into(),
        });
    }
    languages.sort_by(|a, b| a.code.cmp(&b.code));
    languages
}

pub fn detect_system_language() -> String {
    for var in &["LANG", "LANGUAGE", "LC_ALL", "LC_MESSAGES"] {
        if let Ok(lang) = std::env::var(var) {
            let code = lang.split('.').next().unwrap_or("fr")
                .split('_').next().unwrap_or("fr").to_lowercase();
            if paths::i18n_dir().join(format!("{}.json", code)).exists() {
                return code;
            }
        }
    }
    "fr".to_string()
}

pub fn load(lang_code: &str) -> Result<I18n> {
    let file_path = paths::i18n_dir().join(format!("{}.json", lang_code));
    let fallback_path = paths::i18n_dir().join("fr.json");
    let embedded_path = std::path::Path::new("config/i18n").join(format!("{}.json", lang_code));
    let embedded_fallback = std::path::Path::new("config/i18n").join("fr.json");

    let content = if file_path.exists() {
        std::fs::read_to_string(&file_path)?
    } else if embedded_path.exists() {
        std::fs::read_to_string(&embedded_path)?
    } else if fallback_path.exists() {
        std::fs::read_to_string(&fallback_path)?
    } else if embedded_fallback.exists() {
        std::fs::read_to_string(&embedded_fallback)?
    } else {
        return Ok(I18n {
            meta: LanguageMeta { code: "fr".into(), name: "Français".into(), name_en: None, direction: "ltr".into() },
            map: HashMap::new(),
        });
    };

    let file: I18nFile = serde_json::from_str(&content)
        .context(format!("Fichier i18n '{}' invalide", lang_code))?;

    let mut map = HashMap::new();
    for (key, value) in &file.translations {
        let s = match value {
            serde_json::Value::String(s) => s.clone(),
            _ => value.to_string(),
        };
        map.insert(key.clone(), s);
    }

    Ok(I18n { meta: file.language, map })
}

pub fn check_integrity() -> Result<()> {
    let languages = get_available_languages();
    let ref_lang = if languages.iter().any(|l| l.code == "fr") { "fr" } else { &languages[0].code };
    let reference = load(ref_lang)?;
    for lang in &languages {
        if lang.code == ref_lang { continue; }
        let i18n = load(&lang.code)?;
        let missing: Vec<_> = reference.map.keys().filter(|k| !i18n.map.contains_key(*k)).collect();
        if !missing.is_empty() {
            println!("⚠️  {} : {} clés manquantes", lang.code, missing.len());
        }
    }
    Ok(())
}