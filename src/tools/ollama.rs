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

    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        if shell::is_installed("ollama") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled }
    }

    fn install(&self, i18n: &I18n) -> Result<()> {
        OllamaTool::install(i18n)
    }

    fn update(&self, i18n: &I18n) -> Result<()> {
        OllamaTool::update(i18n)
    }

    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        OllamaTool::uninstall(i18n)
    }

    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        OllamaTool::launch(i18n, state, model)
    }
}

impl OllamaTool {
    // ─── Service ────────────────────────────────

    pub fn is_running() -> bool {
        crate::core::ollama_api::detect_url().is_some()
    }

    pub fn start() -> Result<()> {
        shell::run_live("sudo systemctl start ollama")?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn stop() -> Result<()> {
        shell::run_live("sudo systemctl stop ollama")?;
        Ok(())
    }

    pub fn enable_autostart() -> Result<()> {
        shell::run_live("sudo systemctl enable ollama")?;
        Ok(())
    }

    fn is_autostart_enabled() -> bool {
        shell::run_quiet("systemctl is-enabled ollama 2>/dev/null | grep -q enabled").is_ok()
    }

    /// Setup ollama user and directory for models (called when ollama is not installed)
    pub fn setup_ollama_user_dir() -> Result<()> {
        // Create ollama user if not exists
        if shell::run("id -u ollama >/dev/null 2>&1").is_err() {
            shell::run("sudo useradd -r -s /bin/false ollama")?;
        }
        
        // Create home directory for ollama
        shell::run("sudo mkdir -p /home/ollama")?;
        shell::run("sudo chown ollama:ollama /home/ollama")?;
        shell::run("sudo chmod 755 /home/ollama")?;
        
        // Create symlink for /usr/share/ollama if it exists
        if std::path::Path::new("/usr/share/ollama").exists() {
            shell::run("sudo rm -rf /usr/share/ollama")?;
        }
        shell::run("sudo ln -sf /home/ollama /usr/share/ollama")?;
        
        Ok(())
    }

    pub fn ensure_running(i18n: &I18n) -> Result<()> {
        if !shell::is_installed("ollama") {
            display::warning(&i18n.t("ollama.not_installed"));
            display::warning(&i18n.t("ollama.required_mandatory"));
            if Confirm::new()
                .with_prompt(i18n.t("ollama.install_now"))
                .default(true)
                .interact()?
            {
                let tool = OllamaTool;
                tool.install(i18n)?;
                display::success(&i18n.t("ollama.installed"));

                // La config env est déjà générée par install()
                display::success(&i18n.t("config.generated_env"));
            } else {
                anyhow::bail!("{}", i18n.t("ollama.required_mandatory_exit"));
            }
        }

        // Check la santé d'Ollama
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
                
                if !Self::is_autostart_enabled()
                    && Confirm::new()
                        .with_prompt(i18n.t("ollama.enable_autostart"))
                        .default(true)
                        .interact()?
                {
                    Self::enable_autostart()?;
                    display::success(&i18n.t("ollama.autostart_enabled"));
                }
            }
        }
        // Check que la config wzllama existe
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

    // ─── Installation, Update, Launch ─────────────────

    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        // Setup ollama user and directory BEFORE installation
        Self::setup_ollama_user_dir()?;
        
        // Install Ollama
        shell::run_live("curl -fsSL https://ollama.com/install.sh | sh")?;
        
        // Install systemd drop-in for environment variables
        // Use bash explicitly for heredoc compatibility across all shells (fish, zsh, bash)
        shell::run("sudo mkdir -p /etc/systemd/system/ollama.service.d")?;
        
        // Build environment variables from config
        let config = crate::config::env::EnvConfig::load();
        let mut env_lines = vec!["[Service]".to_string()];
        env_lines.push("Environment=\"OLLAMA_MODELS=/home/ollama\"".to_string());
        env_lines.push(format!("Environment=\"OLLAMA_ORIGINS={}\"", config.ollama.origins));
        env_lines.push(format!("Environment=\"OLLAMA_KEEP_ALIVE={}\"", config.ollama.keep_alive));
        env_lines.push(format!("Environment=\"OLLAMA_NUM_PARALLEL={}\"", config.ollama.num_parallel));
        env_lines.push(format!("Environment=\"OLLAMA_MAX_LOADED_MODELS={}\"", config.ollama.max_loaded_models));
        env_lines.push(format!("Environment=\"OLLAMA_FLASH_ATTENTION={}\"", if config.ollama.flash_attention { 1 } else { 0 }));
        env_lines.push(format!("Environment=\"OLLAMA_KV_CACHE_TYPE={}\"", config.ollama.kv_cache_type));
        env_lines.push(format!("Environment=\"OLLAMA_CONTEXT_LENGTH={}\"", config.ollama.context_length));
        if config.ollama.max_vram > 0 {
            env_lines.push(format!("Environment=\"OLLAMA_MAX_VRAM={}\"", config.ollama.max_vram));
        }
        if !config.ollama.cuda_visible_devices.is_empty() {
            env_lines.push(format!("Environment=\"CUDA_VISIBLE_DEVICES={}\"", config.ollama.cuda_visible_devices));
        }
        
        let override_content = env_lines.join("\\n");
        shell::run_quiet(&format!("printf '{}' | sudo tee /etc/systemd/system/ollama.service.d/override.conf > /dev/null", override_content))?;
        shell::run_quiet("sudo systemctl daemon-reload")?;
        
        shell::run_live("sudo systemctl enable ollama")?;
        shell::run_live("sudo systemctl start ollama")?;
    
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

    pub fn update(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("Checking for Ollama updates...");
        shell::run_live("ollama pull --latest 2>/dev/null || true")?;
        display::success("✅ Ollama is up to date");
        Ok(())
    }

    pub fn launch(i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::run(&i18n.t_with_vars("tool.ollama.run_model", &[("model", m)]));
                let cmd: String = format!("ollama run {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::info(&i18n.t("ollama.choose_model"));
            }
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
        let _ = shell::run_quiet("sudo systemctl stop ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo systemctl disable ollama 2>/dev/null");
        display::success(&i18n.t("ollama.service_stopped"));

        // 2. Supprimer le service systemd
        let _ = shell::run_quiet("sudo rm -f /etc/systemd/system/ollama.service 2>/dev/null");
        let _ = shell::run_quiet("sudo systemctl daemon-reload 2>/dev/null");
        display::success(&i18n.t("ollama.service_removed"));

        // 3. Supprimer le binaire et fichiers d'installation
        if let Ok((stdout, _)) = shell::run_quiet("which ollama 2>/dev/null") {
            let bin = stdout.trim();
            if !bin.is_empty() {
                let _ = shell::run(&format!("sudo rm -f {}", bin));
                display::success(&i18n.t_with_vars("ollama.binary_removed", &[("path", bin)]));
            }
        }
        let _ = shell::run_quiet("sudo rm -rf /usr/local/lib/ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -rf /lib/ollama 2>/dev/null");

        // 4. Supprimer les données
        let _ = shell::run_quiet("sudo rm -rf /usr/share/ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -rf /usr/lib/ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -rf ~/.ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -rf /home/ollama 2>/dev/null");  // Custom models dir
        display::success(&i18n.t("ollama.data_removed"));

        // 5. Supprimer l'utilisateur et le groupe système
        if shell::run_quiet("id -u ollama >/dev/null 2>&1").is_ok() {
            let _ = shell::run_quiet("sudo userdel ollama 2>/dev/null");
        }
        if shell::run_quiet("getent group ollama >/dev/null 2>&1").is_ok() {
            let _ = shell::run_quiet("sudo groupdel ollama 2>/dev/null");
        }
        // Remove systemd drop-in
        let _ = shell::run_quiet("sudo rm -rf /etc/systemd/system/ollama.service.d 2>/dev/null");
        display::success(&i18n.t("ollama.user_removed"));

        // 6. Vérification finale
        display::info("Verifying uninstallation...");
        if shell::run("command -v ollama >/dev/null 2>&1").is_ok() {
            display::warning("ollama binary still found - manual cleanup may be needed");
        } else {
            display::success("ollama binary not found - uninstallation complete");
        }

        display::success(&i18n.t("ollama.uninstalled"));
        Ok(())
    }
}