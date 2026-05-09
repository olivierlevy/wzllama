use anyhow::Result;
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
    pub fn has_gpu(&self) -> bool { !self.gpus.is_empty() }
}

pub fn detect() -> HardwareInfo {
    let os = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    let ram_gb = {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_memory();
        sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0)
    };
    let gpus = detect_gpus().unwrap_or_default();
    let total_vram_mb = gpus.iter().map(|g| g.vram_mb).sum();
    HardwareInfo { os, ram_gb, total_vram_mb, gpus }
}

fn detect_gpus() -> Result<Vec<GpuInfo>> {
    if let Ok(output) = Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let mut gpus = Vec::new();
        for line in text.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(vram_mb) = parts[1].trim().parse::<u64>() {
                    gpus.push(GpuInfo { name: parts[0].trim().into(), vram_mb });
                }
            }
        }
        if !gpus.is_empty() { return Ok(gpus); }
    }
    Ok(vec![])
}