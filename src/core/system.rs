use anyhow::{Result, bail};
use sysinfo::System;

pub fn get_available_ram_gb() -> f64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.available_memory() as f64 / (1024.0 * 1024.0 * 1024.0)
}

pub fn get_available_vram_gb() -> Option<f64> {
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.free", "--format=csv,noheader,nounits"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        if let Ok(mb) = text.trim().parse::<f64>() {
            return Some(mb / 1024.0);
        }
    }
    None
}

pub fn detect_package_manager() -> String {
    for (cmd, name) in &[("pacman", "pacman"), ("apt", "apt"), ("dnf", "dnf"), ("brew", "brew")] {
        if crate::core::shell::is_installed(cmd) {
            return name.to_string();
        }
    }
    "unknown".into()
}

/// Reruns la commande d'installation du paquet système via le gestionnaire détecté
pub fn get_package_install_command(pkg: &str) -> Result<String> {
    let pm = detect_package_manager();
    let cmd = match pm.as_str() {
        "pacman" => format!("sudo pacman -S --noconfirm {}", pkg),
        "apt" => format!("sudo apt install -y {}", pkg),
        "dnf" => format!("sudo dnf install -y {}", pkg),
        "brew" => format!("brew install {}", pkg),
        _ => bail!("Gestionnaire de paquets inconnu"),
    };
    Ok(cmd)
}
