use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OllamaTool;

impl Tool for OllamaTool {
    fn id(&self) -> &str { "ollama" }
    fn name(&self) -> &str { "Ollama" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.ollama.description") }

    fn status(&self) -> ToolStatus {
        if shell::is_installed("ollama") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled }
    }

    fn install(&self) -> Result<()> {
        shell::run_live("curl -fsSL https://ollama.com/install.sh | sh")?;
        shell::run("sudo systemctl enable ollama")?;
    
        // Démarrer Ollama pour l'initialiser
        shell::run("sudo systemctl start ollama")?;
    
        // Attendre qu'il soit prêt
        for _ in 0..10 {
            if crate::core::ollama_api::detect_url().is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    
        // Générer la configuration par défaut
        let config = crate::config::env::EnvConfig::default_for_hardware(&crate::core::hardware::detect());
        config.save()?;
        
        // Installer dans les shells
        crate::config::shells::install_all_shells_cli()?;
        
        Ok(())
    }

    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::run(&i18n.t_with_vars("tool.ollama.run_model", &[("model", &m)]));
                let cmd: String = format!("ollama run {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::info(&i18n.t("ollama.choose_model"));
            }
        }
        Ok(())
    }
}

impl OllamaTool {
    // ─── Service ────────────────────────────────

    pub fn is_running() -> bool {
        crate::core::ollama_api::detect_url().is_some()
    }

    pub fn start() -> Result<()> {
        shell::run("sudo systemctl start ollama")?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn stop() -> Result<()> {
        shell::run("sudo systemctl stop ollama")?;
        Ok(())
    }

    pub fn enable_autostart() -> Result<()> {
        shell::run("sudo systemctl enable ollama")?;
        Ok(())
    }

    fn is_autostart_enabled() -> bool {
        shell::run("systemctl is-enabled ollama 2>/dev/null | grep -q enabled").is_ok()
    }

    pub fn ensure_running(i18n: &I18n) -> Result<()> {
        if !shell::is_installed("ollama") {
            display::warning(&i18n.t("ollama.not_installed"));
            if Confirm::new()
                .with_prompt(i18n.t("ollama.install_now"))
                .default(true)
                .interact()?
            {
                let tool = OllamaTool;
                tool.install()?;
                display::success(&i18n.t("ollama.installed"));

                // La config env est déjà générée par install()
                display::success(&i18n.t("config.generated_env"));
                return Ok(());
            }
            return Ok(());
        }

        // Vérifier la santé d'Ollama
        let fixes = crate::core::ollama_doctor::OllamaDoctor::check_and_fix()?;
        for fix in &fixes {
            display::success(fix);
        }

        if !Self::is_running() {
            display::warning(&i18n.t("ollama.not_running"));
            if Confirm::new()
                .with_prompt(i18n.t("ollama.start_now"))
                .default(true)
                .interact()?
            {
                Self::start()?;
                display::success(&i18n.t("ollama.started"));
                
                if !Self::is_autostart_enabled() {
                    if Confirm::new()
                        .with_prompt(i18n.t("ollama.enable_autostart"))
                        .default(true)
                        .interact()?
                    {
                        Self::enable_autostart()?;
                        display::success(&i18n.t("ollama.autostart_enabled"));
                    }
                }
            }
        }
        // Vérifier que la config wzllama existe
        let env_path = crate::config::env::EnvConfig::env_path();
        if !env_path.exists() {
            let config = crate::config::env::EnvConfig::default_for_hardware(&crate::core::hardware::detect());
            config.save()?;
            display::success(&i18n.t("config.generated_env"));
            
            // Installer dans les shells
            crate::config::shells::install_all_shells(i18n)?;
        }
        Ok(())
    }

    // ─── Désinstallation ────────────────────────

    pub fn uninstall(i18n: &I18n) -> Result<()> {
        display::warning(&i18n.t("ollama.uninstall_warning"));
        
        if !Confirm::new()
            .with_prompt(i18n.t("ollama.uninstall_confirm"))
            .default(false)
            .interact()?
        {
            return Ok(());
        }

        display::section(&i18n.t("ollama.uninstalling"));

        // 1. Arrêter et désactiver le service
        let _ = shell::run("sudo systemctl stop ollama 2>/dev/null");
        let _ = shell::run("sudo systemctl disable ollama 2>/dev/null");
        display::success(&i18n.t("ollama.service_stopped"));

        // 2. Supprimer le service systemd
        let _ = shell::run("sudo rm -f /etc/systemd/system/ollama.service 2>/dev/null");
        let _ = shell::run("sudo systemctl daemon-reload 2>/dev/null");
        display::success(&i18n.t("ollama.service_removed"));

        // 3. Supprimer le binaire
        if let Ok((stdout, _)) = shell::run("which ollama 2>/dev/null") {
            let bin = stdout.trim();
            if !bin.is_empty() {
                let _ = shell::run(&format!("sudo rm -f {}", bin));
                display::success(&i18n.t_with_vars("ollama.binary_removed", &[("path", bin)]));
            }
        }

        // 4. Supprimer les données
        let _ = shell::run("sudo rm -rf /usr/share/ollama 2>/dev/null");
        let _ = shell::run("sudo rm -rf /usr/lib/ollama 2>/dev/null");
        let _ = shell::run("sudo rm -rf ~/.ollama 2>/dev/null");
        display::success(&i18n.t("ollama.data_removed"));

        // 5. Supprimer l'utilisateur et le groupe
        let _ = shell::run("sudo userdel ollama 2>/dev/null");
        let _ = shell::run("sudo groupdel ollama 2>/dev/null");
        display::success(&i18n.t("ollama.user_removed"));

        display::success(&i18n.t("ollama.uninstalled"));
        Ok(())
    }
}