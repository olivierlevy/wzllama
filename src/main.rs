mod cli;
mod config;
mod core;
mod display;
mod error;
mod tools;
mod wizard;
mod menu_api;
mod api_server;

use anyhow::Result;
use cli::Cli;
use cli::Command;
use log::info;
use std::sync::OnceLock;

static API_STARTED: OnceLock<bool> = OnceLock::new();

/// Start the API server in background (once per session)
fn start_api_server_background() {
    API_STARTED.get_or_init(|| {
        // Check if already running on port 1133
        if std::process::Command::new("sh")
            .arg("-c")
            .arg("curl -s http://localhost:1133/health > /dev/null 2>&1")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            println!("📡 API server already running on port 1133");
            return true;
        }

        // Start in background thread
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 1133));
                crate::api_server::start_server(addr).await;
            });
        });
        println!("🚀 API server starting on http://localhost:1133");
        true
    });
}

fn main() -> Result<()> {
    config::paths::ensure_dirs()?;
    config::logging::init()?;
    config::logging::install_embedded_i18n()?;
    info!("wzllama v0.2.0 démarré");

    let cli = Cli::parse_args();
    
    // Start API server in background for wizard mode only (not for serve command)
    if matches!(cli.command, None | Some(Command::Wizard)) {
        start_api_server_background();
    }
    
    let result = cli.execute();
    
    // Request API server shutdown when exiting (only if we started it)
    if matches!(cli.command, None | Some(Command::Wizard)) {
        crate::api_server::request_shutdown();
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    
    result
}