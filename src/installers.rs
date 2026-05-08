use crate::config::I18n;
use crate::core::{detect_tool, run_command};
use anyhow::Result;
use colored::*;
use dialoguer::Confirm;
use log::info;

pub struct Installer<'a> {
    i18n: &'a I18n,
    interactive: bool,
    shell: String,
}

impl<'a> Installer<'a> {
    pub fn new(i18n: &'a I18n, interactive: bool) -> Self {
        let shell = detect_shell();
        Self { i18n, interactive, shell }
    }

    pub fn check_and_install(&self, tool: &str) -> Result<()> {
        println!("{}", self.i18n.t_with_vars("install.checking", &[("tool", tool)]));

        if detect_tool(tool) {
            println!("  {} {}", self.i18n.t_with_vars("install.found", &[("tool", tool)]), "✓".green());
            return Ok(());
        }

        println!("  {} {}", self.i18n.t_with_vars("install.not_found", &[("tool", tool)]), "✗".red());

        if !self.interactive {
            return Ok(());
        }

        // Vérification spéciale pour open-webui
        if tool == "open-webui" {
            if let Some(pip_cmd) = self.find_pip_command() {
                println!("  ✓ pip trouvé : {}", pip_cmd.green());
            } else {
                println!("  ⚠️  pip n'est pas installé !");
                println!("  Installez d'abord Python et pip :");
                if cfg!(target_os = "linux") {
                    println!("    sudo apt install python3-pip  (Debian/Ubuntu)");
                    println!("    sudo dnf install python3-pip  (Fedora)");
                }
                return Ok(());
            }
        }

        println!("\n{}", self.i18n.t_with_vars("install.proposal", &[("tool", tool)]));

        let install_cmd = match tool {
            "ollama" => self.get_ollama_cmd(),
            "open-webui" => self.get_open_webui_cmd(),
            "hermes" => self.get_hermes_cmd(),
            "openclaw" => self.get_openclaw_cmd(),
            _ => {
                println!("Pas de commande pour {}", tool);
                return Ok(());
            }
        };

        println!("\n{}", self.i18n.t("install.command"));
        println!("  {}", install_cmd.cyan());
        
        if !self.shell.is_empty() {
            println!("  (Shell détecté : {})", self.shell.dimmed());
        }

        let confirm_text = self.i18n.t("install.confirm");
        let confirm = Confirm::new()
            .with_prompt(confirm_text)
            .default(false)
            .interact()?;

        if !confirm {
            println!("{}", "Installation annulée.".yellow());
            return Ok(());
        }

        println!("{}", self.i18n.t_with_vars("install.installing", &[("tool", tool)]));
        
        match run_command(&install_cmd) {
            Ok((stdout, stderr)) => {
                if !stdout.is_empty() { 
                    println!("{}", stdout); 
                }
                if !stderr.is_empty() && !stderr.contains("warning") {
                    eprintln!("{}", stderr.dimmed());
                }
                println!("{} {} ✓", 
                    self.i18n.t_with_vars("install.success", &[("tool", tool)]), 
                    "".green()
                );
                info!("{} installé avec succès", tool);
                Ok(())
            }
            Err(e) => {
                println!("{} {} ✗", 
                    self.i18n.t_with_vars("install.failed", &[("tool", tool)]), 
                    "".red()
                );
                println!("\n{}", e.to_string().yellow());
                
                // Instructions adaptées au shell
                println!("\n🔧 {}", "Pour installer manuellement :".bold());
                
                if self.shell.contains("fish") {
                    println!("  Avec Fish :");
                    println!("  {}", self.get_fish_alternative(tool).cyan());
                } else {
                    println!("  {}", install_cmd.cyan());
                }
                
                if tool == "open-webui" {
                    println!("\n💡 {}", "Alternative : installation via Docker".dimmed());
                    println!("  docker run -d -p 3000:8080 --name open-webui ghcr.io/open-webui/open-webui:main");
                }
                
                Ok(())
            }
        }
    }

    fn find_pip_command(&self) -> Option<String> {
        // Pour Fish, utiliser une syntaxe compatible
        let candidates = if self.shell.contains("fish") {
            vec![
                "pip3",
                "pip",
                "python3 -m pip",
                "python -m pip",
            ]
        } else {
            vec![
                "pip3",
                "pip",
                "python3 -m pip",
            ]
        };

        for cmd in &candidates {
            let check_cmd = if self.shell.contains("fish") {
                format!("command -v {} 2>/dev/null", cmd.split_whitespace().next().unwrap_or(cmd))
            } else {
                format!("command -v {} 2>/dev/null", cmd.split_whitespace().next().unwrap_or(cmd))
            };
            
            if let Ok((stdout, _)) = run_command(&check_cmd) {
                if !stdout.trim().is_empty() {
                    return Some(cmd.to_string());
                }
            }
        }
        
        None
    }

    fn get_ollama_cmd(&self) -> String {
        if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            "curl -fsSL https://ollama.com/install.sh | sh".to_string()
        } else if cfg!(target_os = "windows") {
            r#"powershell -c "iwr -useb https://ollama.com/install.ps1 | iex""#.to_string()
        } else {
            "echo 'OS non supporté'".to_string()
        }
    }

    fn get_open_webui_cmd(&self) -> String {
        if self.shell.contains("fish") {
            "pip3 install open-webui; or pip install open-webui; or python3 -m pip install open-webui".to_string()
        } else {
            "pip3 install open-webui 2>/dev/null || pip install open-webui 2>/dev/null || python3 -m pip install open-webui".to_string()
        }
    }

    fn get_hermes_cmd(&self) -> String {
        "echo 'Installation de Hermes Agent (à définir)'".to_string()
    }

    fn get_openclaw_cmd(&self) -> String {
        "npm install -g openclaw".to_string()
    }
    
    fn get_fish_alternative(&self, tool: &str) -> String {
        match tool {
            "open-webui" => {
                "python3 -m pip install --user open-webui".to_string()
            }
            "ollama" => {
                "curl -fsSL https://ollama.com/install.sh | sh".to_string()
            }
            _ => "Commande non disponible pour Fish".to_string()
        }
    }
}

// Fonction pour détecter le shell - CORRIGÉE
fn detect_shell() -> String {
    // Vérifier la variable SHELL
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("fish") {
            return "fish".to_string();
        }
        if shell.contains("zsh") {
            return "zsh".to_string();
        }
        if shell.contains("bash") {
            return "bash".to_string();
        }
        return shell;  // Retourner tel quel si pas reconnu
    }
    
    // Vérifier le processus parent (correction)
    let parent_pid = std::process::id().saturating_sub(1);  // Évite underflow
    if parent_pid > 0 {
        let proc_path = format!("/proc/{}/comm", parent_pid);
        if let Ok(comm) = std::fs::read_to_string(&proc_path) {
            let comm = comm.trim().to_lowercase();
            if comm.contains("fish") { return "fish".to_string(); }
            if comm.contains("zsh") { return "zsh".to_string(); }
            if comm.contains("bash") { return "bash".to_string(); }
        }
    }
    
    "unknown".to_string()
}