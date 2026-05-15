use anyhow::Result;
use std::path::PathBuf;
use crate::core::{shell, ollama_api};
use crate::display;

pub struct OllamaDoctor;

impl OllamaDoctor {
    /// Vérifie et corrige les problèmes courants d'Ollama
    pub fn check_and_fix() -> Result<Vec<String>> {
        let mut fixes = vec![];
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/olivier".into());
        
        // 1. Check la clé ed25519
        let key_path = PathBuf::from(&home).join(".ollama/id_ed25519");
        if !key_path.exists() {
            display::warning("Clé ed25519 manquante, génération...");
            Self::generate_key(&key_path)?;
            fixes.push("Clé ed25519 générée".into());
        }
        
        // 2. Check si le port est déjà utilisé (autre instance)
        let port_used = shell::run("lsof -ti :11434 2>/dev/null").map(|(o, _)| !o.trim().is_empty()).unwrap_or(false);
        if port_used {
            let (pids, _) = shell::run("lsof -ti :11434 2>/dev/null")?;
            shell::run(&format!("sudo kill {}", pids.trim()))?;
            std::thread::sleep(std::time::Duration::from_secs(1));
            fixes.push("Ancienne instance Ollama arrêtée".into());
        }
        
        // 3. Check si systemd est bloqué en restart loop
        let failed = shell::run("systemctl is-failed ollama.service 2>/dev/null").is_ok();
        if failed {
            shell::run("sudo systemctl reset-failed ollama.service 2>/dev/null")?;
            fixes.push("Service systemd réinitialisé".into());
        }
        
        // 4. Démarrer si nécessaire
        if !ollama_api::detect_url().is_some() {
            shell::run("sudo systemctl start ollama")?;
            for _ in 0..10 {
                if ollama_api::detect_url().is_some() { break; }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            fixes.push("Ollama démarré".into());
        }
        
        Ok(fixes)
    }
    
    fn generate_key(key_path: &PathBuf) -> Result<()> {
        if let Some(parent) = key_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Lancer ollama serve en arrière-plan pour générer la clé
        let mut child = std::process::Command::new("ollama")
            .arg("serve")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        
        for _ in 0..30 {
            if key_path.exists() { break; }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        
        let _ = child.kill();
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        // Nettoyer le port
        let _ = shell::run("sudo kill $(lsof -ti :11434 2>/dev/null) 2>/dev/null");
        
        Ok(())
    }
    
    /// Vérifie si Ollama est en bonne santé
    #[allow(dead_code)]
    pub fn is_healthy() -> bool {
        ollama_api::detect_url().is_some()
    }
}