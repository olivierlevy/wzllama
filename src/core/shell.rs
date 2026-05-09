use anyhow::{Context, Result};
use std::process::Command;

pub fn run(cmd: &str) -> Result<(String, String)> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());

    let output = if shell.contains("fish") {
        Command::new("fish").args(["-c", cmd]).output()
            .with_context(|| "Erreur fish")?
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()
            .with_context(|| "Erreur cmd")?
    } else {
        Command::new("sh").args(["-c", cmd]).output()
            .with_context(|| "Erreur sh")?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok((stdout, stderr))
    } else {
        let name = if shell.contains("fish") { "Fish" } else if shell.contains("zsh") { "Zsh" } else { "Bash/Sh" };
        Err(anyhow::anyhow!("Commande échouée ({}): {}", name, stderr))
    }
}

pub fn is_installed(cmd: &str) -> bool {
    run(&format!("command -v {} 2>/dev/null", cmd)).is_ok()
}

pub fn detect_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("fish") { return "fish".into(); }
        if shell.contains("zsh") { return "zsh".into(); }
        if shell.contains("bash") { return "bash".into(); }
    }
    "unknown".into()
}