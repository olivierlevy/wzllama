mod cli;
mod config;
mod core;
mod display;
mod error;
mod tools;
mod wizard;
mod tui;

use anyhow::Result;
use cli::Cli;
use log::info;

fn main() -> Result<()> {
    config::paths::ensure_dirs()?;
    config::logging::init()?;
    config::logging::install_embedded_i18n()?;
    info!("wzllama v0.2.0 démarré");

    let cli = Cli::parse_args();
    cli.execute()?;

    Ok(())
}