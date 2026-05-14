use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::os::unix::process::CommandExt;
use colored::Colorize;

/// Sort du mode raw du terminal si nous sommes dedans
/// À appeler avant d'exécuter des commandes qui nécessitent un terminal propre
pub fn exit_raw_mode() {
    let _ = crossterm::terminal::disable_raw_mode();
}

/// Réinitialise l'affichage du terminal (rend visible le curseur)
pub fn reset_terminal() {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(std::io::stdout(), 
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::Show
    );
    println!(); // Ligne vide pour l'invite
}

/// Context for shell execution - allows redirecting output to TUI terminal
pub struct ShellContext {
    pub output: Option<Arc<Mutex<String>>>,
}

impl ShellContext {
    pub fn new() -> Self {
        Self { output: None }
    }
    
    pub fn with_output(output: Arc<Mutex<String>>) -> Self {
        Self { output: Some(output) }
    }
}

impl Default for ShellContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Exécute une commande avec un contexte (peut rediriger vers le terminal intégré)
pub fn run_with_context(cmd: &str, ctx: Option<&ShellContext>) -> Result<()> {
    if let Some(ctx) = ctx {
        if let Some(ref output) = ctx.output {
            // Utiliser le terminal intégré - la commande doit être écrite avant
            let _ = run_sync_with_output(cmd, output);
        } else {
            // Mode normal - sortir du raw mode
            exit_raw_mode();
            println!("   ⏳ {}", cmd.dimmed());
            let _ = run_live(cmd);
        }
    } else {
        // Pas de contexte - mode normal
        exit_raw_mode();
        println!("   ⏳ {}", cmd.dimmed());
        let _ = run_live(cmd);
    }
    Ok(())
}

/// Exécute une commande en affichant sa sortie en temps réel (pour ollama pull)
pub fn run_live(cmd: &str) -> Result<()> {
    exit_raw_mode();
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

    // Lire stdout et stderr en même temps dans des threads séparés
    let stdout_handle = if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        Some(thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    if !line.trim().is_empty() {
                        println!("   {}", line.dimmed());
                    }
                }
            }
        }))
    } else {
        None
    };

    let stderr_handle = if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        Some(thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    if !line.trim().is_empty() {
                        // Afficher les barres de progression et messages
                        println!("   {}", line.dimmed());
                    }
                }
            }
        }))
    } else {
        None
    };

    // Attendre les threads de lecture
    if let Some(handle) = stdout_handle {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Commande échouée avec code {}", status))
    }
}

/// Exécute une commande et écrit la sortie dans un buffer partagé
/// Version asynchrone (non bloquante) - lance la commande dans un thread
/// La commande ($ cmd) doit être écrite AVANT l'appel (pour un affichage immédiat)
pub fn run_sync_with_output(cmd: &str, output: &Arc<Mutex<String>>) -> Result<()> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let output = Arc::clone(output);
    let cmd = cmd.to_string();

    thread::spawn(move || {
        let output_result = if shell.contains("fish") {
            Command::new("fish").args(["-c", &cmd]).output()
        } else {
            Command::new("sh").args(["-c", &cmd]).output()
        };

        if let Ok(output_result) = output_result {
            let stdout = String::from_utf8_lossy(&output_result.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output_result.stderr).to_string();
            
            let mut out = output.lock().unwrap();
            if !stdout.is_empty() {
                out.push_str(&stdout);
                out.push('\n');
            }
            if !stderr.is_empty() {
                out.push_str(&stderr);
                out.push('\n');
            }
        }
    });
    
    Ok(())
}

pub fn run_cmd(cmd: &str) -> Result<()> {
    exit_raw_mode();
    println!("{}", format!("{}", cmd).bright_black());
    let _ = run(cmd);
    Ok(())
}

pub fn run(cmd: &str) -> Result<(String, String)> {
    exit_raw_mode();
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

/// Exécute une commande en la forkant pour permettre les interactions (sudo, etc.)
/// Le processus parent attend que la commande se termine
pub fn spawn(cmd: &str) -> Result<()> {
    exit_raw_mode();
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    
    // Fork pour lancer la commande avec interaction terminal
    match unsafe { libc::fork() } {
        0 => {
            // Processus enfant - lancer la commande
            // setsid pour prendre le contrôle du terminal
            unsafe { libc::setsid(); }
            let _err = Command::new(&shell).args(["-i", "-c", cmd]).exec();
            std::process::exit(1);
        }
        -1 => {
            Err(anyhow::anyhow!("Failed to fork"))
        }
        _ => {
            // Processus parent - attendre
            let mut status: i32 = 0;
            unsafe { libc::wait(&mut status); }
            if status == 0 {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Commande échouée avec code {}", status))
            }
        }
    }
}

pub fn is_installed(cmd: &str) -> bool {
    run(&format!("command -v {} 2>/dev/null", cmd)).is_ok()
}

#[allow(dead_code)]
pub fn detect_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("fish") { return "fish".into(); }
        if shell.contains("zsh") { return "zsh".into(); }
        if shell.contains("bash") { return "bash".into(); }
    }
    "unknown".into()
}

/// Exécute une commande en remplaçant le processus courant (ne retourne jamais)
/// Avant l'exécution, sort du mode raw et réinitialise le terminal
pub fn exec(cmd: &str) -> ! {
    // Sortir du mode raw et réinitialiser le terminal avant l'exec
    reset_terminal();
    
    // Si la commande est bash, zsh, ou sh, on lance le shell directement (interactif)
    if cmd == "bash" || cmd == "zsh" || cmd == "sh" {
        if std::path::Path::new("/bin/bash").exists() {
            let err = Command::new("/bin/bash").exec();
            panic!("exec failed: {}", err);
        } else if std::path::Path::new("/bin/zsh").exists() {
            let err = Command::new("/bin/zsh").exec();
            panic!("exec failed: {}", err);
        } else {
            let err = Command::new("/bin/sh").exec();
            panic!("exec failed: {}", err);
        }
    }
    
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let err = Command::new(&shell).arg("-c").arg(cmd).exec();
    // exec ne retourne qu'en cas d'erreur
    panic!("exec failed: {}", err);
}

/// Lance une application depuis le TUI en sortant proprement du mode raw
/// Usage: depuis le TUI, appel `spawn_and_exit("ollama run qwen")`
/// Cette fonction ne retourne jamais - après la commande, le programme se termine.
pub fn spawn_and_exit(cmd: &str) -> ! {
    // Sortir du mode raw et réinitialiser le terminal
    reset_terminal();
    
    // Lancer la commande dans un nouveau shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let err = Command::new(&shell).arg("-c").arg(cmd).exec();
    panic!("spawn_and_exit failed: {}", err);
}

/// Lance un shell interactif depuis le TUI
/// Utilise fork+exec pour donner le contrôle du terminal au shell
pub fn launch_interactive_shell() -> Result<()> {
    // Sortir du mode raw et réinitialiser le terminal
    reset_terminal();
    
    // Essayer de lancer bash, zsh ou sh
    let shell = if std::path::Path::new("/bin/bash").exists() {
        "/bin/bash"
    } else if std::path::Path::new("/bin/zsh").exists() {
        "/bin/zsh"
    } else {
        "/bin/sh"
    };
    
    // Fork et exec pour lancer le shell interactif
    match unsafe { libc::fork() } {
        0 => {
            // Processus enfant - lancer le shell interactif
            unsafe { libc::setsid(); }
            let _err = Command::new(shell).arg("-i").exec();
            std::process::exit(1);
        }
        -1 => {
            Err(anyhow::anyhow!("Failed to fork"))
        }
        _ => {
            // Processus parent - attendre
            let mut status: i32 = 0;
            unsafe { libc::wait(&mut status); }
            Ok(())
        }
    }
}

/// Ouvre une URL dans le navigateur par défaut du système
pub fn open_url(url: &str) {
    let cmd = if cfg!(target_os = "linux") {
        format!("xdg-open {}", url)
    } else if cfg!(target_os = "macos") {
        format!("open {}", url)
    } else if cfg!(target_os = "windows") {
        format!("start {}", url)
    } else {
        return;
    };
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .spawn();
}
