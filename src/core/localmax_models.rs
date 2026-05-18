use anyhow::Result;
use colored::Colorize;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde::Serialize;
use std::fs;

const BASE_URL: &str = "https://localmaxxing.com/api";

/// Dynamic mapping from HuggingFace model IDs to Ollama model names
/// Extracts model family, variant, and parameter count from the HF ID
pub fn hf_to_ollama_name(hf_id: &str) -> String {
    let hf_lower = hf_id.to_lowercase();
    
    // Extract parameter size (e.g., "14b", "30b", "72b")
    let param_size = extract_param_size(&hf_lower);
    
    // Detect model family and generate appropriate Ollama name
    if hf_lower.contains("qwen") {
        if hf_lower.contains("qwen3.6") || hf_lower.contains("qwen3_6") {
            // Qwen3.6 not yet in Ollama, use closest qwen3 variant
            match param_size.as_str() {
                "72b" | "70b" => "qwen3:30b",
                "35b" | "32b" | "30b" => "qwen3:30b",
                "27b" | "24b" | "14b" => "qwen3:14b",
                "8b" => "qwen3:8b",
                _ => "qwen3:latest",
            }.to_string()
        } else if hf_lower.contains("qwen3") {
            match param_size.as_str() {
                "30b" | "32b" => "qwen3:30b",
                "14b" => "qwen3:14b",
                "8b" => "qwen3:8b",
                _ => "qwen3:latest",
            }.to_string()
        } else if hf_lower.contains("qwen2.5-coder") || (hf_lower.contains("coder") && hf_lower.contains("qwen2")) {
            match param_size.as_str() {
                "32b" => "qwen2.5-coder:32b",
                "14b" => "qwen2.5-coder:14b",
                "7b" => "qwen2.5-coder:7b",
                _ => "qwen2.5-coder:14b",
            }.to_string()
        } else if hf_lower.contains("qwen2.5") {
            match param_size.as_str() {
                "72b" => "qwen2.5:72b",
                "32b" => "qwen2.5:32b",
                "14b" => "qwen2.5:14b",
                _ => "qwen2.5:latest",
            }.to_string()
        } else if hf_lower.contains("coder") {
            "qwen2.5-coder:14b".to_string()
        } else {
            format!("qwen{}:{}", get_qwen_variant(&hf_lower), param_size)
        }
    } else if hf_lower.contains("phi") {
        if hf_lower.contains("phi-4") || hf_lower.contains("phi4") {
            "phi4:latest".to_string()
        } else {
            "phi3:latest".to_string()
        }
    } else if hf_lower.contains("deepseek") {
        if hf_lower.contains("coder") {
            "deepseek-coder:latest".to_string()
        } else if hf_lower.contains("r1") || hf_lower.contains("reasoner") {
            "deepseek-r1:latest".to_string()
        } else {
            "deepseek-v2:latest".to_string()
        }
    } else if hf_lower.contains("gemma") {
        if hf_lower.contains("gemma-3") || hf_lower.contains("gemma3") {
            match param_size.as_str() {
                "27b" => "gemma3:27b",
                "12b" => "gemma3:12b",
                _ => "gemma3:latest",
            }.to_string()
        } else if hf_lower.contains("gemma-2") || hf_lower.contains("gemma2") {
            match param_size.as_str() {
                "27b" => "gemma2:27b",
                _ => "gemma2:9b",
            }.to_string()
        } else {
            "gemma:latest".to_string()
        }
    } else if hf_lower.contains("llama") || hf_lower.contains("meta-llama") {
        if hf_lower.contains("llama-3.1") || hf_lower.contains("llama3.1") {
            match param_size.as_str() {
                "70b" => "llama3.1:70b",
                "8b" => "llama3.1:8b",
                _ => "llama3.1:latest",
            }.to_string()
        } else if hf_lower.contains("llama-3") || hf_lower.contains("llama3") {
            match param_size.as_str() {
                "70b" => "llama3:70b",
                "8b" => "llama3:8b",
                _ => "llama3:latest",
            }.to_string()
        } else {
            format!("llama3:{}", param_size)
        }
    } else if hf_lower.contains("mistral") {
        if hf_lower.contains("mixtral") {
            "mixtral:8x7b".to_string()
        } else if hf_lower.contains("codestral") {
            "codestral:latest".to_string()
        } else {
            "mistral:latest".to_string()
        }
    } else if hf_lower.contains("gpt-oss") || hf_lower.contains("openai") {
        "gpt-oss:latest".to_string()
    } else {
        // Dynamic fallback for unknown models
        // Try to find a reasonable Ollama equivalent based on size and type
        let is_coder = hf_lower.contains("coder") || hf_lower.contains("code");
        let is_instruct = hf_lower.contains("instruct");
        
        // Map to the best available Ollama model of similar size and purpose
        let base_recommendation = if is_coder {
            // For coding models, recommend Qwen2.5-coder or DeepSeek-coder
            match param_size.as_str() {
                "72b" | "70b" => "qwen2.5-coder:32b",
                "32b" | "30b" => "qwen2.5-coder:32b",
                "27b" | "14b" | "8b" => "qwen2.5-coder:14b",
                "7b" => "qwen2.5-coder:7b",
                _ => "qwen2.5-coder:14b",
            }
        } else {
            // For general models, recommend based on family hints
            if hf_lower.contains("deepseek") {
                match param_size.as_str() {
                    "72b" | "70b" => "deepseek-r1:latest",
                    "32b" | "30b" => "deepseek-r1:latest",
                    _ => "deepseek-r1:latest",
                }
            } else {
                // Generic fallback based on size
                match param_size.as_str() {
                    "72b" | "70b" => "qwen2.5:72b",
                    "32b" | "30b" => "qwen2.5:32b",
                    "27b" | "14b" => "qwen2.5:14b",
                    "8b" => "qwen2.5:7b",
                    _ => "qwen2.5:7b",
                }
            }
        };
        
        base_recommendation.to_string()
    }
}

/// Extract parameter size from model name (e.g., "14b", "72b", "30b")
fn extract_param_size(hf_lower: &str) -> String {
    // Common pattern: number followed by 'b' (e.g., 7b, 14b, 30b, 32b, 72b)
    // Use simple string parsing instead of regex
    let mut best_match = 0u32;
    
    for part in hf_lower.split(|c: char| !c.is_ascii_digit()) {
        if let Ok(num) = part.parse::<u32>() {
            if num >= 1 && num <= 1000 {
                best_match = num;
            }
        }
    }
    
    if best_match >= 70 { "72b".to_string() }
    else if best_match >= 30 { "30b".to_string() }
    else if best_match >= 14 { "14b".to_string() }
    else if best_match >= 8 { "8b".to_string() }
    else if best_match >= 7 { "7b".to_string() }
    else if best_match >= 3 { "3b".to_string() }
    else if best_match >= 1 { format!("{}b", best_match) }
    else { "latest".to_string() }
}

/// Get Qwen variant number from model name
fn get_qwen_variant(hf_lower: &str) -> &'static str {
    if hf_lower.contains("qwen3") { "3" }
    else if hf_lower.contains("qwen2.5") { "2.5" }
    else if hf_lower.contains("qwen2") { "2" }
    else { "" }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LocalMaxModel {
    pub id: String,
    #[serde(rename = "hfId", default)]
    pub hf_id: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub organization: String,
    #[serde(default)]
    pub family: Option<String>,  // Can be null in API
    #[serde(default)]
    pub params: Option<f64>,
    #[serde(rename = "isMoE", default)]
    pub is_moe: bool,
    #[serde(default)]
    pub tags: Option<String>,  // JSON string dans l'API
    #[serde(rename = "_count", default)]
    pub _count: Option<ModelCount>,
    #[serde(default)]
    pub speed_stats: Option<SpeedStats>,
    #[serde(default)]
    pub base_model: Option<BaseModelInfo>,
    // Fields that can be null in API - use flatten to accept them
    #[serde(rename = "activeParams", default)]
    pub active_params: Option<f64>,
    #[serde(rename = "pipelineTag", default)]
    pub pipeline_tag: Option<String>,
    // Unknown fields from API are silently ignored via flatten
    #[serde(flatten)]
    extras: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone, Default, Serialize)]
pub struct ModelCount {
    #[serde(rename = "benchmarkRuns")]
    pub benchmark_runs: u32,
}

#[derive(Debug, Deserialize, Clone, Default, Serialize)]
pub struct SpeedStats {
    #[serde(rename = "maxTokS")]
    pub max_tok_s: Option<f64>,
    #[serde(rename = "medianTokS")]
    pub median_tok_s: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "hfId")]
    pub hf_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub params: Option<f64>,
    #[serde(rename = "benchmarkCount", default)]
    pub benchmark_count: Option<u32>,
}

/// Response from localmaxxing API - models come wrapped in {base: {...}, finetunes: [...]}
#[derive(Debug, Deserialize, Clone)]
pub struct LocalMaxResponse {
    #[serde(default)]
    pub base: Option<LocalMaxModel>,
    #[serde(default)]
    pub finetunes: Vec<LocalMaxModel>,
}

/// BaseModel info from the API
#[derive(Debug, Deserialize, Clone, Default, Serialize)]
pub struct BaseModelInfo {
    #[serde(rename = "hfId")]
    pub hf_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

impl LocalMaxResponse {
    /// Returns the base model (primary) - use this for recommendations
    pub fn into_base(self) -> Option<LocalMaxModel> {
        self.base
    }
    
    /// Returns all models including finetunes
    pub fn all_models(self) -> Vec<LocalMaxModel> {
        let mut models = self.base.into_iter().collect::<Vec<_>>();
        models.extend(self.finetunes);
        models
    }
}

impl LocalMaxModel {
    /// Convert from wrapped API response to usable model
    pub fn from_response(resp: LocalMaxResponse) -> Vec<Self> {
        resp.all_models()
    }
}

pub fn fetch_latest_models(limit: u32) -> Result<Vec<LocalMaxModel>> {
    let client = Client::new();
    let url = format!("{}/models?limit={}&offset=0&tree=true", BASE_URL, limit);
    
    let response = client.get(&url)
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| anyhow::anyhow!("Network error: {}", e))?;
    
    if response.status().is_success() {
        let body = response.text()
            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;
        
        // Try parsing as wrapped response {base: {...}, finetunes: [...]}
        if let Ok(responses) = serde_json::from_str::<Vec<LocalMaxResponse>>(&body) {
            let models: Vec<LocalMaxModel> = responses.into_iter().flat_map(|r| r.all_models()).collect();
            return Ok(models);
        }
        
        // Try parsing as flat list
        if let Ok(models) = serde_json::from_str::<Vec<LocalMaxModel>>(&body) {
            return Ok(models);
        }
        
        anyhow::bail!("Failed to parse models response")
    } else {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Failed to fetch models: status {} - {}", status, body)
    }
}

/// Fetch models filtered by hardware (e.g., RTX 3060, RTX 4090)
pub fn fetch_models_by_hardware(hardware_name: &str, limit: u32) -> Result<Vec<LocalMaxModel>> {
    let client = Client::new();
    // Use tree=true to get the correct JSON structure with {base: {...}, finetunes: [...]}
    let url = format!("{}/models?hardwareName={}&limit={}&tree=true", BASE_URL, hardware_name, limit);
    
    let response = client.get(&url)
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| anyhow::anyhow!("Network error: {}", e))?;
    
    if response.status().is_success() {
        // API returns [{base: {...}, finetunes: [...]}, ...]
        let responses: Vec<LocalMaxResponse> = response.json()
            .map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))?;
        // Flatten to get all models (base + finetunes)
        let models: Vec<LocalMaxModel> = responses.into_iter().flat_map(|r| r.all_models()).collect();
        Ok(models)
    } else {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Failed to fetch models for hardware '{}': status {} - {}", hardware_name, status, body)
    }
}

pub fn fetch_models_by_search(query: &str, limit: u32) -> Result<Vec<LocalMaxModel>> {
    use crate::core::cache;
    
    // Helper to get cache directory
    fn cache_dir() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(".wzllama/cache"))
    }
    
    // Try daily cache first with tree=true format
    let daily_tree_cache = cache_dir()?.join("localmax_tree.json");
    let daily_search_cache = cache_dir()?.join("localmax_search_code.json");
    
    // Try to read from daily cache
    if query == "code" && daily_search_cache.exists() {
        if let Ok(data) = fs::read_to_string(&daily_search_cache) {
            if let Ok(responses) = serde_json::from_str::<Vec<LocalMaxResponse>>(&data) {
                let models: Vec<LocalMaxModel> = responses.into_iter().flat_map(|r| r.all_models()).collect();
                if !models.is_empty() {
                    return Ok(models);
                }
            }
        }
    }
    
    // Try generic tree cache for other queries
    if daily_tree_cache.exists() {
        if let Ok(data) = fs::read_to_string(&daily_tree_cache) {
            if let Ok(responses) = serde_json::from_str::<Vec<LocalMaxResponse>>(&data) {
                let models: Vec<LocalMaxModel> = responses.into_iter().flat_map(|r| r.all_models()).collect();
                // Filter by query if not empty
                if !models.is_empty() {
                    if query.is_empty() || query == "performance" {
                        return Ok(models);
                    }
                }
            }
        }
    }
    
    // Try regular cache as fallback
    let cache_key = format!("localmax_search_{}_{}", query.replace(' ', "_"), limit);
    if let Ok(Some(cached)) = cache::read_cache(&cache_key, false) {
        if let Ok(models) = serde_json::from_str::<Vec<LocalMaxModel>>(&cached) {
            return Ok(models);
        }
    }
    
    let client = Client::new();
    let url = format!("{}/models?search={}&limit={}", BASE_URL, query, limit);
    
    let response = client.get(&url)
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| anyhow::anyhow!("Network error: {}", e))?;
    
    if response.status().is_success() {
        let body = response.text()
            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;
        
        // Check if body is empty or whitespace
        if body.trim().is_empty() {
            anyhow::bail!("Empty response from API");
        }
        
        // Try parsing as flat list first
        if let Ok(models) = serde_json::from_str::<Vec<LocalMaxModel>>(&body) {
            if !models.is_empty() {
                if let Ok(json) = serde_json::to_string(&models) {
                    let _ = cache::write_cache(&cache_key, &json);
                }
                return Ok(models);
            }
        }
        
        // Try parsing as wrapped response {base: {...}, finetunes: [...]}
        if let Ok(responses) = serde_json::from_str::<Vec<LocalMaxResponse>>(&body) {
            let models: Vec<LocalMaxModel> = responses.into_iter().flat_map(|r| r.all_models()).collect();
            if !models.is_empty() {
                if let Ok(json) = serde_json::to_string(&models) {
                    let _ = cache::write_cache(&cache_key, &json);
                }
                return Ok(models);
            }
        }
        
        anyhow::bail!("Failed to parse API response as models. Response: {}", body.chars().take(200).collect::<String>())
    } else {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Failed to search models: status {} - {}", status, body)
    }
}

pub fn search_models_fuzzy(q: &str) -> Result<Vec<SearchResult>> {
    let client = Client::new();
    let url = format!("{}/models/search?q={}&limit=10", BASE_URL, q);
    
    let response = client.get(&url)
        .header("Accept", "application/json")
        .send()?;
    
    if response.status().is_success() {
        let results: Vec<SearchResult> = response.json()?;
        Ok(results)
    } else {
        anyhow::bail!("Failed to search models")
    }
}

impl LocalMaxModel {
    pub fn to_ollama_model(&self) -> crate::core::ollama_api::OllamaModel {
        let ollama_name = hf_to_ollama_name(&self.hf_id);
        crate::core::ollama_api::OllamaModel {
            name: ollama_name.clone(),
            model: ollama_name,
            modified_at: None,
            size: None,
            details: None,
        }
    }
    
    pub fn to_ollama_name(&self) -> String {
        hf_to_ollama_name(&self.hf_id)
    }
    
    /// Check if the HF model maps directly to an Ollama model of the same family
    pub fn is_direct_ollama_mapping(&self) -> bool {
        let hf_lower = self.hf_id.to_lowercase();
        // Known direct mappings (HF model ID contains the same name as Ollama)
        hf_lower.contains("qwen")
            || hf_lower.contains("phi")
            || hf_lower.contains("deepseek")
            || hf_lower.contains("gemma")
            || hf_lower.contains("llama")
            || hf_lower.contains("mistral")
            || hf_lower.contains("mixtral")
            || hf_lower.contains("codestral")
    }
    
    /// Returns the ollama name, with a suffix indicating if it's a fallback
    pub fn to_ollama_name_with_indicator(&self) -> String {
        let name = self.to_ollama_name();
        if self.is_direct_ollama_mapping() {
            name
        } else {
            format!("{} (recommended)", name)
        }
    }
    
    pub fn formatted_performance(&self) -> String {
        if let Some(stats) = &self.speed_stats {
            if let Some(toks) = stats.median_tok_s {
                format!("{:.1} tok/s", toks)
            } else {
                "No data".to_string()
            }
        } else if let Some(ref _count) = self._count {
            format!("{} runs", _count.benchmark_runs)
        } else {
            "No data".to_string()
        }
    }
}

/// Returns popular fallback models when API is unavailable
pub fn get_popular_models() -> Vec<LocalMaxModel> {
    vec![
        LocalMaxModel {
            id: "qwen3-30b".to_string(),
            hf_id: "Qwen/Qwen3-30B".to_string(),
            display_name: Some("Qwen3 30B".to_string()),
            organization: "Qwen".to_string(),
            family: Some("qwen3".to_string()),
            params: Some(30.0),
            is_moe: false,
            tags: None,
            _count: None,
            speed_stats: None,
            base_model: None,
            active_params: None,
            pipeline_tag: None,
            extras: Default::default(),
        },
        LocalMaxModel {
            id: "qwen3-14b".to_string(),
            hf_id: "Qwen/Qwen3-14B".to_string(),
            display_name: Some("Qwen3 14B".to_string()),
            organization: "Qwen".to_string(),
            family: Some("qwen3".to_string()),
            params: Some(14.0),
            is_moe: false,
            tags: None,
            _count: None,
            speed_stats: None,
            base_model: None,
            active_params: None,
            pipeline_tag: None,
            extras: Default::default(),
        },
        LocalMaxModel {
            id: "qwen2.5-coder-14b".to_string(),
            hf_id: "Qwen/Qwen2.5-Coder-14B".to_string(),
            display_name: Some("Qwen2.5 Coder 14B".to_string()),
            organization: "Qwen".to_string(),
            family: Some("qwen2.5-coder".to_string()),
            params: Some(14.0),
            is_moe: false,
            tags: None,
            _count: None,
            speed_stats: None,
            base_model: None,
            active_params: None,
            pipeline_tag: None,
            extras: Default::default(),
        },
        LocalMaxModel {
            id: "llama3.1-8b".to_string(),
            hf_id: "meta-llama/Llama-3.1-8B".to_string(),
            display_name: Some("Llama 3.1 8B".to_string()),
            organization: "Meta".to_string(),
            family: Some("llama".to_string()),
            params: Some(8.0),
            is_moe: false,
            tags: None,
            _count: None,
            speed_stats: None,
            base_model: None,
            active_params: None,
            pipeline_tag: None,
            extras: Default::default(),
        },
        LocalMaxModel {
            id: "deepseek-r1".to_string(),
            hf_id: "deepseek-ai/DeepSeek-R1".to_string(),
            display_name: Some("DeepSeek R1".to_string()),
            organization: "DeepSeek".to_string(),
            family: Some("deepseek".to_string()),
            params: Some(72.0),
            is_moe: false,
            tags: None,
            _count: None,
            speed_stats: None,
            base_model: None,
            active_params: None,
            pipeline_tag: None,
            extras: Default::default(),
        },
    ]
}