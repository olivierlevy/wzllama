use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use colored::Colorize;
use serde_json::json;

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaModel {
    pub name: String,
    pub model: String,
    #[allow(dead_code)]
    pub modified_at: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub details: Option<ModelDetails>,
}

impl OllamaModel {
    /// Format size in human-readable format
    pub fn formatted_size(&self) -> String {
        match self.size {
            Some(bytes) if bytes >= 1_000_000_000 => format!("{:.1} GB", bytes as f64 / 1_000_000_000.0),
            Some(bytes) if bytes >= 1_000_000 => format!("{:.1} MB", bytes as f64 / 1_000_000.0),
            Some(bytes) => format!("{} KB", bytes / 1000),
            None => "?".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelDetails {
    #[serde(default)] pub family: Option<String>,
    #[serde(default)] #[allow(dead_code)] pub parameter_size: Option<String>,
    #[serde(default)] #[allow(dead_code)] pub quantization_level: Option<String>,
}

/// Response from /api/show endpoint
#[derive(Debug, Deserialize, Clone)]
pub struct ModelShowResponse {
    #[serde(default)] pub license: Option<String>,
    #[serde(default)] pub modelfile: Option<String>,
    #[serde(default)] pub parameters: Option<String>,
    #[serde(default)] pub template: Option<String>,
    #[serde(default)] pub system: Option<String>,
    #[serde(default)] pub details: Option<ModelDetails>,
    #[serde(default)] pub model_info: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse { models: Vec<OllamaModel> }

/// Récupère les modèles locaux via l'API Ollama
pub fn fetch_local_models(base_url: &str) -> Result<Vec<OllamaModel>> {
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    let client = Client::builder().timeout(std::time::Duration::from_secs(5)).build()?;
    let resp = client.get(&url).send().context("Ollama injoignable")?;
    let data: OllamaTagsResponse = resp.json().context("Parsing /api/tags échoué")?;
    Ok(data.models)
}

/// Get all local models (convenience function)
pub fn get_models() -> Vec<OllamaModel> {
    if let Some(url) = detect_url() {
        fetch_local_models(&url).unwrap_or_default()
    } else {
        vec![]
    }
}

/// Check if a specific model is running
pub fn is_model_running(model_name: &str) -> bool {
    let running = get_running_models();
    running.iter().any(|m| m.starts_with(model_name.split(':').next().unwrap_or(model_name)))
}

/// Détecte si Ollama est lancé
pub fn detect_url() -> Option<String> {
    for url in &["http://localhost:11434", "http://127.0.0.1:11434"] {
        if reqwest::blocking::get(format!("{}/api/tags", url)).map(|r| r.status().is_success()).unwrap_or(false) {
            return Some(url.to_string());
        }
    }
    None
}

/// Récupère les modèles en cours d'exécution
pub fn get_running_models() -> Vec<String> {
    if let Ok(resp) = reqwest::blocking::get("http://localhost:11434/api/ps") {
        if let Ok(json) = resp.json::<serde_json::Value>() {
            if let Some(models) = json["models"].as_array() {
                return models.iter()
                    .filter_map(|m| m["name"].as_str().map(String::from))
                    .collect();
            }
        }
    }
    vec![]
}

/// Télécharge un modèle avec affichage de la progression
pub fn pull_model(model: &str) -> Result<()> {
    println!("📥 Téléchargement de {}...", model.cyan().bold());
    crate::core::shell::run_live(&format!("ollama pull {}", model))?;
    println!("   ✅ {} installé !", model.green());
    Ok(())
}

/// Crée un modèle personnalisé
pub fn create_model(name: &str, modelfile: &str) -> Result<()> {
    let tmp = format!("/tmp/wzllama_{}.Modelfile", name);
    std::fs::write(&tmp, modelfile)?;
    crate::core::shell::run(&format!("ollama create {} -f {}", name, tmp))?;
    Ok(())
}

/// Supprime un modèle
pub fn delete_model(name: &str) -> Result<()> {
    crate::core::shell::run(&format!("ollama rm {}", name))?;
    Ok(())
}

/// Liste les modèles créés par wzllama
pub fn list_wzllama_models() -> Vec<String> {
    if let Some(url) = detect_url() {
        if let Ok(models) = fetch_local_models(&url) {
            return models.iter()
                .filter(|m| m.name.starts_with("wzllama-"))
                .map(|m| m.name.clone())
                .collect();
        }
    }
    vec![]
}

/// Récupère le catalogue distant officiel
pub fn fetch_remote_catalog() -> Result<Vec<OllamaModel>> {
    let url = "https://ollama.com/api/tags";
    let client = Client::builder().timeout(std::time::Duration::from_secs(10)).build()?;
    let resp = client.get(url).send().context("Catalogue injoignable")?;
    let data: OllamaTagsResponse = resp.json().context("Parsing catalogue échoué")?;
    Ok(data.models)
}

/// Get detailed model information from local Ollama server using POST /api/show
pub fn show_model(model_name: &str) -> Result<ModelShowResponse> {
    let url = "http://localhost:11434/api/show";
    let client = Client::builder().timeout(std::time::Duration::from_secs(5)).build()?;
    let resp = client.post(url)
        .json(&json!({ "name": model_name }))
        .send()
        .context("Local Ollama show failed")?;
    let data: ModelShowResponse = resp.json().context("Parsing /api/show failed")?;
    Ok(data)
}

/// Get detailed model information from remote Ollama catalog using POST /api/show
pub fn show_remote_model(model_name: &str) -> Result<ModelShowResponse> {
    let url = "https://ollama.com/api/show";
    let client = Client::builder().timeout(std::time::Duration::from_secs(10)).build()?;
    let resp = client.post(url)
        .json(&json!({ "name": model_name }))
        .send()
        .context("Remote Ollama show failed")?;
    let data: ModelShowResponse = resp.json().context("Parsing /api/show failed")?;
    Ok(data)
}

/// Get model details - tries local first, then remote
pub fn get_model_details(model_name: &str) -> Result<ModelShowResponse> {
    // Try local first
    if detect_url().is_some() {
        if let Ok(details) = show_model(model_name) {
            return Ok(details);
        }
    }
    // Fall back to remote
    show_remote_model(model_name)
}

/// Fusionne locaux + distants sudo doublons
pub fn merge_models(local: &[OllamaModel], remote: &[OllamaModel]) -> Vec<(OllamaModel, bool)> {
    use std::collections::HashSet;
    let mut seen: HashSet<String> = local.iter().map(|m| m.name.split(':').next().unwrap_or(&m.name).to_lowercase()).collect();
    let mut all: Vec<_> = local.iter().map(|m| (m.clone(), true)).collect();
    for m in remote {
        let key = m.name.split(':').next().unwrap_or(&m.name).to_lowercase();
        if !seen.contains(&key) {
            all.push((m.clone(), false));
            seen.insert(key);
        }
    }
    all
}

pub fn run_benchmark() -> Result<()> {
    println!("Benchmark à implémenter");
    Ok(())
}