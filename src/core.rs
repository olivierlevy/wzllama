use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

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