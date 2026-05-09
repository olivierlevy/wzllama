use crate::core::shell;

pub fn is_installed() -> bool { shell::is_installed("docker") }
pub fn is_running() -> bool { shell::run("docker info 2>/dev/null").is_ok() }
pub fn start() -> anyhow::Result<()> { shell::run("sudo systemctl start docker")?; Ok(()) }

pub fn install_linux(pkg_manager: &str) -> anyhow::Result<()> {
    let cmd = match pkg_manager {
        "pacman" => "sudo pacman -S --noconfirm docker && sudo systemctl enable --now docker && sudo usermod -aG docker $USER",
        "apt" => "sudo apt install -y docker.io && sudo systemctl enable --now docker && sudo usermod -aG docker $USER",
        "dnf" => "sudo dnf install -y docker && sudo systemctl enable --now docker && sudo usermod -aG docker $USER",
        _ => return Err(anyhow::anyhow!("Gestionnaire de paquets inconnu")),
    };
    shell::run(cmd)?;
    Ok(())
}