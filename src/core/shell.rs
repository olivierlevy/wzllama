#![allow(dead_code)]

use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{BufRead, BufReader};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Sort du mode raw du terminal si nous sommes dedans
/// À appeler avant d'exécuter des commandes qui nécessitent un terminal propre
pub fn exit_raw_mode() {
    // No-op: raw mode handling removed with TUI
}

/// Réinitialise l'affichage du terminal (rend visible le curseur)
pub fn reset_terminal() {
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
        Self {
            output: Some(output),
        }
    }
}

impl Default for ShellContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Executes une commande avec un contexte (peut rediriger vers le terminal intégré)
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

/// Executes une commande en affichant sa sortie en temps réel (pour ollama pull)
pub fn run_live(cmd: &str) -> Result<()> {
    exit_raw_mode();
    println!("   ⏳ {}", cmd.dimmed());

    // Use appropriate shell for the platform
    #[cfg(unix)]
    let mut child = Command::new("sh")
        .args(["-c", cmd])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    #[cfg(not(unix))]
    let mut child = {
        let shell = std::env::var("SHELL")
            .unwrap_or_else(|_| std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into()));
        Command::new(&shell)
            .args(["/C", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
    };

    // Lire stdout et stderr en même temps dans des threads séparés
    let stdout_handle = if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        Some(thread::spawn(move || {
            for line in reader.lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    println!("   {}", line.dimmed());
                }
            }
        }))
    } else {
        None
    };

    let stderr_handle = if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        Some(thread::spawn(move || {
            for line in reader.lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    // Afficher les barres de progression et messages
                    println!("   {}", line.dimmed());
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

/// Executes une commande et écrit la sortie dans un buffer partagé
/// Version asynchrone (non bloquante) - lance la commande dans un thread
/// La commande ($ cmd) doit être écrite AVANT l'appel (pour un affichage immédiat)
pub fn run_sync_with_output(cmd: &str, output: &Arc<Mutex<String>>) -> Result<()> {
    let output = Arc::clone(output);
    let cmd = cmd.to_string();

    thread::spawn(move || {
        // Use appropriate shell for the platform
        #[cfg(unix)]
        let output_result = Command::new("sh").args(["-c", &cmd]).output();

        #[cfg(not(unix))]
        let output_result = {
            let shell = std::env::var("SHELL")
                .unwrap_or_else(|_| std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into()));
            Command::new(&shell).args(["/C", &cmd]).output()
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
    println!("{}", cmd.to_string().bright_black());
    let _ = run(cmd);
    Ok(())
}

pub fn run(cmd: &str) -> Result<(String, String)> {
    exit_raw_mode();
    // Use appropriate shell for the platform
    #[cfg(unix)]
    let output = Command::new("sh")
        .args(["-c", cmd])
        .output()
        .with_context(|| "Sh error")?;

    #[cfg(not(unix))]
    let output = {
        let shell = std::env::var("SHELL")
            .unwrap_or_else(|_| std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into()));
        Command::new(&shell)
            .args(["/C", cmd])
            .output()
            .with_context(|| "Shell error")?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok((stdout, stderr))
    } else {
        Err(anyhow::anyhow!("Command failed (sh): {}", stderr))
    }
}

/// Executes une commande en la forkant pour permettre les interactions (sudo, etc.)
/// Le processus parent attend que la commande se termine
#[cfg(unix)]
pub fn spawn(cmd: &str) -> Result<()> {
    exit_raw_mode();
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());

    // Fork pour lancer la commande avec interaction terminal
    match unsafe { libc::fork() } {
        0 => {
            // Processus enfant - lancer la commande
            // setsid pour prendre le contrôle du terminal
            unsafe {
                libc::setsid();
            }
            let _err = Command::new(&shell).args(["-i", "-c", cmd]).exec();
            std::process::exit(1);
        }
        -1 => Err(anyhow::anyhow!("Failed to fork")),
        _ => {
            // Processus parent - attendre
            let mut status: i32 = 0;
            unsafe {
                libc::wait(&mut status);
            }
            if status == 0 {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Commande échouée avec code {}", status))
            }
        }
    }
}

/// Windows version: uses spawn instead of fork for interactive commands
#[cfg(not(unix))]
pub fn spawn(cmd: &str) -> Result<()> {
    exit_raw_mode();
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "cmd.exe".into());

    // On Windows, we use the shell with /C flag (no fork available)
    let status = Command::new(&shell)
        .args(["/C", cmd])
        .status()
        .with_context(|| "Failed to spawn command")?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Commande échouée"))
    }
}

pub fn is_installed(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Check if a command is installed without exiting raw mode
pub fn is_installed_quiet(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Check if a command is installed, including in ~/.local/bin and other common locations
pub fn is_installed_with_local_bin(cmd: &str) -> bool {
    // Fast path: check PATH first via which
    if which::which(cmd).is_ok() {
        return true;
    }

    // Check well-known install locations not always on PATH
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

    #[cfg(unix)]
    let extra_paths = vec![
        home.join(".local/bin").join(cmd),
        home.join(".opencode/bin").join(cmd),
        home.join(".factoryai/bin").join(cmd),
        home.join("go/bin").join(cmd),
        std::path::PathBuf::from("/usr/local/bin").join(cmd),
        std::path::PathBuf::from("/usr/bin").join(cmd),
    ];

    #[cfg(not(unix))]
    let extra_paths = vec![
        home.join("AppData")
            .join("Local")
            .join("Programs")
            .join(cmd)
            .with_extension("exe"),
        home.join("AppData")
            .join("Roaming")
            .join("npm")
            .join(cmd)
            .with_extension("cmd"),
        home.join("AppData").join("Roaming").join("npm").join(cmd),
        home.join(".local")
            .join("bin")
            .join(cmd)
            .with_extension("exe"),
    ];

    extra_paths.iter().any(|p| p.exists())
}

/// Run a command without exiting raw mode (for internal use)
/// Uses sh explicitly for compatibility across all shells (fish, zsh, bash)
pub fn run_quiet(cmd: &str) -> Result<(String, String)> {
    // Use appropriate shell for the platform
    #[cfg(unix)]
    let output = Command::new("sh")
        .args(["-c", cmd])
        .output()
        .with_context(|| "Sh error")?;

    #[cfg(not(unix))]
    let output = {
        let shell = std::env::var("SHELL")
            .unwrap_or_else(|_| std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into()));
        Command::new(&shell)
            .args(["/C", cmd])
            .output()
            .with_context(|| "Shell error")?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok((stdout, stderr))
    } else {
        Err(anyhow::anyhow!("Command failed: {}", stderr))
    }
}

#[allow(dead_code)]
pub fn detect_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("fish") {
            return "fish".into();
        }
        if shell.contains("zsh") {
            return "zsh".into();
        }
        if shell.contains("bash") {
            return "bash".into();
        }
    }
    // On Windows, check for common shells
    #[cfg(windows)]
    {
        if let Ok(shell) = std::env::var("COMSPEC") {
            if shell.contains("powershell") {
                return "powershell".into();
            }
            if shell.contains("cmd") {
                return "cmd".into();
            }
        }
    }
    "unknown".into()
}

/// Get the home directory in a cross-platform way
pub fn get_home_dir() -> String {
    #[cfg(unix)]
    {
        std::env::var("HOME").unwrap_or_else(|_| "/home".to_string())
    }
    #[cfg(not(unix))]
    {
        std::env::var("USERPROFILE")
            .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    }
}

/// Executes une commande en remplaçant le processus courant (ne reruns jamais)
/// Avant l'exécution, sort du mode raw et réinitialise le terminal
#[cfg(unix)]
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
    // exec ne reruns qu'en cas d'erreur
    panic!("exec failed: {}", err);
}

/// Windows version: spawns a new process and exits
#[cfg(not(unix))]
pub fn exec(cmd: &str) -> ! {
    // Sortir du mode raw et réinitialiser le terminal
    reset_terminal();

    // Sur Windows, on utilise cmd.exe /C ou powershell
    let shell = if cfg!(target_os = "windows") {
        std::env::var("SHELL").unwrap_or_else(|_| "cmd.exe".into())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into())
    };

    // Spawn the command and exit
    let result = if cfg!(target_os = "windows") {
        Command::new(&shell).args(["/C", cmd]).spawn()
    } else {
        Command::new(&shell).args(["-c", cmd]).spawn()
    };

    if let Ok(mut child) = result {
        let _ = child.wait();
    }

    std::process::exit(0);
}

/// Lance une application depuis le TUI en sortant proprement du mode raw
/// Usage: depuis le TUI, appel `spawn_and_exit("ollama run qwen")`
/// Cette fonction ne reruns jamais - après la commande, le programme se termine.
#[cfg(unix)]
pub fn spawn_and_exit(cmd: &str) -> ! {
    // Sortir du mode raw et réinitialiser le terminal
    reset_terminal();

    // Lancer la commande dans un nouveau shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let err = Command::new(&shell).arg("-c").arg(cmd).exec();
    panic!("spawn_and_exit failed: {}", err);
}

/// Windows version: spawns a new process and exits
#[cfg(not(unix))]
pub fn spawn_and_exit(cmd: &str) -> ! {
    // Sortir du mode raw et réinitialiser le terminal
    reset_terminal();

    // Sur Windows, on utilise cmd.exe /C
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "cmd.exe".into());
    let result = if cfg!(target_os = "windows") {
        Command::new(&shell).args(["/C", cmd]).spawn()
    } else {
        Command::new(&shell).args(["-c", cmd]).spawn()
    };

    if let Ok(mut child) = result {
        let _ = child.wait();
    }

    std::process::exit(0);
}

/// Lance un shell interactif depuis le TUI
/// Utilise fork+exec pour donner le contrôle du terminal au shell
#[cfg(unix)]
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
            unsafe {
                libc::setsid();
            }
            let _err = Command::new(shell).arg("-i").exec();
            std::process::exit(1);
        }
        -1 => Err(anyhow::anyhow!("Failed to fork")),
        _ => {
            // Processus parent - attendre
            let mut status: i32 = 0;
            unsafe {
                libc::wait(&mut status);
            }
            Ok(())
        }
    }
}

/// Windows version: spawns an interactive shell
#[cfg(not(unix))]
pub fn launch_interactive_shell() -> Result<()> {
    // Sortir du mode raw et réinitialiser le terminal
    reset_terminal();

    // On Windows, utilise cmd.exe ou powershell
    let shell = if std::env::var("SHELL").is_ok() {
        std::env::var("SHELL").unwrap()
    } else if std::env::var("COMSPEC").is_ok() {
        std::env::var("COMSPEC").unwrap()
    } else {
        "cmd.exe".into()
    };

    // Spawn interactive shell
    let status = if cfg!(target_os = "windows") {
        Command::new(&shell).args(["/Q"]).status()
    } else {
        Command::new(&shell).arg("-i").status()
    };

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => Err(anyhow::anyhow!("Shell exited with error")),
        Err(e) => Err(anyhow::anyhow!("Failed to launch shell: {}", e)),
    }
}

/// Ouvre une URL dans le navigateur par défaut du système
pub fn open_url(url: &str) {
    #[cfg(unix)]
    let result = std::process::Command::new("sh")
        .arg("-c")
        .arg(if cfg!(target_os = "linux") {
            format!("xdg-open {}", url)
        } else if cfg!(target_os = "macos") {
            format!("open {}", url)
        } else {
            return;
        })
        .spawn();

    #[cfg(not(unix))]
    let result = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd.exe")
            .args(["/C", "start", &url])
            .spawn()
    } else {
        // Fallback for other platforms
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&format!("xdg-open {}", url))
            .spawn()
    };

    let _ = result;
}

#[cfg(test)]
mod tests {
    use super::is_installed_with_local_bin;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        vars: Vec<(&'static str, Option<OsString>)>,
    }

    impl EnvGuard {
        fn capture(keys: &[&'static str]) -> Self {
            Self {
                vars: keys
                    .iter()
                    .map(|key| (*key, std::env::var_os(key)))
                    .collect(),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in &self.vars {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    #[cfg(windows)]
    fn test_root(name: &str) -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .join("target")
            .join("shell-tests")
            .join(name)
    }

    #[test]
    #[cfg(windows)]
    fn detects_windows_home_local_bin_without_home_env() {
        let _guard = env_lock().lock().unwrap();
        let _env = EnvGuard::capture(&["HOME", "USERPROFILE", "APPDATA"]);
        let root = test_root("userprofile-local-bin");
        let bin = root.join(".local").join("bin");
        let cmd = "wzllama-home-local-bin-test";

        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join(format!("{cmd}.cmd")), "@echo off\r\n").unwrap();

        std::env::remove_var("HOME");
        std::env::set_var("USERPROFILE", &root);
        std::env::set_var("APPDATA", root.join("AppData").join("Roaming"));

        assert!(is_installed_with_local_bin(cmd));
    }

    #[test]
    #[cfg(windows)]
    fn detects_windows_npm_shim_from_appdata() {
        let _guard = env_lock().lock().unwrap();
        let _env = EnvGuard::capture(&["HOME", "USERPROFILE", "APPDATA"]);
        let root = test_root("appdata-npm");
        let appdata = root.join("AppData").join("Roaming");
        let npm_bin = appdata.join("npm");
        let cmd = "wzllama-appdata-npm-test";

        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&npm_bin).unwrap();
        fs::write(npm_bin.join(format!("{cmd}.cmd")), "@echo off\r\n").unwrap();

        std::env::remove_var("HOME");
        std::env::set_var("USERPROFILE", &root);
        std::env::set_var("APPDATA", &appdata);

        assert!(is_installed_with_local_bin(cmd));
    }
}
