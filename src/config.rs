use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Structure pour les métadonnées de langue
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

// Structure complète du fichier i18n
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct I18nFile {
    #[serde(rename = "_language")]
    pub language: LanguageMeta,
    // Le reste est dynamique
    #[serde(flatten)]
    pub translations: HashMap<String, serde_json::Value>,
}

// Type pour l'i18n utilisé dans l'application
pub type I18nMap = HashMap<String, String>;

// Structure pour stocker l'i18n complet avec métadonnées
pub struct I18n {
    pub meta: LanguageMeta,
    pub map: I18nMap,
}

impl I18n {
    pub fn t(&self, key: &str) -> String {
        self.map.get(key).cloned().unwrap_or_else(|| key.to_string())
    }
    
    pub fn t_with_vars(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let base_text = self.map.get(key).cloned().unwrap_or_else(|| key.to_string());
        let mut text = base_text;
        for (var, value) in vars {
            text = text.replace(&format!("{{{}}}", var), value);
        }
        text
    }
}

// Types pour usages.yaml (inchangés)
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

// Clés i18n obligatoires
pub const REQUIRED_I18N_KEYS: &[&str] = &[
    "app.title", "app.welcome", "app.goodbye",
    "menu.language.choice", "menu.main.title",
    "menu.usage.title", "system.detecting",
    "install.checking", "model.selection",
    "estimation.title",
];

// Chemins
pub fn wzllama_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".wzllama")
}

pub fn config_dir() -> PathBuf {
    wzllama_dir().join("config")
}

pub fn i18n_dir() -> PathBuf {
    wzllama_dir().join("i18n")
}

pub fn log_dir() -> PathBuf {
    wzllama_dir().join("logs")
}

// Scanner les langues disponibles
pub fn get_available_languages() -> Vec<LanguageMeta> {
    let i18n_path = i18n_dir();
    let mut languages = Vec::new();

    if let Ok(entries) = fs::read_dir(&i18n_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "json" {
                        if let Ok(content) = fs::read_to_string(&path) {
                            // Essayer de parser le fichier pour extraire les métadonnées
                            if let Ok(i18n_file) = serde_json::from_str::<I18nFile>(&content) {
                                languages.push(i18n_file.language);
                            }
                        }
                    }
                }
            }
        }
    }

    // Si aucune langue trouvée, ajouter l'anglais par défaut
    if languages.is_empty() {
        languages.push(LanguageMeta {
            code: "en".to_string(),
            name: "English".to_string(),
            name_en: Some("English".to_string()),
            direction: "ltr".to_string(),
        });
    }

    // Trier par code de langue
    languages.sort_by(|a, b| a.code.cmp(&b.code));
    languages
}

// Détecter la langue système
pub fn detect_system_language() -> String {
    for var in &["LANG", "LANGUAGE", "LC_ALL", "LC_MESSAGES"] {
        if let Ok(lang) = std::env::var(var) {
            let code = lang
                .split('.')
                .next()
                .unwrap_or("en")
                .split('_')
                .next()
                .unwrap_or("en")
                .to_lowercase();
            
            let lang_file = i18n_dir().join(format!("{}.json", code));
            if lang_file.exists() {
                return code;
            }
        }
    }
    "en".to_string()
}

// Initialisation des templates
pub fn ensure_user_templates() -> Result<()> {
    let user_cfg = config_dir();
    let user_i18n = i18n_dir();
    let log_path = log_dir();

    fs::create_dir_all(&user_cfg)?;
    fs::create_dir_all(&user_i18n)?;
    fs::create_dir_all(&log_path)?;

    let exe_dir = match std::env::current_exe() {
        Ok(path) => path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        Err(_) => PathBuf::from("."),
    };
    
    let default_cfg_dir = exe_dir.join("config");
    let cfg_source = if default_cfg_dir.exists() {
        default_cfg_dir
    } else {
        PathBuf::from("config")
    };

    // Copier les fichiers de configuration
    for file in &["default_usages.yaml", "default_config.yaml"] {
        let src = cfg_source.join(file);
        let dest_name = file.strip_prefix("default_").unwrap_or(file);
        let dest = user_cfg.join(dest_name);
        if !dest.exists() && src.exists() {
            fs::copy(&src, &dest)?;
        }
    }

    // Copier TOUS les fichiers i18n
    let i18n_source = cfg_source.join("i18n");
    if i18n_source.exists() {
        if let Ok(entries) = fs::read_dir(&i18n_source) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "json") {
                    if let Some(filename) = path.file_name() {
                        let dest = user_i18n.join(filename);
                        if !dest.exists() {
                            fs::copy(&path, &dest)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

// Initialisation des logs
pub fn init_logging() -> Result<()> {
    let log_file = log_dir().join("wzllama.log");
    if let Some(parent) = log_file.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(&log_file)?;
    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .filter_level(log::LevelFilter::Debug)
        .init();
    Ok(())
}

// Chargement i18n avec métadonnées
pub fn load_i18n(lang_code: &str) -> Result<I18n> {
    let file_path = i18n_dir().join(format!("{}.json", lang_code));
    let fallback = i18n_dir().join("en.json");

    let content = if file_path.exists() {
        fs::read_to_string(&file_path)
            .context(format!("Impossible de lire {}", file_path.display()))?
    } else if fallback.exists() {
        fs::read_to_string(&fallback)
            .context(format!("Impossible de lire le fallback {}", fallback.display()))?
    } else {
        let available = get_available_languages();
        if let Some(first_lang) = available.first() {
            let any_path = i18n_dir().join(format!("{}.json", first_lang.code));
            fs::read_to_string(&any_path)
                .context("Aucun fichier i18n trouvé")?
        } else {
            return Ok(I18n {
                meta: LanguageMeta {
                    code: "en".to_string(),
                    name: "English".to_string(),
                    name_en: Some("English".to_string()),
                    direction: "ltr".to_string(),
                },
                map: HashMap::new(),
            });
        }
    };

    // Parser le fichier complet
    let i18n_file: I18nFile = serde_json::from_str(&content)
        .context(format!("Fichier i18n '{}' invalide", file_path.display()))?;

    // Convertir les traductions en HashMap<String, String>
    let mut map = HashMap::new();
    for (key, value) in &i18n_file.translations {
        let str_value = match value {
            serde_json::Value::String(s) => s.clone(),
            _ => value.to_string(),
        };
        map.insert(key.clone(), str_value);
    }

    Ok(I18n {
        meta: i18n_file.language,
        map,
    })
}

// Chargement usages.yaml (inchangé)
pub fn load_usages() -> UsagesConfig {
    let path = config_dir().join("usages.yaml");

    match fs::read_to_string(&path) {
        Ok(content) => match serde_yaml::from_str(&content) {
            Ok(cfg) => {
                let errors = validate_usages(&cfg);
                if !errors.is_empty() {
                    eprintln!("[!] Problèmes dans usages.yaml :");
                    for e in &errors {
                        eprintln!("  - {}", e);
                    }
                    eprintln!("Utilisation de la configuration interne par défaut.");
                    default_usages()
                } else {
                    cfg
                }
            }
            Err(e) => {
                eprintln!("[!] Impossible de parser usages.yaml : {}", e);
                default_usages()
            }
        },
        Err(e) => {
            eprintln!("[!] Impossible de lire usages.yaml : {}", e);
            default_usages()
        }
    }
}

// Configuration par défaut (inchangée)
fn default_usages() -> UsagesConfig {
    let mut usages = HashMap::new();

    usages.insert("big_book".to_string(), UsageSpec {
        i18n_key: "usage.big_book.label".to_string(),
        description_key: Some("usage.big_book.description".to_string()),
        weights: {
            let mut w = HashMap::new();
            w.insert("default".to_string(), 0.7);
            w.insert("writer".to_string(), 0.9);
            w
        },
        params: UsageParams {
            r#type: "book".to_string(),
            pages_per_chunk: Some(20),
            loc_per_chunk: None,
            context_tokens: Some(8192),
            max_tokens_per_call: None,
        },
    });

    usages.insert("big_code".to_string(), UsageSpec {
        i18n_key: "usage.big_code.label".to_string(),
        description_key: Some("usage.big_code.description".to_string()),
        weights: {
            let mut w = HashMap::new();
            w.insert("default".to_string(), 0.6);
            w.insert("dev".to_string(), 0.9);
            w
        },
        params: UsageParams {
            r#type: "code".to_string(),
            pages_per_chunk: None,
            loc_per_chunk: Some(500),
            context_tokens: Some(4096),
            max_tokens_per_call: None,
        },
    });

    usages.insert("fast_agents".to_string(), UsageSpec {
        i18n_key: "usage.fast_agents.label".to_string(),
        description_key: Some("usage.fast_agents.description".to_string()),
        weights: {
            let mut w = HashMap::new();
            w.insert("default".to_string(), 0.9);
            w
        },
        params: UsageParams {
            r#type: "agents".to_string(),
            pages_per_chunk: None,
            loc_per_chunk: None,
            context_tokens: Some(2048),
            max_tokens_per_call: Some(1024),
        },
    });

    usages.insert("mixed".to_string(), UsageSpec {
        i18n_key: "usage.mixed.label".to_string(),
        description_key: Some("usage.mixed.description".to_string()),
        weights: {
            let mut w = HashMap::new();
            w.insert("default".to_string(), 0.5);
            w
        },
        params: UsageParams {
            r#type: "mixed".to_string(),
            pages_per_chunk: None,
            loc_per_chunk: None,
            context_tokens: Some(4096),
            max_tokens_per_call: None,
        },
    });

    UsagesConfig { usages }
}

// Validation des usages (inchangée)
pub fn validate_usages(cfg: &UsagesConfig) -> Vec<String> {
    let mut errors = Vec::new();
    if cfg.usages.is_empty() {
        errors.push("La section 'usages' est vide.".to_string());
        return errors;
    }
    let valid_types = vec!["book", "code", "agents", "mixed"];
    for (key, spec) in &cfg.usages {
        if spec.i18n_key.trim().is_empty() {
            errors.push(format!("Usage '{}': i18n_key manquant.", key));
        }
        if !spec.weights.contains_key("default") {
            errors.push(format!("Usage '{}': poids 'default' obligatoire.", key));
        }
        if !valid_types.contains(&spec.params.r#type.as_str()) {
            errors.push(format!(
                "Usage '{}': type '{}' invalide (attendu: {:?})",
                key, spec.params.r#type, valid_types
            ));
        }
    }
    errors
}

// Validation de l'i18n (modifiée pour I18n)
pub fn validate_i18n(i18n: &I18n, lang_code: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // Vérifier les métadonnées
    if i18n.meta.name.is_empty() {
        errors.push(format!("Langue '{}': métadonnées 'name' manquantes", lang_code));
    }

    // Vérifier les clés obligatoires
    for key in REQUIRED_I18N_KEYS {
        if !i18n.map.contains_key(*key) {
            errors.push(format!("Clé manquante '{}' dans la langue '{}'", key, lang_code));
        }
    }

    errors
}

// Validation de tous les templates
pub fn validate_all_templates() -> Result<()> {
    println!("Validation des templates...\n");

    // Valider usages.yaml
    let usages = load_usages();
    let usage_errors = validate_usages(&usages);
    
    if usage_errors.is_empty() {
        println!("✓ usages.yaml est valide");
    } else {
        println!("✗ usages.yaml contient {} erreur(s) :", usage_errors.len());
        for error in &usage_errors {
            println!("  - {}", error);
        }
    }

    // Valider TOUTES les langues disponibles
    let languages = get_available_languages();
    println!("\nLangues détectées :");
    for lang in &languages {
        println!("  - {} (code: {})", lang.name, lang.code);
    }
    
    for lang in &languages {
        match load_i18n(&lang.code) {
            Ok(i18n) => {
                let errors = validate_i18n(&i18n, &lang.code);
                if errors.is_empty() {
                    println!("✓ i18n/{} ({}) est valide", lang.code, lang.name);
                } else {
                    println!("✗ i18n/{} ({}) contient {} erreur(s) :", lang.code, lang.name, errors.len());
                    for error in &errors {
                        println!("  - {}", error);
                    }
                }
            }
            Err(e) => {
                println!("✗ Impossible de charger i18n/{} : {}", lang.code, e);
            }
        }
    }

    Ok(())
}

// Vérification de l'intégrité i18n
pub fn check_i18n_integrity() -> Result<()> {
    let languages = get_available_languages();
    
    if languages.is_empty() {
        println!("Aucune langue trouvée !");
        return Ok(());
    }

    // Utiliser l'anglais comme référence
    let ref_lang_code = if languages.iter().any(|l| l.code == "en") {
        "en".to_string()
    } else {
        languages[0].code.clone()
    };

    let reference = load_i18n(&ref_lang_code)?;
    println!("Langue de référence : {} (code: {})", reference.meta.name, ref_lang_code);

    for lang in &languages {
        if lang.code == ref_lang_code {
            continue;
        }
        
        match load_i18n(&lang.code) {
            Ok(i18n) => {
                let ref_keys: Vec<&String> = reference.map.keys().collect();
                let mut missing = Vec::new();
                let mut extra = Vec::new();
                
                for key in &ref_keys {
                    if !i18n.map.contains_key(key.as_str()) {
                        missing.push(key.as_str());
                    }
                }
                
                for key in i18n.map.keys() {
                    if !reference.map.contains_key(key) {
                        extra.push(key.as_str());
                    }
                }
                
                if missing.is_empty() && extra.is_empty() {
                    println!("✅ {} ({}) : complet", lang.name, lang.code);
                } else {
                    println!("⚠️  {} ({}) :", lang.name, lang.code);
                    if !missing.is_empty() {
                        println!("   - {} clés manquantes", missing.len());
                    }
                    if !extra.is_empty() {
                        println!("   - {} clés supplémentaires", extra.len());
                    }
                }
            }
            Err(e) => {
                println!("❌ Impossible de charger {} (code: {}) : {}", lang.name, lang.code, e);
            }
        }
    }

    println!("\n💡 Pour ajouter une nouvelle langue :");
    println!("   1. Copiez config/i18n/en.json vers config/i18n/[code].json");
    println!("   2. Modifiez la section \"_language\" avec les bonnes métadonnées");
    println!("   3. Traduisez les valeurs (gardez les clés identiques)");

    Ok(())
}

// Réinitialisation des templates
pub fn reset_templates() -> Result<()> {
    println!("Sauvegarde des templates existants...");

    let usages_path = config_dir().join("usages.yaml");
    if usages_path.exists() {
        let backup_path = usages_path.with_extension("yaml.bak");
        fs::rename(&usages_path, &backup_path)?;
        println!("  usages.yaml -> usages.yaml.bak");
    }

    let config_path = config_dir().join("config.yaml");
    if config_path.exists() {
        let backup_path = config_path.with_extension("yaml.bak");
        fs::rename(&config_path, &backup_path)?;
        println!("  config.yaml -> config.yaml.bak");
    }

    if let Ok(entries) = fs::read_dir(i18n_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "json") {
                let backup_path = path.with_extension("json.bak");
                fs::rename(&path, &backup_path)?;
                println!("  {} -> {}.bak", 
                    path.file_name().unwrap().to_string_lossy(),
                    path.file_stem().unwrap().to_string_lossy());
            }
        }
    }

    println!("\nRestauration des templates par défaut...");
    ensure_user_templates()?;
    println!("Templates réinitialisés avec succès !");

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WzllamaState {
    pub installed: InstalledTools,
    pub fleets: HashMap<String, FleetState>,
    pub last_model: Option<String>,
    pub last_usage: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstalledTools {
    pub docker: bool,
    pub ollama: bool,
    pub open_webui: bool,
    pub openclaw: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FleetState {
    pub profile: String,
    pub orchestrator: String,
    pub agents: Vec<String>,
    pub openclaw_installed: bool,
}

pub fn state_file() -> PathBuf {
    wzllama_dir().join("state.json")
}

pub fn load_state() -> WzllamaState {
    let path = state_file();
    if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_default())
            .unwrap_or_default()
    } else {
        WzllamaState::default()
    }
}

pub fn save_state(state: &WzllamaState) -> Result<()> {
    let path = state_file();
    std::fs::write(&path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

pub fn mark_installed(tool: &str, state: &mut WzllamaState) {
    match tool {
        "docker" => state.installed.docker = true,
        "ollama" => state.installed.ollama = true,
        "Open WebUI" => state.installed.open_webui = true,
        "openclaw" => state.installed.openclaw = true,
        _ => {}
    }
    let _ = save_state(state);
}

pub fn add_fleet(profile: &str, orchestrator: &str, agents: Vec<String>, state: &mut WzllamaState) {
    state.fleets.insert(profile.to_string(), FleetState {
        profile: profile.to_string(),
        orchestrator: orchestrator.to_string(),
        agents,
        openclaw_installed: false,
    });
    let _ = save_state(state);
}

/// Scanne les dossiers ~/.openclaw-* pour détecter les flottes existantes
pub fn detect_openclaw_fleets() -> HashMap<String, FleetState> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut fleets = HashMap::new();
    
    if let Ok(entries) = std::fs::read_dir(&home) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dirname) = path.file_name().and_then(|n| n.to_str()) {
                    if dirname.starts_with(".openclaw-") {
                        let profile = dirname.strip_prefix(".openclaw-").unwrap_or(dirname);
                        let config_path = path.join("openclaw.json");
                        
                        if config_path.exists() {
                            if let Ok(content) = std::fs::read_to_string(&config_path) {
                                if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                                    let orchestrator = config["agents"]["defaults"]["model"]["primary"]
                                        .as_str()
                                        .map(|s| s.strip_prefix("ollama/").unwrap_or(s).to_string())
                                        .unwrap_or_else(|| "inconnu".to_string());
                                    
                                    let agents: Vec<String> = config["agents"]["list"]
                                        .as_array()
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|a| {
                                                    a["model"]["primary"].as_str()
                                                        .map(|s| s.strip_prefix("ollama/").unwrap_or(s).to_string())
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    
                                    // Vérifier si le service systemd existe
                                    let installed = std::process::Command::new("systemctl")
                                        .args(["--user", "is-enabled", &format!("openclaw-gateway-{}.service", profile)])
                                        .output()
                                        .map(|o| o.status.success())
                                        .unwrap_or(false);
                                    
                                    fleets.insert(profile.to_string(), FleetState {
                                        profile: profile.to_string(),
                                        orchestrator,
                                        agents,
                                        openclaw_installed: installed,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    fleets
}

/// Met à jour le state.json avec les flottes détectées
pub fn sync_fleets() -> WzllamaState {
    let mut state = load_state();
    let detected = detect_openclaw_fleets();
    state.fleets = detected;
    let _ = save_state(&state);
    state
}
