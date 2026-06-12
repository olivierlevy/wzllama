#![allow(dead_code)]

use crate::config::paths;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{OnceLock, Arc};

// Arc-swap for atomic hot-swap of I18n
use arc_swap::ArcSwap;
use tokio::sync::watch;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LanguageMeta {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub name_en: Option<String>,
    #[serde(default = "default_direction")]
    pub direction: String,
}

fn default_direction() -> String {
    "ltr".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct I18nFile {
    #[serde(rename = "_language")]
    pub language: LanguageMeta,
    #[serde(flatten)]
    pub translations: HashMap<String, serde_json::Value>,
}

#[derive(Clone)]
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
                direction: "ltr".into(),
            },
            map: HashMap::new(),
        }
    }
}

impl I18n {
    pub fn t(&self, key: &str) -> String {
        self.map
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn t_with_vars(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let mut text = self.t(key);
        for (var, value) in vars {
            text = text.replace(&format!("{{{}}}", var), value);
        }
        text
    }
}

// Global hot-swappable I18n store and notification channel
static GLOBAL_I18N: OnceLock<ArcSwap<I18n>> = OnceLock::new();
static I18N_WATCH: OnceLock<watch::Sender<String>> = OnceLock::new();

/// Initialize global I18n store and watch channel (idempotent)
pub fn init_global() {
    // Load the language from state (or default)
    let lang = crate::config::state::load_language();
    let i = load(&lang).unwrap_or_default();
    GLOBAL_I18N.get_or_init(|| ArcSwap::from_pointee(i));
    I18N_WATCH.get_or_init(|| {
        let (tx, _rx) = watch::channel(lang);
        tx
    });
}

/// Get the current I18n as an Arc for cheap cloning
pub fn get_current() -> Arc<I18n> {
    let store = GLOBAL_I18N.get_or_init(|| {
        let lang = crate::config::state::load_language();
        let i = load(&lang).unwrap_or_default();
        ArcSwap::from_pointee(i)
    });
    store.load_full()
}

/// Subscribe to language change notifications. Returns a watch::Receiver<String> that yields the latest language code when changed.
pub fn subscribe() -> watch::Receiver<String> {
    let sender = I18N_WATCH.get_or_init(|| {
        let lang = crate::config::state::load_language();
        let (tx, _rx) = watch::channel(lang);
        tx
    });
    sender.subscribe()
}

/// Atomically reload the global I18n for `lang_code` and notify subscribers.
pub fn reload(lang_code: &str) -> Result<()> {
    let i = load(lang_code)?;
    if let Some(store) = GLOBAL_I18N.get() {
        store.store(Arc::new(i));
    } else {
        GLOBAL_I18N.set(ArcSwap::from_pointee(i)).ok();
    }

    if let Some(sender) = I18N_WATCH.get() {
        let _ = sender.send(lang_code.to_string());
    }

    Ok(())
}

pub fn get_available_languages() -> Vec<LanguageMeta> {
    let i18n_path = paths::i18n_dir();
    let mut languages = Vec::new();

    // Chercher dans ~/.wzllama/i18n d'abord
    if let Ok(entries) = std::fs::read_dir(&i18n_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "json") {
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
                if path.is_file() && path.extension().is_some_and(|e| e == "json") {
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
            code: "fr".into(),
            name: "Français".into(),
            name_en: Some("French".into()),
            direction: "ltr".into(),
        });
    }
    languages.sort_by(|a, b| a.code.cmp(&b.code));
    languages
}

pub fn detect_system_language() -> String {
    for var in &["LANG", "LANGUAGE", "LC_ALL", "LC_MESSAGES"] {
        if let Ok(lang) = std::env::var(var) {
            let code = lang
                .split('.')
                .next()
                .unwrap_or("fr")
                .split('_')
                .next()
                .unwrap_or("fr")
                .to_lowercase();
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
            meta: LanguageMeta {
                code: "fr".into(),
                name: "Français".into(),
                name_en: None,
                direction: "ltr".into(),
            },
            map: HashMap::new(),
        });
    };

    let file: I18nFile =
        serde_json::from_str(&content).context(format!("Fichier i18n '{}' invalide", lang_code))?;

    let mut map = HashMap::new();
    for (key, value) in &file.translations {
        let s = match value {
            serde_json::Value::String(s) => s.clone(),
            _ => value.to_string(),
        };
        map.insert(key.clone(), s);
    }

    Ok(I18n {
        meta: file.language,
        map,
    })
}

pub fn check_integrity() -> Result<()> {
    let languages = get_available_languages();
    let ref_lang = if languages.iter().any(|l| l.code == "fr") {
        "fr"
    } else {
        &languages[0].code
    };
    let reference = load(ref_lang)?;
    for lang in &languages {
        if lang.code == ref_lang {
            continue;
        }
        let i18n = load(&lang.code)?;
        let missing: Vec<_> = reference
            .map
            .keys()
            .filter(|k| !i18n.map.contains_key(*k))
            .collect();
        if !missing.is_empty() {
            println!("⚠️  {} : {} clés manquantes", lang.code, missing.len());
        }
    }
    Ok(())
}
