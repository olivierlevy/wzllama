use anyhow::{Result, bail};
use dialoguer::Confirm;
use std::path::Path;
use std::thread;
use std::time::Duration;
use crate::config::I18n;
use crate::core::{shell, system};
use crate::display;

pub fn is_installed() -> bool { shell::is_installed("docker") }

/// Exécute une commande docker, sans sudo d'abord, puis avec sudo si nécessaire
pub fn run(cmd: &str) -> bool {
    shell::run(&format!("docker {} 2>/dev/null", cmd)).is_ok() ||
    shell::run(&format!("sudo docker {} 2>/dev/null", cmd)).is_ok()
}

/// Exécute une commande docker live (pour run_live), sans sudo d'abord
pub fn run_live(cmd: &str) -> Result<()> {
    if shell::run(&format!("docker {} 2>/dev/null", cmd)).is_ok() {
        Ok(())
    } else {
        shell::run_live(&format!("sudo docker {}", cmd))
    }
}

pub fn is_running() -> bool { 
    // Vérifier que Docker répond aux commandes de conteneurs (plus fiable que docker info)
    // Essayer sans sudo d'abord, puis avec sudo
    if shell::run("docker ps >/dev/null 2>&1").is_ok() {
        return true;
    }
    // Attendre un peu et réessayer (docker peut mettre du temps à être prêt)
    std::thread::sleep(std::time::Duration::from_millis(500));
    if shell::run("docker ps >/dev/null 2>&1").is_ok() {
        return true;
    }
    if shell::run("sudo docker ps >/dev/null 2>&1").is_ok() {
        return true;
    }
    // Fallback: vérifier si le socket Docker existe
    std::path::Path::new("/var/run/docker.sock").exists()
}
pub fn start() -> Result<()> { shell::run("systemctl start docker 2>/dev/null || sudo systemctl start docker").map(|_| ()) }
pub fn startup() -> Result<()> { shell::run("systemctl enable docker 2>/dev/null || sudo systemctl enable docker").map(|_| ()) }
pub fn restart_socket() -> Result<()> { shell::run("systemctl restart docker.socket 2>/dev/null || sudo systemctl restart docker.socket").map(|_| ()) }

pub fn install_linux() -> Result<()> {
    let install_docker = system::get_package_install_command("docker")?;
    // Structure claire: installer docker, puis démarrer le service, puis ajouter l'utilisateur au groupe
    let cmd = format!(
        "{} && (systemctl enable --now docker 2>/dev/null || sudo systemctl enable --now docker 2>/dev/null || true) && (groupadd docker 2>/dev/null || sudo groupadd docker 2>/dev/null || true) && (usermod -aG docker $USER 2>/dev/null || sudo usermod -aG docker $USER)",
        install_docker
    );
    shell::run_live(&cmd)?;
    display::info("Docker installé. Déconnectez-vous et reconnectez-vous pour utiliser docker sans sudo.");
    Ok(())
}

/// Vérifie que Docker est installé, que le socket est présent et que le service tourne.
/// Version non-interactive pour les appels depuis le TUI (pas de Confirm).
pub fn ensure_ready_no_confirm() -> Result<()> {
    if !is_installed() {
        bail!("Docker non installé");
    }

    if !Path::new("/var/run/docker.sock").exists() {
        restart_socket()?;
        thread::sleep(Duration::from_secs(2));
    }

    // Vérifier que Docker répond aux commandes
    if !is_running() {
        bail!("Docker arrêté - démarrez-le avec: sudo systemctl start docker");
    }
    Ok(())
}

/// Vérifie que Docker est installé, que le socket est présent et que le service tourne.
/// En cas de problème, propose une correction. Retourne Ok(()) si tout est prêt,
/// ou une erreur si l'utilisateur refuse ou si une action échoue.
pub fn ensure_ready(i18n: &I18n) -> Result<()> {
    // Essayer d'abord sans confirmation
    if ensure_ready_no_confirm().is_ok() {
        return Ok(());
    }
    
    // Si ça échoue, demander confirmation
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
            // Attendre que Docker soit vraiment prêt - jusqu'à 30 secondes
            // Vérifier avec ou sans sudo (essayer les deux)
            let mut ready = false;
            for i in 1..=60 {
                std::thread::sleep(std::time::Duration::from_millis(500));
                // Essayer docker ps -a sans sudo d'abord
                if shell::run("docker ps -a >/dev/null 2>&1").is_ok() {
                    ready = true;
                    break;
                }
            }
            if !ready {
                // Essayer avec sudo
                for i in 1..=30 {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if shell::run("sudo docker ps -a >/dev/null 2>&1").is_ok() {
                        ready = true;
                        break;
                    }
                }
            }
            if !ready {
                bail!("Docker n'est pas prêt après 30 secondes. Réessayez plus tard.");
            }
        } else {
            bail!("Docker arrêté");
        }
    }
    Ok(())
}
