use clap::{Parser, Subcommand};
use anyhow::Result;
// Imports des modules
use crate::wizard;
use crate::config;
use crate::core::ollama_api;

#[derive(Parser)]
#[command(name = "wzllama", about = "Assistant IA locale", version = "0.2.0")]
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
}

impl Cli {
    pub fn parse_args() -> Self { Cli::parse() }

    pub fn execute(&self) -> Result<()> {
        match self.command.as_ref().unwrap_or(&Command::Wizard) {
            Command::Wizard if self.dry_run => {
                println!("[DRY-RUN]");
                Ok(())
            }
            Command::Wizard => wizard::run(),
            Command::Validate => config::templates::validate_all(),
            Command::Bench => ollama_api::run_benchmark(),
            Command::ResetTemplates => config::templates::reset_all(),
            Command::CheckI18n => config::i18n::check_integrity(),
        }
    }
}