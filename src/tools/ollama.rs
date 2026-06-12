use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};
use crate::wizard::menu_picker;
use anyhow::Result;
use dialoguer::Confirm;

pub struct OllamaTool;

impl Tool for OllamaTool {
    fn id(&self) -> &str {
        "ollama"
    }
    fn name(&self) -> &str {
        "Ollama"
    }
    fn description(&self, i18n: &I18n) -> String {
        i18n.t("tool.ollama.description")
    }

    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        if shell::is_installed("ollama") {
            ToolStatus::Installed
        } else {
            ToolStatus::NotInstalled
        }
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

    #[cfg(unix)]
    pub fn start() -> Result<()> {
        shell::run_live("sudo systemctl start ollama")?;
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn start() -> Result<()> {
        use std::process::{Command, Stdio};
        // Start ollama serve as a detached background process
        Command::new("ollama")
            .arg("serve")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start ollama: {}", e))?;
        Ok(())
    }

    #[allow(dead_code)]
    #[cfg(unix)]
    pub fn stop() -> Result<()> {
        shell::run_live("sudo systemctl stop ollama")?;
        Ok(())
    }

    #[allow(dead_code)]
    #[cfg(not(unix))]
    pub fn stop() -> Result<()> {
        shell::run_quiet("taskkill /F /IM ollama.exe")
            .map(|_| ())
            .unwrap_or(()); // best-effort
        Ok(())
    }

    #[cfg(unix)]
    pub fn enable_autostart() -> Result<()> {
        shell::run_live("sudo systemctl enable ollama")?;
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn enable_autostart() -> Result<()> {
        // Ollama on Windows registers itself as a startup app during installation
        display::info("Ollama manages its own startup on Windows.");
        Ok(())
    }

    #[cfg(unix)]
    fn is_autostart_enabled() -> bool {
        shell::run_quiet("systemctl is-enabled ollama 2>/dev/null | grep -q enabled").is_ok()
    }

    #[cfg(not(unix))]
    fn is_autostart_enabled() -> bool {
        true // Ollama handles this automatically on Windows
    }

    // ─── Setup (Unix-only) ──────────────────────

    #[cfg(unix)]
    pub fn setup_ollama_user_dir() -> Result<()> {
        if shell::run("id -u ollama >/dev/null 2>&1").is_err() {
            shell::run("sudo useradd -r -s /bin/false ollama")?;
        }
        shell::run("sudo mkdir -p /home/ollama")?;
        shell::run("sudo chown ollama:ollama /home/ollama")?;
        shell::run("sudo chmod 755 /home/ollama")?;
        if std::path::Path::new("/usr/share/ollama").exists() {
            shell::run("sudo rm -rf /usr/share/ollama")?;
        }
        shell::run("sudo ln -sf /home/ollama /usr/share/ollama")?;
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn setup_ollama_user_dir() -> Result<()> {
        Ok(()) // Not needed on Windows
    }

    // ─── ensure_running ─────────────────────────

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
                display::success(&i18n.t("config.generated_env"));
            } else {
                anyhow::bail!("{}", i18n.t("ollama.required_mandatory_exit"));
            }
        }

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
                // Wait a moment for the server to come up
                for _ in 0..10 {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if Self::is_running() {
                        break;
                    }
                }
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

        let env_path = crate::config::env::EnvConfig::env_path();
        if !env_path.exists() {
            let config = crate::config::env::EnvConfig::default();
            config.save()?;
            display::success(&i18n.t("config.generated_env"));
            crate::config::shells::install_all_shells(i18n)?;
        }
        Ok(())
    }

    // ─── Installation ────────────────────────────

    #[cfg(unix)]
    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        Self::setup_ollama_user_dir()?;
        shell::run_live("curl -fsSL https://ollama.com/install.sh | sh")?;

        shell::run("sudo mkdir -p /etc/systemd/system/ollama.service.d")?;
        let config = crate::config::env::EnvConfig::load();
        let mut env_lines = vec!["[Service]".to_string()];
        env_lines.push("Environment=\"OLLAMA_MODELS=/home/ollama\"".to_string());
        env_lines.push(format!(
            "Environment=\"OLLAMA_ORIGINS={}\"",
            config.ollama.origins
        ));
        env_lines.push(format!(
            "Environment=\"OLLAMA_KEEP_ALIVE={}\"",
            config.ollama.keep_alive
        ));
        env_lines.push(format!(
            "Environment=\"OLLAMA_NUM_PARALLEL={}\"",
            config.ollama.num_parallel
        ));
        env_lines.push(format!(
            "Environment=\"OLLAMA_MAX_LOADED_MODELS={}\"",
            config.ollama.max_loaded_models
        ));
        env_lines.push(format!(
            "Environment=\"OLLAMA_FLASH_ATTENTION={}\"",
            if config.ollama.flash_attention { 1 } else { 0 }
        ));
        env_lines.push(format!(
            "Environment=\"OLLAMA_KV_CACHE_TYPE={}\"",
            config.ollama.kv_cache_type
        ));
        env_lines.push(format!(
            "Environment=\"OLLAMA_CONTEXT_LENGTH={}\"",
            config.ollama.context_length
        ));
        if config.ollama.max_vram > 0 {
            env_lines.push(format!(
                "Environment=\"OLLAMA_MAX_VRAM={}\"",
                config.ollama.max_vram
            ));
        }
        if !config.ollama.cuda_visible_devices.is_empty() {
            env_lines.push(format!(
                "Environment=\"CUDA_VISIBLE_DEVICES={}\"",
                config.ollama.cuda_visible_devices
            ));
        }
        let override_content = env_lines.join("\\n");
        shell::run_quiet(&format!(
            "printf '{}' | sudo tee /etc/systemd/system/ollama.service.d/override.conf > /dev/null",
            override_content
        ))?;
        shell::run_quiet("sudo systemctl daemon-reload")?;
        shell::run_live("sudo systemctl enable ollama")?;
        shell::run_live("sudo systemctl start ollama")?;

        for _ in 0..10 {
            if crate::core::ollama_api::detect_url().is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        let config = crate::config::env::EnvConfig::default();
        config.save()?;
        crate::config::shells::install_all_shells_cli()?;
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn install(_i18n: &I18n) -> Result<()> {
        display::info("Installing Ollama via winget...");
        // Try winget first (available on Windows 10/11)
        let result = std::process::Command::new("winget")
            .args(["install", "--id", "Ollama.Ollama", "-e", "--silent"])
            .status();

        match result {
            Ok(s) if s.success() => {
                display::success("Ollama installed successfully via winget.");
            }
            _ => {
                // Fallback: open browser to download page
                display::warning("winget not available or install failed.");
                display::info("Opening Ollama download page in your browser...");
                shell::open_url("https://ollama.com/download/windows");
                display::info("Please install Ollama manually, then re-run wzllama.");
                anyhow::bail!("Manual Ollama installation required. Download from https://ollama.com/download/windows");
            }
        }

        // Add Ollama to user PATH (standard install location)
        let ollama_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("AppData")
            .join("Local")
            .join("Programs")
            .join("Ollama");

        if ollama_dir.exists() {
            let ollama_dir_str = ollama_dir.to_string_lossy().to_string();
            let user_path = std::env::var("PATH").unwrap_or_default();
            if !user_path.contains(&ollama_dir_str) {
                let _ = std::process::Command::new("powershell")
                    .args([
                        "-Command",
                        &format!(
                            "[Environment]::SetEnvironmentVariable('PATH', \
                             ([Environment]::GetEnvironmentVariable('PATH','User') + ';{}'), 'User')",
                            ollama_dir_str
                        ),
                    ])
                    .status();
                // Also update current process PATH immediately
                let new_path = format!("{};{}", user_path, ollama_dir_str);
                std::env::set_var("PATH", &new_path);
                display::success(&format!("Added {} to PATH", ollama_dir_str));
            }
        }

        // Wait for ollama to be available in PATH
        for _ in 0..15 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if shell::is_installed("ollama") {
                break;
            }
        }

        let config = crate::config::env::EnvConfig::default();
        config.save()?;
        Ok(())
    }

    // ─── Update ──────────────────────────────────

    pub fn update(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("Checking for Ollama updates...");
        #[cfg(unix)]
        shell::run_live("curl -fsSL https://ollama.com/install.sh | sh")?;
        #[cfg(not(unix))]
        {
            let result = std::process::Command::new("winget")
                .args(["upgrade", "--id", "Ollama.Ollama", "-e", "--silent"])
                .status();
            match result {
                Ok(s) if s.success() => {}
                _ => {
                    display::info("Open https://ollama.com/download/windows to update manually.");
                }
            }
        }
        display::success("Ollama is up to date");
        Ok(())
    }

    // ─── Launch ──────────────────────────────────

    pub fn launch(i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::run(&i18n.t_with_vars("tool.ollama.run_model", &[("model", m)]));
                let cmd = format!("ollama run {}", m);
                shell::exec(&cmd);
            }
            None => {
                display::info(&i18n.t("ollama.choose_model"));
                if let Some(selected) = menu_picker::pick_model(i18n)? {
                    let cmd = format!("ollama run {}", selected);
                    shell::exec(&cmd);
                } else {
                    display::warning(&i18n.t("ollama.no_models"));
                }
            }
        }
        Ok(())
    }

    // ─── Désinstallation ────────────────────────

    #[cfg(unix)]
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

        let _ = shell::run_quiet("sudo systemctl stop ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo systemctl disable ollama 2>/dev/null");
        display::success(&i18n.t("ollama.service_stopped"));

        let _ = shell::run_quiet("sudo rm -f /etc/systemd/system/ollama.service 2>/dev/null");
        let _ = shell::run_quiet("sudo systemctl daemon-reload 2>/dev/null");
        display::success(&i18n.t("ollama.service_removed"));

        if let Ok((stdout, _)) = shell::run_quiet("which ollama 2>/dev/null") {
            let bin = stdout.trim();
            if !bin.is_empty() {
                let _ = shell::run(&format!("sudo rm -f {}", bin));
                display::success(&i18n.t_with_vars("ollama.binary_removed", &[("path", bin)]));
            }
        }
        let _ = shell::run_quiet("sudo rm -rf /usr/local/lib/ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -rf /lib/ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -rf /usr/share/ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -rf /usr/lib/ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -rf ~/.ollama 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -rf /home/ollama 2>/dev/null");
        display::success(&i18n.t("ollama.data_removed"));

        if shell::run_quiet("id -u ollama >/dev/null 2>&1").is_ok() {
            let _ = shell::run_quiet("sudo userdel ollama 2>/dev/null");
        }
        if shell::run_quiet("getent group ollama >/dev/null 2>&1").is_ok() {
            let _ = shell::run_quiet("sudo groupdel ollama 2>/dev/null");
        }
        let _ = shell::run_quiet("sudo rm -rf /etc/systemd/system/ollama.service.d 2>/dev/null");
        display::success(&i18n.t("ollama.user_removed"));

        if shell::is_installed("ollama") {
            display::warning("ollama binary still found - manual cleanup may be needed");
        } else {
            display::success("ollama binary not found - uninstallation complete");
        }
        display::success(&i18n.t("ollama.uninstalled"));
        Ok(())
    }

    #[cfg(not(unix))]
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

        let result = std::process::Command::new("winget")
            .args(["uninstall", "--id", "Ollama.Ollama", "-e", "--silent"])
            .status();

        match result {
            Ok(s) if s.success() => {
                display::success(&i18n.t("ollama.uninstalled"));
            }
            _ => {
                display::warning("winget uninstall failed. Use Windows Settings > Apps to remove Ollama manually.");
            }
        }
        Ok(())
    }
}
