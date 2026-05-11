use anyhow::{Result, bail};
use colored::*;
use dialoguer::Confirm;
use std::path::Path;
use std::thread;
use std::time::Duration;
use crate::config::I18n;
use crate::core::{shell, system};
use crate::display;

pub fn is_installed() -> bool { shell::is_installed("docker") }
pub fn is_running() -> bool { shell::run("docker info 2>/dev/null").is_ok() }
pub fn start() -> Result<()> { shell::run("sudo systemctl start docker")?; Ok(()) }
pub fn startup() -> Result<()> { shell::run("sudo systemctl enable docker")?; Ok(()) }
pub fn restart_socket() -> Result<()> { shell::run("sudo systemctl restart docker.socket")?; Ok(()) }

pub fn install_linux() -> Result<()> {
    let install_docker = system::get_package_install_command("docker")?;
    let cmd = format!(
        "{} && sudo systemctl enable --now docker && sudo usermod -aG docker $USER",
        install_docker
    );
    shell::run_live(&cmd)?;
    let _ = startup();
    Ok(())
}

/// Vérifie que Docker est installé, que le socket est présent et que le service tourne.
/// En cas de problème, propose une correction. Retourne Ok(()) si tout est prêt,
/// ou une erreur si l'utilisateur refuse ou si une action échoue.
pub fn ensure_ready(i18n: &I18n) -> Result<()> {
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
            bail!("Docker refusé");
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
        } else {
            bail!("Docker arrêté");
        }
    }
    Ok(())
}
