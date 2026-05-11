use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::os::unix::process::CommandExt;
use colored::Colorize;

/// Exécute une commande en affichant sa sortie en temps réel (pour ollama pull)
pub fn run_live(cmd: &str) -> Result<()> {
    println!("   ⏳ {}", cmd.dimmed());
    
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    
    let mut child = if shell.contains("fish") {
        Command::new("fish").args(["-c", cmd])
            .stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn()?
    } else {
        Command::new("sh").args(["-c", cmd])
            .stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn()?
    };

    // Lire stdout en temps réel
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    if !line.trim().is_empty() {
                        println!("   {}", line.dimmed());
                    }
                }
            }
        });
    }

    // Lire stderr en temps réel (ollama pull écrit ses progrès sur stderr)
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    // Afficher les barres de progression et messages
                    println!("   {}", line.dimmed());
                }
            }
        }
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Commande échouée avec code {}", status))
    }
}

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

/// Exécute une commande en remplaçant le processus courant (ne retourne jamais)
pub fn exec(cmd: &str) -> ! {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let err = Command::new(&shell).arg("-c").arg(cmd).exec();
    // exec ne retourne qu'en cas d'erreur
    panic!("exec failed: {}", err);
}
