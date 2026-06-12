#![allow(dead_code)]

use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::blocking::Client;
use serde::Deserialize;
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
            Some(bytes) if bytes >= 1_000_000_000 => {
                format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
            }
            Some(bytes) if bytes >= 1_000_000 => format!("{:.1} MB", bytes as f64 / 1_000_000.0),
            Some(bytes) => format!("{} KB", bytes / 1000),
            None => "?".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelDetails {
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub parameter_size: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub quantization_level: Option<String>,
}

/// Response from /api/show endpoint
#[derive(Debug, Deserialize, Clone)]
pub struct ModelShowResponse {
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub modelfile: Option<String>,
    #[serde(default)]
    pub parameters: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub details: Option<ModelDetails>,
    #[serde(default)]
    pub model_info: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

/// Récupère les modèles locaux via l'API Ollama
pub fn fetch_local_models(base_url: &str) -> Result<Vec<OllamaModel>> {
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
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
    running
        .iter()
        .any(|m| m.starts_with(model_name.split(':').next().unwrap_or(model_name)))
}

/// Détecte si Ollama est lancé
pub fn detect_url() -> Option<String> {
    for url in &["http://localhost:11434", "http://127.0.0.1:11434"] {
        if reqwest::blocking::get(format!("{}/api/tags", url))
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
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
                return models
                    .iter()
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
    println!("   ✅ {} installed !", model.green());
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
            return models
                .iter()
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
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let resp = client.get(url).send().context("Catalogue injoignable")?;
    let data: OllamaTagsResponse = resp.json().context("Parsing catalogue échoué")?;
    Ok(data.models)
}

/// List of popular models not always in the API catalog
/// These are verified working models from ollama.com/library
fn get_popular_models() -> Vec<OllamaModel> {
    vec![
        // Llama 3.3 - Best all-around model (2026)
        OllamaModel {
            name: "llama3.3:8b".to_string(),
            model: "llama3.3:8b".to_string(),
            modified_at: None,
            size: Some(8 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "llama3.3:70b".to_string(),
            model: "llama3.3:70b".to_string(),
            modified_at: None,
            size: Some(70 * 1024 * 1024 * 1024),
            details: None,
        },
        // Qwen 3.6 - Latest models (27b, 35b)
        OllamaModel {
            name: "qwen3.6:27b".to_string(),
            model: "qwen3.6:27b".to_string(),
            modified_at: None,
            size: Some(27 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "qwen3.6:35b-a3b".to_string(),
            model: "qwen3.6:35b-a3b".to_string(),
            modified_at: None,
            size: Some(35 * 1024 * 1024 * 1024),
            details: None,
        },
        // Mistral Nemo - 12B reasoning model
        OllamaModel {
            name: "mistral-nemo:12b".to_string(),
            model: "mistral-nemo:12b".to_string(),
            modified_at: None,
            size: Some(12 * 1024 * 1024 * 1024),
            details: None,
        },
        // CodeLlama - Code generation
        OllamaModel {
            name: "codellama:7b".to_string(),
            model: "codellama:7b".to_string(),
            modified_at: None,
            size: Some(7 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "codellama:34b".to_string(),
            model: "codellama:34b".to_string(),
            modified_at: None,
            size: Some(34 * 1024 * 1024 * 1024),
            details: None,
        },
        // Embedding models
        OllamaModel {
            name: "nomic-embed-text:7b".to_string(),
            model: "nomic-embed-text:7b".to_string(),
            modified_at: None,
            size: Some(7 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "mxbai-embed-large:335m".to_string(),
            model: "mxbai-embed-large:335m".to_string(),
            modified_at: None,
            size: Some(335 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "all-minilm:3b".to_string(),
            model: "all-minilm:3b".to_string(),
            modified_at: None,
            size: Some(3 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "bge-large:340m".to_string(),
            model: "bge-large:340m".to_string(),
            modified_at: None,
            size: Some(340 * 1024 * 1024),
            details: None,
        },
        // Large models
        OllamaModel {
            name: "gpt-oss:120b".to_string(),
            model: "gpt-oss:120b".to_string(),
            modified_at: None,
            size: Some(65 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "deepseek-v3:671b".to_string(),
            model: "deepseek-v3:671b".to_string(),
            modified_at: None,
            size: Some(400 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "deepseek-coder-v2:16b".to_string(),
            model: "deepseek-coder-v2:16b".to_string(),
            modified_at: None,
            size: Some(16 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "devstral:24b".to_string(),
            model: "devstral:24b".to_string(),
            modified_at: None,
            size: Some(24 * 1024 * 1024 * 1024),
            details: None,
        },
        // Qwen 3 family
        OllamaModel {
            name: "qwen3:30b".to_string(),
            model: "qwen3:30b".to_string(),
            modified_at: None,
            size: Some(30 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "qwen3:35b-a3b".to_string(),
            model: "qwen3:35b-a3b".to_string(),
            modified_at: None,
            size: Some(35 * 1024 * 1024 * 1024),
            details: None,
        },
        // Vision models
        OllamaModel {
            name: "qwen2.5vl:72b".to_string(),
            model: "qwen2.5vl:72b".to_string(),
            modified_at: None,
            size: Some(72 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "llava-llama3:8b".to_string(),
            model: "llava-llama3:8b".to_string(),
            modified_at: None,
            size: Some(8 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "minicpm-v:8b".to_string(),
            model: "minicpm-v:8b".to_string(),
            modified_at: None,
            size: Some(8 * 1024 * 1024 * 1024),
            details: None,
        },
        // Coder models
        OllamaModel {
            name: "qwen2.5-coder:32b".to_string(),
            model: "qwen2.5-coder:32b".to_string(),
            modified_at: None,
            size: Some(32 * 1024 * 1024 * 1024),
            details: None,
        },
        OllamaModel {
            name: "starcoder2:15b".to_string(),
            model: "starcoder2:15b".to_string(),
            modified_at: None,
            size: Some(15 * 1024 * 1024 * 1024),
            details: None,
        },
    ]
}

/// Fetch catalog and merge with popular models list
pub fn fetch_full_catalog() -> Result<Vec<OllamaModel>> {
    let mut models = fetch_remote_catalog()?;

    // Try to scrape the library page for additional models
    if let Ok(scraped) = scrape_library_models() {
        use std::collections::HashSet;
        let existing: HashSet<String> = models.iter().map(|m| m.name.clone()).collect();

        for model in scraped {
            if !existing.contains(&model.name) {
                models.push(model);
            }
        }
    }

    // Add models from popular list that aren't already included
    let existing: std::collections::HashSet<String> =
        models.iter().map(|m| m.name.clone()).collect();

    for model in get_popular_models() {
        if !existing.contains(&model.name) {
            models.push(model);
        }
    }

    Ok(models)
}

/// Get detailed model information from local Ollama server using POST /api/show
pub fn show_model(model_name: &str) -> Result<ModelShowResponse> {
    let url = "http://localhost:11434/api/show";
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    let resp = client
        .post(url)
        .json(&json!({ "name": model_name }))
        .send()
        .context("Local Ollama show failed")?;
    let data: ModelShowResponse = resp.json().context("Parsing /api/show failed")?;
    Ok(data)
}

/// Get detailed model information from remote Ollama catalog using POST /api/show
pub fn show_remote_model(model_name: &str) -> Result<ModelShowResponse> {
    let url = "https://ollama.com/api/show";
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let resp = client
        .post(url)
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
    let mut seen: HashSet<String> = local
        .iter()
        .map(|m| m.name.split(':').next().unwrap_or(&m.name).to_lowercase())
        .collect();
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

/// Scrape ollama.com/library to get a comprehensive list of models
pub fn scrape_library_models() -> Result<Vec<OllamaModel>> {
    let url = "https://ollama.com/library";
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let html = client
        .get(url)
        .send()
        .context("Failed to fetch ollama library page")?
        .text()?;

    use scraper::{Html, Selector};

    let document = Html::parse_document(&html);

    // Select all links to models in the library grid
    // The library page uses links like /library/qwen3, /library/llama3.1, etc.
    let link_selector = Selector::parse("a[href^='/library/']")
        .map_err(|e| anyhow::anyhow!("Invalid selector: {}", e))?;

    let mut models = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            // Extract model name from href like /library/qwen3 or /library/llama3.1:8b
            let name = href.trim_start_matches("/library/");
            // Filter out non-model links like "/library" itself or empty names
            if !name.is_empty() && name.len() > 1 && !seen.contains(name) {
                // Skip links that look like paths but not model names (very long or containing special chars)
                if name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == ':' || c == '_')
                {
                    seen.insert(name.to_string());
                    models.push(OllamaModel {
                        name: name.to_string(),
                        model: name.to_string(),
                        modified_at: None,
                        size: None,
                        details: None,
                    });
                }
            }
        }
    }

    Ok(models)
}

pub fn run_benchmark() -> Result<()> {
    println!("Benchmark à implémenter");
    Ok(())
}
