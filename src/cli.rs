use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::config;
use crate::core::{ollama_api, shell};
use crate::wizard;

#[derive(Parser)]
#[command(name = "wzllama", about = "Assistant IA locale", version = "0.3.0")]
pub struct Cli {
    #[arg(long, global = true)]
    pub dry_run: bool,

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
    #[command(visible_alias = "s")]
    Serve,
    /// Install Open WebUI with Docker checks
    InstallWebui,
    /// Launch Open WebUI with Docker checks
    LaunchWebui,
    /// Catalog management: refresh or list Ollama integrations
    Catalog {
        #[command(subcommand)]
        subcommand: CatalogCommand,
    },
    /// Update all installed tools
    UpdateAll,
}

#[derive(Subcommand)]
pub enum CatalogCommand {
    /// Force-refresh the tool catalog from docs.ollama.com
    Refresh,
    /// List all tools in the catalog, grouped by category
    List,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    pub fn execute(&self) -> Result<()> {
        match self.command.as_ref().unwrap_or(&Command::Wizard) {
            Command::Wizard if self.dry_run => {
                println!("[DRY-RUN]");
                Ok(())
            }
            Command::Wizard => {
                let mut state = crate::config::WzllamaState::load();
                let i18n = wizard::select_language(&mut state)?;
                let hardware = crate::core::hardware::detect();
                crate::menu_api::MainMenuRunner::new(&i18n, &mut state, &hardware).run()
            }
            Command::Validate => config::templates::validate_all(),
            Command::Bench => ollama_api::run_benchmark(),
            Command::ResetTemplates => config::templates::reset_all(),
            Command::CheckI18n => config::i18n::check_integrity(),
            Command::Uninstall => wizard::menu_config::uninstall_wzllama_cli(),

            Command::Catalog { subcommand } => match subcommand {
                CatalogCommand::Refresh => {
                    crate::core::catalog_refresh::CatalogRefresher::force_refresh()?;
                    Ok(())
                }
                CatalogCommand::List => {
                    use crate::tools::catalog::{ToolCatalog, ToolCategory};
                    let catalog = ToolCatalog::load();
                    let categories = [
                        ToolCategory::CodingAgent,
                        ToolCategory::Assistant,
                        ToolCategory::Ide,
                        ToolCategory::ChatRag,
                        ToolCategory::Automation,
                        ToolCategory::Notebook,
                        ToolCategory::Unknown,
                    ];
                    println!("📦 Ollama Integrations Catalog ({})", catalog.version);
                    println!();
                    for cat in &categories {
                        let tools: Vec<_> =
                            catalog.tools.iter().filter(|t| &t.category == cat).collect();
                        if tools.is_empty() {
                            continue;
                        }
                        println!("  ── {}:", cat.display_name());
                        for t in tools {
                            let install = t.install_cmd.as_deref().unwrap_or("ollama launch");
                            println!("    • {} ({})\t[{}]", t.name, t.id, install);
                        }
                        println!();
                    }
                    Ok(())
                }
            },

            Command::UpdateAll => {
                let state = crate::config::WzllamaState::load();
                let i18n = crate::config::I18n::default();
                let summary =
                    crate::core::tool_updater::ToolUpdater::update_all_verbose(&state, &i18n)?;
                println!();
                println!(
                    "📊 Update summary: {} updated, {} failed, {} skipped",
                    summary.updated.len(),
                    summary.failed.len(),
                    summary.skipped.len()
                );
                for (name, err) in &summary.failed {
                    println!("  ❌ {}: {}", name, err);
                }
                Ok(())
            }

            Command::InstallWebui => {
                if let Err(e) = crate::tools::docker::ensure_ready_no_confirm() {
                    println!("⚠️  Docker non prêt: {}", e);
                    println!("💡 Pour installer Docker: curl -fsSL https://get.docker.com | sh");
                    println!("💡 Pour ajouter votre utilisateur au groupe docker: sudo usermod -aG docker $USER");
                    println!("💡 Puis déconnectez-vous et log back in");
                    return Ok(());
                }
                let exists = shell::run("docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$' || sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$'").is_ok();
                if exists {
                    shell::run(
                        "docker start open-webui 2>/dev/null || sudo docker start open-webui",
                    )?;
                    println!("✅ Open WebUI started");
                } else {
                    shell::run("docker run -d --network=host --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data -e OLLAMA_BASE_URL=http://127.0.0.1:11434 --name open-webui --restart always ghcr.io/open-webui/open-webui:ollama 2>/dev/null || sudo docker run -d --network=host --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data -e OLLAMA_BASE_URL=http://127.0.0.1:11434 --name open-webui --restart always ghcr.io/open-webui/open-webui:ollama")?;
                    println!("✅ Open WebUI installed");
                }
                Ok(())
            }
            Command::LaunchWebui => {
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
            Command::Serve => {
                let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 1133));
                tokio::runtime::Runtime::new()?.block_on(crate::api_server::start_server(addr));
                Ok(())
            }
        }
    }
}
