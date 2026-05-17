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

/// Detect the Linux distribution
pub fn detect_distro() -> &'static str {
    if crate::core::shell::run("which apt apt-get 2>/dev/null").is_ok() {
        "debian"  // Debian, Ubuntu, Mint
    } else if crate::core::shell::run("which dnf 2>/dev/null").is_ok() {
        "fedora"  // Fedora
    } else if crate::core::shell::run("which yum 2>/dev/null").is_ok() {
        "rhel"    // RHEL, CentOS
    } else if crate::core::shell::run("which pacman 2>/dev/null").is_ok() {
        "arch"    // Arch Linux, Manjaro
    } else if crate::core::shell::run("which zypper 2>/dev/null").is_ok() {
        "opensuse"
    } else if crate::core::shell::run("which emerge 2>/dev/null").is_ok() {
        "gentoo"
    } else if crate::core::shell::run("which xbps-install 2>/dev/null").is_ok() {
        "void"
    } else if std::path::Path::new("/etc/nixos/configuration.nix").exists() {
        "nixos"
    } else {
        "unknown"
    }
}
