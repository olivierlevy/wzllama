use anyhow::{Result, bail};
use dialoguer::Confirm;
use std::path::Path;
use std::thread;
use std::time::Duration;
use crate::config::I18n;
use crate::core::{shell, system};
use crate::display;

pub fn is_installed() -> bool { shell::is_installed("docker") }

/// Executes a docker command, without sudo first, then with sudo if needed
pub fn run(cmd: &str) -> bool {
    shell::run(&format!("docker {} 2>/dev/null", cmd)).is_ok() ||
    shell::run(&format!("sudo docker {} 2>/dev/null", cmd)).is_ok()
}

/// Executes a docker command live (for run_live), without sudo first
pub fn run_live(cmd: &str) -> Result<()> {
    if shell::run(&format!("docker {} 2>/dev/null", cmd)).is_ok() {
        Ok(())
    } else {
        shell::run_live(&format!("sudo docker {}", cmd))
    }
}

pub fn is_running() -> bool { 
    // Check that Docker responds to container commands (more reliable than docker info)
    // Try without sudo first, then with sudo
    if shell::run("docker ps >/dev/null 2>&1").is_ok() {
        return true;
    }
    // Wait a bit and retry (docker may take time to be ready)
    std::thread::sleep(std::time::Duration::from_millis(500));
    if shell::run("docker ps >/dev/null 2>&1").is_ok() {
        return true;
    }
    if shell::run("sudo docker ps >/dev/null 2>&1").is_ok() {
        return true;
    }
    // Fallback: vérifier si the socket Docker existe
    std::path::Path::new("/var/run/docker.sock").exists()
}
pub fn start() -> Result<()> { shell::run("systemctl start docker 2>/dev/null || sudo systemctl start docker").map(|_| ()) }
pub fn restart_socket() -> Result<()> { shell::run("systemctl restart docker.socket 2>/dev/null || sudo systemctl restart docker.socket").map(|_| ()) }

pub fn install_linux() -> Result<()> {
    let install_docker = system::get_package_install_command("docker")?;
    // Clear structure: install docker, then start the service, then add user to group
    let cmd = format!(
        "{} && (systemctl enable --now docker 2>/dev/null || sudo systemctl enable --now docker 2>/dev/null || true) && (groupadd docker 2>/dev/null || sudo groupadd docker 2>/dev/null || true) && (usermod -aG docker $USER 2>/dev/null || sudo usermod -aG docker $USER)",
        install_docker
    );
    shell::run_live(&cmd)?;
    display::info("Docker installed. Log out and log back in to use docker without sudo.");
    Ok(())
}

/// Checks that Docker is installed, the socket is present and the service is running.
/// Non-interactive version for TUI calls (no Confirm).
pub fn ensure_ready_no_confirm() -> Result<()> {
    if !is_installed() {
        bail!("Docker not installed");
    }

    if !Path::new("/var/run/docker.sock").exists() {
        restart_socket()?;
        thread::sleep(Duration::from_secs(2));
    }

    // Check that Docker responds to commands
    if !is_running() {
        bail!("Docker stopped - start it with: sudo systemctl start docker");
    }
    Ok(())
}

/// Checks that Docker is installed, the socket is present and the service is running.
/// If there's a problem, proposes a fix. Returns Ok(()) if ready,
/// or an error if the user refuses or an action fails.
pub fn ensure_ready(i18n: &I18n) -> Result<()> {
    // Try without confirmation first
    if ensure_ready_no_confirm().is_ok() {
        return Ok(());
    }
    
    // If it fails, ask for confirmation
    if !is_installed() {
        display::warning(&i18n.t("install.docker.not_installed"));
        if Confirm::new()
            .with_prompt(i18n.t("install.docker.install_now"))
            .default(true)
            .interact()?
        {
            install_linux()?;
            display::success(&i18n.t("install.docker.installed"));
        } else {
            bail!("Docker refused");
        }
    }

    if !Path::new("/var/run/docker.sock").exists() {
        display::warning(&i18n.t("install.docker.socket_missing"));
        restart_socket()?;
        thread::sleep(Duration::from_secs(2));
    }

    if !is_running() {
        display::warning(&i18n.t("install.docker.stopped"));
        if Confirm::new()
            .with_prompt(i18n.t("install.docker.start_now"))
            .default(true)
            .interact()?
        {
            start()?;
            // Wait for Docker to really be ready - up to 30 seconds
            // Check with or without sudo (try both)
            let mut ready = false;
            for _i in 1..=60 {
                std::thread::sleep(std::time::Duration::from_millis(500));
                // Try docker ps -a without sudo first
                if shell::run("docker ps -a >/dev/null 2>&1").is_ok() {
                    ready = true;
                    break;
                }
            }
            if !ready {
                // Try with sudo
                for _i in 1..=30 {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if shell::run("sudo docker ps -a >/dev/null 2>&1").is_ok() {
                        ready = true;
                        break;
                    }
                }
            }
            if !ready {
                bail!("Docker is not ready after 30 seconds. Try again later.");
            }
        } else {
            bail!("Docker stopped");
        }
    }
    Ok(())
}
