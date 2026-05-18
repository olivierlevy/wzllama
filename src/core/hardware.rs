#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(unix)]
#[allow(unused_imports)]
use std::os::unix::ffi::OsStrExt;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareInfo {
    pub os: String,
    pub ram_gb: f64,
    pub total_vram_mb: u64,
    pub gpus: Vec<GpuInfo>,
    /// Available disk space in GB at the models directory
    pub available_disk_gb: f64,
}

impl HardwareInfo {
    pub fn default_for_test() -> Self {
        Self {
            os: "linux x86_64".to_string(),
            ram_gb: 16.0,
            total_vram_mb: 0,
            gpus: vec![],
            available_disk_gb: 100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
}

impl HardwareInfo {
    pub fn has_gpu(&self) -> bool { !self.gpus.is_empty() }
}

/// Get available disk space in GB for a given path
pub fn get_available_disk_space_gb(path: &str) -> f64 {
    // Try using sysinfo's disk functions
    if let Some(disk) = sysinfo::Disks::new().into_iter().find(|d| d.mount_point().to_string_lossy().contains(path.split('/').next().unwrap_or("/"))) {
        let _ = disk;
        // sysinfo Disk gives us total and available space
    }
    
    // Fallback: use statvfs on Unix systems
    #[cfg(unix)]
    {
        use std::ffi::CString;
        
        let path_c = CString::new(path.as_bytes()).unwrap_or_else(|_| CString::new(b"/").unwrap());
        let mut stat = unsafe { std::mem::zeroed() };
        unsafe {
            if libc::statvfs(path_c.as_ptr(), &mut stat) == 0 {
                let available = stat.f_bavail as f64 * stat.f_frsize as f64;
                return available / (1024.0 * 1024.0 * 1024.0);
            }
        }
    }
    
    100.0 // Default to 100GB if can't determine
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
    
    // Check available disk space for Ollama models directory
    let available_disk_gb = get_available_disk_space_gb("/home/ollama")
        .max(get_available_disk_space_gb("/usr/share/ollama"))
        .max(get_available_disk_space_gb("/var/lib/ollama"));
    
    HardwareInfo { os, ram_gb, total_vram_mb, gpus, available_disk_gb }
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