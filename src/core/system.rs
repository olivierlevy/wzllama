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