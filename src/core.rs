use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct OllamaModel {
    pub name: String,
    pub model: String,
    pub modified_at: Option<String>,
    pub size: Option<u64>,
    #[serde(default)]
    pub details: Option<ModelDetails>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ModelDetails {
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub parameter_size: Option<String>,
    #[serde(default)]
    pub quantization_level: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub os: String,
    pub ram_gb: f64,
    pub total_vram_mb: u64,
    pub gpus: Vec<GpuInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
}

impl HardwareInfo {
    pub fn has_gpu(&self) -> bool {
        !self.gpus.is_empty()
    }

    #[allow(dead_code)]
    pub fn can_run_on_gpu(&self, model_size_gb: f64) -> bool {
        self.total_vram_mb as f64 / 1024.0 >= model_size_gb + 0.5
    }
}

pub fn detect_hardware() -> HardwareInfo {
    let os = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    
    let ram_gb = {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_memory();
        sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0)
    };

    let gpus = detect_gpus().unwrap_or_else(|_| vec![]);
    let total_vram_mb = gpus.iter().map(|g| g.vram_mb).sum();

    HardwareInfo {
        os,
        ram_gb,
        total_vram_mb,
        gpus,
    }
}

fn detect_gpus() -> Result<Vec<GpuInfo>> {
    // Essayer nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let mut gpus = Vec::new();

        for line in text.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let name = parts[0].trim().to_string();
                if let Ok(vram_mb) = parts[1].trim().parse::<u64>() {
                    gpus.push(GpuInfo { name, vram_mb });
                }
            }
        }

        if !gpus.is_empty() {
            return Ok(gpus);
        }
    }

    // Fallback : pas de GPU détecté
    Ok(vec![])
}

#[allow(dead_code)]
pub fn detect_tool(name: &str) -> bool {
    which::which(name).is_ok()
}

pub fn run_command(cmd: &str) -> Result<(String, String)> {
    // Détecter le shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    
    let output = if shell.contains("fish") {
        // Fish a besoin de -c pour exécuter des commandes
        Command::new("fish")
            .args(["-c", cmd])
            .output()
            .with_context(|| "Erreur d'exécution de la commande avec fish")?
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .output()
            .with_context(|| "Erreur d'exécution de la commande avec cmd")?
    } else {
        // Bash/Zsh/sh
        Command::new("sh")
            .args(["-c", cmd])
            .output()
            .with_context(|| "Erreur d'exécution de la commande avec sh")?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok((stdout, stderr))
    } else {
        // Message d'erreur plus informatif
        let shell_name = if shell.contains("fish") { "Fish" } 
                        else if shell.contains("zsh") { "Zsh" } 
                        else { "Bash/Sh" };
        
        Err(anyhow::anyhow!(
            "Commande échouée (shell: {}):\n  {}\n\nErreur:\n{}", 
            shell_name, cmd, stderr
        ))
    }
}

pub fn estimate_tokens_book(pages: u32) -> u64 {
    (pages as u64) * 550 // ~550 tokens par page
}

pub fn estimate_tokens_code(loc: u32) -> u64 {
    (loc as u64) * 8 // ~8 tokens par ligne de code
}

#[allow(dead_code)]
pub fn estimate_chunks(tokens: u64, chunk_size: u64) -> u64 {
    (tokens + chunk_size - 1) / chunk_size
}

pub fn estimate_time_minutes(tokens: u64, tokens_per_second: f64) -> (f64, f64) {
    let seconds = tokens as f64 / tokens_per_second;
    let minutes = seconds / 60.0;
    let margin = 0.3; // ±30%
    (minutes * (1.0 - margin), minutes * (1.0 + margin))
}

pub fn get_performance(model_size: u32, use_gpu: bool) -> f64 {
    match (model_size, use_gpu) {
        (3, true) => 30.0,
        (3, false) => 8.0,
        (7, true) => 20.0,
        (7, false) => 5.0,
        (14, true) => 12.0,
        (14, false) => 2.0,
        (32, true) => 8.0,
        (32, false) => 1.0,
        (70, true) => 4.0,
        (70, false) => 0.5,
        _ => 10.0, // valeure par défaut
    }
}

pub fn recommend_model_size(
    usage_type: &str,
    hardware: &HardwareInfo,
) -> (u32, bool) {
    let available_ram_gb = if hardware.has_gpu() {
        hardware.total_vram_mb as f64 / 1024.0
    } else {
        hardware.ram_gb
    };

    match usage_type {
        "agents" => {
            // Privilégier les petits modèles rapides
            if available_ram_gb >= 6.0 {
                (7, hardware.has_gpu())
            } else {
                (3, false)
            }
        }
        "book" | "code" => {
            // Qualité avant tout
            if available_ram_gb >= 20.0 {
                (32, hardware.has_gpu())
            } else if available_ram_gb >= 10.0 {
                (14, hardware.has_gpu())
            } else if available_ram_gb >= 6.0 {
                (7, hardware.has_gpu())
            } else {
                (3, false)
            }
        }
        _ => {
            // Usage général
            if available_ram_gb >= 10.0 {
                (14, hardware.has_gpu())
            } else if available_ram_gb >= 6.0 {
                (7, hardware.has_gpu())
            } else {
                (3, false)
            }
        }
    }
}

pub fn run_benchmark() -> Result<()> {
    println!("Fonctionnalité de benchmark à implémenter");
    println!("Cette fonctionnalité nécessite qu'Ollama soit installé et en cours d'exécution");
    Ok(())
}

/// Récupère les modèles disponibles localement via l'API Ollama
pub fn fetch_local_models(base_url: &str) -> Result<Vec<OllamaModel>> {
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .context("Erreur création client HTTP")?;
    
    let resp = client
        .get(&url)
        .send()
        .context("Impossible de contacter Ollama. Vérifiez qu'il est lancé.")?;
    
    let resp = resp
        .error_for_status()
        .context("Réponse HTTP invalide de l'API Ollama")?;
    
    let data: OllamaTagsResponse = resp
        .json()
        .context("Parsing JSON /api/tags échoué")?;
    
    Ok(data.models)
}

/// Vérifie si Ollama est lancé et retourne l'URL de base
pub fn detect_ollama_url() -> Option<String> {
    let urls = [
        "http://localhost:11434",
        "http://127.0.0.1:11434",
    ];
    
    for url in &urls {
        if let Ok(resp) = reqwest::blocking::get(format!("{}/api/tags", url)) {
            if resp.status().is_success() {
                return Some(url.to_string());
            }
        }
    }
    None
}

/// Trouve le meilleur modèle local correspondant à un usage
pub fn pick_best_local_model(
    models: &[OllamaModel],
    usage_type: &str,
    preferred_size: u32,
) -> Option<OllamaModel> {
    let preferred = match usage_type {
        "code" => vec!["qwen2.5-coder", "codellama", "deepseek-coder", "starcoder"],
        "book" => vec!["qwen2.5", "mistral", "llama3", "mixtral"],
        "agents" => vec!["qwen2.5", "phi3", "gemma"],
        _ => vec!["qwen2.5", "mistral", "llama3"],
    };

    models
        .iter()
        .filter(|m| {
            let name = m.name.to_lowercase();
            preferred.iter().any(|p| name.contains(p))
        })
        .max_by(|a, b| {
            // Privilégier la taille la plus proche
            let size_a = extract_size(&a.name);
            let size_b = extract_size(&b.name);
            
            let diff_a = (size_a as i64 - preferred_size as i64).abs();
            let diff_b = (size_b as i64 - preferred_size as i64).abs();
            
            diff_a.cmp(&diff_b).then(size_b.cmp(&size_a))
        })
        .cloned()
}

pub fn extract_size(model_name: &str) -> u32 {
    // Extraire "7b", "14b", etc.
    for part in model_name.split([':', '-', '/']) {
        if let Some(size) = part.strip_suffix('b') {
            if let Ok(n) = size.parse::<u32>() {
                return n;
            }
        }
    }
    0
}

/// Score basé uniquement sur les métadonnées, sans nom en dur
pub fn score_model_dynamic(
    model: &OllamaModel,
    usage_type: &str,
    hardware: &HardwareInfo,
) -> f32 {
    let name = model.name.to_lowercase();
    let size = extract_size(&model.name);

    // Exclure les modèles cloud/remote
    if size == 0 && (name.contains("cloud") || name.contains("remote")) {
        return 0.0;
    }

    let available_ram = if hardware.has_gpu() {
        hardware.total_vram_mb as f64 / 1024.0
    } else {
        hardware.ram_gb
    };

    let mut score: f32 = 0.2; // base

    // 1. Taille adaptée à la RAM/VRAM disponible
    let size_score = if (size as f64 * 2.0) <= available_ram {
        if (size as f64 * 2.0) <= available_ram * 0.3 {
            0.4 // très très confortable (< 30% RAM)
        } else if (size as f64 * 2.0) <= available_ram * 0.5 {
            0.25 // confortable
        } else {
            0.1 // ça passe
        }
    } else {
        -1.0 // éliminé
    };
    score += size_score;
    if score < 0.0 { return 0.0; }

    // 2. Taille adaptée à l'usage
    match usage_type {
        "agents" => {
            if size <= 7 { score += 0.2; } // petit = rapide
            else if size <= 14 { score += 0.1; }
        }
        "book" | "code" => {
            if size >= 32 { score += 0.3; } // grand = qualitatif
            else if size >= 14 { score += 0.2; }
            else if size >= 7 { score += 0.05; }
        }
        _ => {
            if size >= 14 { score += 0.15; }
            else if size >= 7 { score += 0.1; }
        }
    }

    // 3. Famille : bonus selon mot-clés dans le nom/famille
    let family_hints = model
        .details
        .as_ref()
        .and_then(|d| d.family.as_deref())
        .unwrap_or("");
    
    // Mots-clés positifs (indépendants de la famille)
    let positive_keywords = ["instruct", "chat", "latest"];
    for kw in &positive_keywords {
        if name.contains(kw) || family_hints.contains(kw) {
            score += 0.05;
        }
    }
    
    // Mots-clés par usage (score bonus)
    let usage_keywords: &[&str] = match usage_type {
        "code" => &["code", "coder", "dev", "program"],
        "book" => &["writer", "story", "text", "prose", "large"],
        "agents" => &["small", "fast", "light", "mini", "tiny"],
        _ => &[],
    };
    for kw in usage_keywords {
        if name.contains(kw) || family_hints.contains(kw) {
            score += 0.1;
        }
    }

    // 4. Quantization : Q4 = bon compromis, Q8 = meilleure qualité
    let quant = model
        .details
        .as_ref()
        .and_then(|d| d.quantization_level.as_deref())
        .unwrap_or("");
    if quant.contains("Q4") { score += 0.05; }
    if quant.contains("Q8") || quant.contains("F16") { score += 0.1; }

    // 5. Modifié récemment = maintenu
    if model.modified_at.is_some() {
        score += 0.05;
    }

    score.min(1.0).max(0.0)
}

/// Trie TOUS les modèles locaux par pertinence pour un usage
pub fn rank_local_models(
    models: &[OllamaModel],
    usage_type: &str,
    hardware: &HardwareInfo,
    limit: usize,
) -> Vec<(OllamaModel, f32)> {
    let mut scored: Vec<(OllamaModel, f32)> = models
        .iter()
        .filter(|m| {
            let name = m.name.to_lowercase();
            let size = extract_size(&m.name);
            // Exclure les modèles cloud (taille 0 + mot-clé cloud/remote)
            !(size == 0 && (name.contains("cloud") || name.contains("remote")))
        })
        .map(|m| {
            let score = score_model_dynamic(m, usage_type, hardware);
            (m.clone(), score)
        })
        .filter(|(_, s)| *s > 0.0)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored
}