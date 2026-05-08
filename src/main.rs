mod cli;
mod config;
mod core;
mod error;
mod installers;
mod wizard;

use anyhow::Result;
use cli::Cli;
use log::info;

fn main() -> Result<()> {
    // Initialiser les logs
    config::init_logging()?;
    info!("wzllama démarré");

    // S'assurer que les templates utilisateur existent
    config::ensure_user_templates()?;

    // Parser les arguments CLI
    let cli = Cli::parse_args();

    // Exécuter la commande appropriée
    cli.execute()?;

    Ok(())
}