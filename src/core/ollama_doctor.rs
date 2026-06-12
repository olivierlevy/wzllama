use crate::core::ollama_api;
use crate::display;
use anyhow::Result;

pub struct OllamaDoctor;

impl OllamaDoctor {
    /// Vérifie et corrige les problèmes courants d'Ollama
    pub fn check_and_fix() -> Result<Vec<String>> {
        let mut fixes = vec![];

        #[cfg(unix)]
        {
            use crate::core::shell;
            let home = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/home"))
                .to_string_lossy()
                .to_string();

            // 1. Check la clé ed25519
            let key_path = PathBuf::from(&home).join(".ollama/id_ed25519");
            if !key_path.exists() {
                display::warning("Clé ed25519 manquante, génération...");
                Self::generate_key_unix(&key_path)?;
                fixes.push("Clé ed25519 générée".into());
            }

            // 2. Check si le port est déjà utilisé (autre instance)
            if let Ok((pids, _)) = shell::run_quiet("lsof -ti :11434 2>/dev/null") {
                let pids = pids.trim();
                if !pids.is_empty() {
                    let _ = shell::run(&format!("sudo kill {}", pids));
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    fixes.push("Ancienne instance Ollama arrêtée".into());
                }
            }

            // 3. Check si systemd est bloqué en restart loop
            if shell::run_quiet("systemctl is-failed ollama.service 2>/dev/null").is_ok() {
                let _ = shell::run_quiet("sudo systemctl reset-failed ollama.service 2>/dev/null");
                fixes.push("Service systemd reset".into());
            }

            // 4. Démarrer si nécessaire
            if ollama_api::detect_url().is_none() {
                let _ = shell::run_quiet("sudo systemctl start ollama");
                for _ in 0..10 {
                    if ollama_api::detect_url().is_some() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                fixes.push("Ollama started".into());
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, Ollama manages its own keys and service.
            // Just ensure it's running; start it if not.
            if ollama_api::detect_url().is_none() {
                display::info("Starting Ollama...");
                let started = std::process::Command::new("ollama")
                    .arg("serve")
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();

                match started {
                    Ok(_) => {
                        for _ in 0..15 {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            if ollama_api::detect_url().is_some() {
                                break;
                            }
                        }
                        fixes.push("Ollama started".into());
                    }
                    Err(e) => {
                        log::warn!("Could not start ollama automatically: {}", e);
                    }
                }
            }
        }

        Ok(fixes)
    }

    #[cfg(unix)]
    fn generate_key_unix(key_path: &std::path::Path) -> Result<()> {
        if let Some(parent) = key_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Start ollama serve briefly to trigger key generation
        let mut child = std::process::Command::new("ollama")
            .arg("serve")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        for _ in 0..30 {
            if key_path.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        let _ = child.kill();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let _ =
            crate::core::shell::run_quiet("sudo kill $(lsof -ti :11434 2>/dev/null) 2>/dev/null");
        Ok(())
    }

    /// Vérifie si Ollama est en bonne santé
    #[allow(dead_code)]
    pub fn is_healthy() -> bool {
        ollama_api::detect_url().is_some()
    }
}
