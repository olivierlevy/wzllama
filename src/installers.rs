use crate::config::I18n;
use crate::core::run_command;
use anyhow::Result;
use colored::*;
use dialoguer::Confirm;
use log::info;

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub detect_cmd: String,
    pub install_cmd: Option<String>,
}

pub struct Installer<'a> {
    i18n: &'a I18n,
    interactive: bool,
}

impl<'a> Installer<'a> {
    pub fn new(i18n: &'a I18n, interactive: bool) -> Self {
        Self { i18n, interactive }
    }

    pub fn install_all_tools(&self) -> Result<()> {
        println!("\n{}", "═".repeat(50).cyan());
        println!("{}", self.t("install.title").bold());
        println!("{}", "═".repeat(50).cyan());

        self.check_and_install_docker()?;

        let tools = self.get_all_tools();
        for tool in &tools {
            self.check_and_install(tool)?;
        }

        Ok(())
    }

    fn t(&self, key: &str) -> String {
        self.i18n.t(key)
    }

    fn tv(&self, key: &str, vars: &[(&str, &str)]) -> String {
        self.i18n.t_with_vars(key, vars)
    }

    fn check_and_install_docker(&self) -> Result<()> {
        println!("\n🔍 {}", "Docker".bold());
        
        if self.is_installed("docker") {
            println!("   {}", self.t("install.docker.installed").green());
            
            if run_command("docker info 2>/dev/null").is_ok() {
                println!("   {}", self.t("install.docker.running").green());
            } else {
                println!("   {}", self.t("install.docker.stopped").yellow());
                
                // Proposer de démarrer le daemon
                if self.interactive {
                    let confirm = Confirm::new()
                        .with_prompt("   Démarrer le daemon Docker ?")
                        .default(true)
                        .interact()?;
                    
                    if confirm {
                        if run_command("sudo systemctl start docker").is_ok() {
                            println!("   ✅ Daemon Docker démarré");
                        } else {
                            println!("   {}", self.t("install.docker.start_hint").dimmed());
                        }
                    }
                } else {
                    println!("   {}", self.t("install.docker.start_hint").dimmed());
                }
            }
            return Ok(());
        }

        println!("   {}", self.t("install.docker.not_installed").red());

        if !self.interactive {
            return Ok(());
        }

        let confirm = Confirm::new()
            .with_prompt(self.t("install.docker.confirm"))
            .default(true)
            .interact()?;

        if confirm {
            self.install_docker()?;
        } else {
            println!("   {}", self.tv("install.docker.required", &[("tool", "Open WebUI")]).yellow());
        }

        Ok(())
    }

    fn install_docker(&self) -> Result<()> {
        let pkg_manager = detect_package_manager();
        let relog = self.t("install.docker.relog");
        
        let cmd = match pkg_manager.as_str() {
            "pacman" => format!("sudo pacman -S --noconfirm docker && sudo systemctl enable --now docker && sudo usermod -aG docker $USER && echo '{}'", relog),
            "apt" => format!("sudo apt install -y docker.io && sudo systemctl enable --now docker && sudo usermod -aG docker $USER && echo '{}'", relog),
            "dnf" => format!("sudo dnf install -y docker && sudo systemctl enable --now docker && sudo usermod -aG docker $USER && echo '{}'", relog),
            _ => "echo 'https://docs.docker.com/engine/install/'".to_string(),
        };

        println!("\n{}", self.tv("install.installing", &[("tool", "Docker")]));
        println!("   {}", cmd.cyan());

        match run_command(&cmd) {
            Ok(_) => {
                println!("   {}", self.tv("install.success", &[("tool", "Docker")]).green());
                // ICI :
                println!("   {}", self.t("install.docker.relog").yellow());
                println!("   {}", "newgrp docker".dimmed());
            },
            Err(e) => {
                println!("   {}", self.tv("install.failed", &[("tool", "Docker")]).red());
                println!("\n   {}", self.t("install.manual_command").dimmed());
                println!("   https://docs.docker.com/engine/install/");
            }
        }

        Ok(())
    }

    fn get_all_tools(&self) -> Vec<Tool> {
        // Vérifier si docker fonctionne sans sudo
        let docker_cmd = if run_command("docker info 2>/dev/null").is_ok() {
            "docker"
        } else if run_command("sudo docker info 2>/dev/null").is_ok() {
            "sudo docker"
        } else {
            "docker"  // on tente quand même
        };

        let open_webui_cmd = format!(
            "{} run -d -p 3000:8080 \
            --add-host=host.docker.internal:host-gateway \
            -v open-webui:/app/backend/data \
            --name open-webui \
            --restart always \
            ghcr.io/open-webui/open-webui:main",
            docker_cmd
        );

        vec![
            Tool {
                name: "Ollama".to_string(),
                detect_cmd: "ollama".to_string(),
                install_cmd: Some("curl -fsSL https://ollama.com/install.sh | sh".to_string()),
            },
            Tool {
                name: "Open WebUI".to_string(),
                detect_cmd: "docker".to_string(),
                install_cmd: Some(open_webui_cmd),
            },
            Tool {
                name: "OpenClaw".to_string(),
                detect_cmd: "openclaw".to_string(),
                install_cmd: Some("npm install -g openclaw".to_string()),
            },
        ]
    }

    fn is_installed(&self, cmd: &str) -> bool {
        let check = format!("command -v {} 2>/dev/null", cmd);
        run_command(&check).is_ok()
    }

    fn check_and_install(&self, tool: &Tool) -> Result<()> {
        println!("\n🔍 {}", tool.name.bold());

        if tool.name == "Open WebUI" {
            if run_command("docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok() {
                let is_running = run_command("docker ps --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok();
                if is_running {
                    println!("   {}", self.tv("install.already_running", &[("tool", &tool.name)]).green());
                } else {
                    println!("   {}", self.tv("install.container_stopped", &[("tool", &tool.name)]).yellow());
                    println!("   {}", self.t("install.docker_start_hint").dimmed());
                }
                return Ok(());
            }
        } else if self.is_installed(&tool.detect_cmd) {
            println!("   {}", self.tv("install.already_installed", &[("tool", &tool.name)]).green());
            return Ok(());
        }

        println!("   {}", self.tv("install.not_installed", &[("tool", &tool.name)]).red());

        if !self.interactive {
            return Ok(());
        }

        let confirm = Confirm::new()
            .with_prompt(format!("   {}", self.tv("install.confirm_tool", &[("tool", &tool.name)])))
            .default(true)
            .interact()?;

        if !confirm {
            println!("   {}", self.tv("install.skipped", &[("tool", &tool.name)]).dimmed());
            return Ok(());
        }

        if let Some(cmd) = &tool.install_cmd {
            println!("\n{}", self.tv("install.installing", &[("tool", &tool.name)]).bold().cyan());
            println!("   {}", cmd.cyan());

            match run_command(cmd) {
                Ok((stdout, stderr)) => {
                    if !stdout.is_empty() {
                        println!("{}", stdout);
                    }
                    if !stderr.is_empty() {
                        println!("{}", stderr.dimmed());
                    }
                    println!("   {}", self.tv("install.success", &[("tool", &tool.name)]).green());
                    
                    if tool.name == "Open WebUI" {
                        println!("\n   {}", self.tv("install.open_webui_url", &[("tool", &tool.name)]).bold());
                    }
                    if tool.name == "Ollama" {
                        println!("\n   {}", self.t("install.ollama_hint").dimmed());
                    }
                }
                Err(e) => {
                    println!("   {}", self.tv("install.failed", &[("tool", &tool.name)]).red());
                    println!("\n   {}", self.t("install.manual_command").dimmed());
                    println!("   {}", cmd.cyan());
                }
            }
        }

        Ok(())
    }
}

fn detect_package_manager() -> String {
    for (cmd, name) in &[
        ("pacman", "pacman"),
        ("apt", "apt"),
        ("dnf", "dnf"),
        ("zypper", "zypper"),
        ("brew", "brew"),
    ] {
        if run_command(&format!("command -v {}", cmd)).is_ok() {
            return name.to_string();
        }
    }
    "unknown".to_string()
}