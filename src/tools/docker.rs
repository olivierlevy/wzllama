use crate::core::{shell, system};

pub fn is_installed() -> bool { shell::is_installed("docker") }
pub fn is_running() -> bool { shell::run("docker info 2>/dev/null").is_ok() }
pub fn start() -> anyhow::Result<()> { shell::run("sudo systemctl start docker")?; Ok(()) }

pub fn install_linux() -> anyhow::Result<()> {
    let install_docker = system::get_package_install_command("docker")?;
    let cmd = format!(
        "{} && sudo systemctl enable --now docker && sudo usermod -aG docker $USER",
        install_docker
    );
    shell::run_live(&cmd)?;
    Ok(())
}