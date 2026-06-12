#![allow(dead_code)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

const DEFAULT_PORT: u16 = 8787;
const DEFAULT_HOST: &str = "127.0.0.1";

/// System hardware information from llmfit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFitSystem {
    pub total_ram_gb: f64,
    pub available_ram_gb: f64,
    pub cpu_cores: u32,
    pub cpu_name: String,
    pub has_gpu: bool,
    pub gpu_vram_gb: Option<f64>,
    pub gpu_name: Option<String>,
    pub gpu_count: u32,
    pub unified_memory: bool,
    pub backend: String,
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFitNode {
    pub name: String,
    pub os: String,
}

/// System response from /api/v1/system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFitSystemResponse {
    pub node: LLMFitNode,
    pub system: LLMFitSystem,
}

/// Model fit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFitModel {
    pub name: String,
    pub provider: String,
    pub parameter_count: String,
    pub params_b: f64,
    pub context_length: u32,
    pub use_case: String,
    pub category: String,
    pub release_date: String,
    pub is_moe: bool,
    pub fit_level: String,
    pub fit_label: String,
    pub run_mode: String,
    pub run_mode_label: String,
    pub score: f64,
    pub estimated_tps: f64,
    pub runtime: String,
    pub runtime_label: String,
    pub best_quant: String,
    pub memory_required_gb: f64,
    pub memory_available_gb: f64,
    pub utilization_pct: f64,
}

/// Models response from /api/v1/models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFitModelsResponse {
    pub node: LLMFitNode,
    pub system: LLMFitSystem,
    pub total_models: u32,
    pub returned_models: u32,
    pub models: Vec<LLMFitModel>,
}

/// Installed runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFitRuntime {
    pub name: String,
    pub path: Option<String>,
}

/// Response from /api/v1/runtimes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFitRuntimesResponse {
    pub runtimes: Vec<LLMFitRuntime>,
}

/// Installed model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFitInstalledModel {
    pub name: String,
    pub runtime: String,
    pub path: Option<String>,
}

/// Response from /api/v1/installed_models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFitInstalledModelsResponse {
    pub models: Vec<LLMFitInstalledModel>,
}

/// Client for llmfit HTTP API
pub struct LLMFitClient {
    host: String,
    port: u16,
}

impl Default for LLMFitClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LLMFitClient {
    pub fn new() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
        }
    }

    pub fn with_port(port: u16) -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port,
        }
    }

    pub fn with_host(host: &str) -> Self {
        Self {
            host: host.to_string(),
            port: DEFAULT_PORT,
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Check if llmfit server is running
    pub fn is_running(&self) -> bool {
        let url = format!("{}/health", self.base_url());
        reqwest::blocking::get(&url).is_ok()
    }

    /// Get system information
    pub fn get_system(&self) -> Result<LLMFitSystemResponse> {
        let url = format!("{}/api/v1/system", self.base_url());
        let response = reqwest::blocking::get(&url)?;
        let system: LLMFitSystemResponse = response.json()?;
        Ok(system)
    }

    /// Get top models
    pub fn get_top_models(
        &self,
        limit: Option<u32>,
        min_fit: Option<&str>,
        use_case: Option<&str>,
    ) -> Result<Vec<LLMFitModel>> {
        let mut url = format!("{}/api/v1/models/top", self.base_url());
        let mut params = Vec::new();

        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(m) = min_fit {
            params.push(format!("min_fit={}", m));
        }
        if let Some(u) = use_case {
            params.push(format!("use_case={}", u));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response = reqwest::blocking::get(&url)?;
        let models_resp: LLMFitModelsResponse = response.json()?;
        Ok(models_resp.models)
    }

    /// Search models
    pub fn search_models(&self, query: &str, limit: Option<u32>) -> Result<Vec<LLMFitModel>> {
        let mut url = format!("{}/api/v1/models", self.base_url());
        let mut params = vec![format!("search={}", query)];

        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        url = format!("{}?{}", url, params.join("&"));

        let response = reqwest::blocking::get(&url)?;
        let models_resp: LLMFitModelsResponse = response.json()?;
        Ok(models_resp.models)
    }

    /// Get runtimes
    pub fn get_runtimes(&self) -> Result<Vec<LLMFitRuntime>> {
        let url = format!("{}/api/v1/runtimes", self.base_url());
        let response = reqwest::blocking::get(&url)?;
        let runtimes_resp: LLMFitRuntimesResponse = response.json()?;
        Ok(runtimes_resp.runtimes)
    }

    /// Get installed models
    pub fn get_installed_models(&self) -> Result<Vec<LLMFitInstalledModel>> {
        let url = format!("{}/api/v1/installed_models", self.base_url());
        let response = reqwest::blocking::get(&url)?;
        let installed_resp: LLMFitInstalledModelsResponse = response.json()?;
        Ok(installed_resp.models)
    }
}

/// Start llmfit server (HTTP mode)
pub fn start_server(port: Option<u16>) -> Result<()> {
    let port = port.unwrap_or(DEFAULT_PORT);

    // Check if already running
    let client = LLMFitClient::with_port(port);
    if client.is_running() {
        return Ok(());
    }

    // Start llmfit serve in background
    let _output = Command::new("llmfit")
        .args(["serve", "--port", &port.to_string()])
        .spawn()?;

    // Give it time to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    if !client.is_running() {
        return Err(anyhow!("Failed to start llmfit server"));
    }

    Ok(())
}

/// Stop llmfit server
pub fn stop_server() -> Result<()> {
    let _ = Command::new("pkill").args(["-f", "llmfit serve"]).output();
    Ok(())
}
