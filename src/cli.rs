use clap::{Parser, Subcommand};
use anyhow::Result;
// Imports des modules
use crate::wizard;
use crate::config;
use crate::core::{ollama_api, shell};

#[derive(Parser)]
#[command(name = "wzllama", about = "Assistant IA locale", version = "0.2.0")]
pub struct Cli {
    #[arg(long, global = true)]
    pub dry_run: bool,

    // TUI mode is disabled - CLI wizard remains the primary interface
    // #[arg(long, global = true)]
    // pub tui: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(visible_alias = "w")]
    Wizard,
    #[command(visible_alias = "v")]
    Validate,
    #[command(visible_alias = "b")]
    Bench,
    #[command(visible_alias = "r")]
    ResetTemplates,
    #[command(visible_alias = "i")]
    CheckI18n,
    #[command(visible_alias = "u")]
    Uninstall,
    /// Install Open WebUI with Docker checks
    InstallWebui,
    /// Launch Open WebUI with Docker checks
    LaunchWebui,
}

impl Cli {
    pub fn parse_args() -> Self { Cli::parse() }

    pub fn execute(&self) -> Result<()> {
        // TUI mode disabled - CLI wizard remains the primary interface
        // The --tui flag is commented out in the struct definition above
        
        match self.command.as_ref().unwrap_or(&Command::Wizard) {
            Command::Wizard if self.dry_run => {
                println!("[DRY-RUN]");
                Ok(())
            }
            Command::Wizard => {
                let mut state = crate::config::WzllamaState::load();
                let i18n = wizard::select_language(&mut state)?;
                let hardware = crate::core::hardware::detect();
                wizard::run(&i18n, &mut state, &hardware)
            }
            Command::Validate => config::templates::validate_all(),
            Command::Bench => ollama_api::run_benchmark(),
            Command::ResetTemplates => config::templates::reset_all(),
            Command::CheckI18n => config::i18n::check_integrity(),
            Command::Uninstall => wizard::menu_config::uninstall_wzllama_cli(),
            Command::InstallWebui => {
                // Check Docker puis installer Open WebUI
                if let Err(e) = crate::tools::docker::ensure_ready_no_confirm() {
                    println!("⚠️  Docker non prêt: {}", e);
                    println!("💡 Pour installer Docker: curl -fsSL https://get.docker.com | sh");
                    println!("💡 Pour ajouter votre utilisateur au groupe docker: sudo usermod -aG docker $USER");
                    println!("💡 Puis déconnectez-vous et log back in");
                    return Ok(());
                }
                // Check si déjà installé
                let exists = shell::run("docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$' || sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$'").is_ok();
                if exists {
                    shell::run("docker start open-webui 2>/dev/null || sudo docker start open-webui")?;
                    println!("✅ Open WebUI démarré");
                } else {
                    shell::run("docker run -d --network=host --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data -e OLLAMA_BASE_URL=http://127.0.0.1:11434 --name open-webui --restart always ghcr.io/open-webui/open-webui:ollama 2>/dev/null || sudo docker run -d --network=host --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data -e OLLAMA_BASE_URL=http://127.0.0.1:11434 --name open-webui --restart always ghcr.io/open-webui/open-webui:ollama")?;
                    println!("✅ Open WebUI installé");
                }
                Ok(())
            }
            Command::LaunchWebui => {
                // Check Docker puis lancer Open WebUI
                if let Err(e) = crate::tools::docker::ensure_ready_no_confirm() {
                    println!("⚠️  Docker non prêt: {}", e);
                    println!("💡 Pour démarrer Docker: sudo systemctl start docker");
                    println!("💡 Si erreur de permission: sudo usermod -aG docker $USER");
                    return Ok(());
                }
                let url = "http://localhost:8080";
                println!("🌐 Open WebUI : {}", url);
                shell::open_url(url);
                println!("✅ Open WebUI lancé dans le navigateur");
                Ok(())
            }
        }
    }
}