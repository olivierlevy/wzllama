use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(
    name = "wzllama",
    about = "Assistant pour votre IA locale",
    version = "0.1.0",
    author = "wzllama team"
)]
pub struct Cli {
    /// Mode dry-run : affiche sans exécuter
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Sous-commande optionnelle
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Lancer le wizard interactif (par défaut)
    #[command(visible_alias = "w")]
    Wizard,

    /// Valider les templates (usages.yaml + i18n)
    #[command(visible_alias = "v")]
    Validate,

    /// Mini-benchmark Ollama
    #[command(visible_alias = "b")]
    Bench,

    /// Réinitialiser les templates utilisateur
    #[command(visible_alias = "r")]
    ResetTemplates,

    /// Vérifier l'intégrité des fichiers i18n
    #[command(visible_alias = "i")]
    CheckI18n,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    pub fn execute(&self) -> Result<()> {
        let command = self.command.as_ref().unwrap_or(&Command::Wizard);

        match command {
            Command::Wizard => {
                if self.dry_run {
                    println!("[DRY-RUN] Lancement du wizard interactif...");
                    Ok(())
                } else {
                    crate::wizard::run_wizard()
                }
            }
            Command::Validate => crate::config::validate_all_templates(),
            Command::Bench => crate::core::run_benchmark(),
            Command::ResetTemplates => crate::config::reset_templates(),
            Command::CheckI18n => crate::config::check_i18n_integrity(),
        }
    }
}