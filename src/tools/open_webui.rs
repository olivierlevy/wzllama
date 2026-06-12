use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::docker;
use crate::tools::tool_trait::{Tool, ToolStatus};
use anyhow::Result;
use dialoguer::Confirm;

pub struct OpenWebUITool;

impl Tool for OpenWebUITool {
    fn id(&self) -> &str {
        "open_webui"
    }
    fn name(&self) -> &str {
        "Open WebUI"
    }
    fn description(&self, i18n: &I18n) -> String {
        i18n.t("tool.openwebui.description")
    }
    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        // Check if container exists with docker inspect (more reliable)
        // If Docker is not running, return NotInstalled (menu will launch ensure_ready)
        if !docker::is_running() {
            return ToolStatus::NotInstalled;
        }
        // Try without sudo first
        let exists = docker::run("inspect open-webui");
        if exists {
            ToolStatus::Installed
        } else {
            ToolStatus::NotInstalled
        }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        OpenWebUITool::install(i18n)
    }

    fn update(&self, i18n: &I18n) -> Result<()> {
        OpenWebUITool::update(i18n)
    }

    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        OpenWebUITool::uninstall(i18n)
    }

    fn requires_docker(&self) -> bool {
        true
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        OpenWebUITool::launch(i18n, state, model)
    }
}

impl OpenWebUITool {
    pub fn pull() -> Result<()> {
        docker::run_live("pull ghcr.io/open-webui/open-webui:ollama")?;
        #[cfg(unix)]
        docker::run_live(
            "run -d \
            --network=host \
            --add-host=host.docker.internal:host-gateway \
            -v open-webui:/app/backend/data \
            -e OLLAMA_BASE_URL=http://127.0.0.1:11434 \
            --name open-webui \
            --restart always \
            ghcr.io/open-webui/open-webui:ollama",
        )?;
        #[cfg(not(unix))]
        docker::run_live(
            "run -d \
            -p 3000:8080 \
            -v open-webui:/app/backend/data \
            -e OLLAMA_BASE_URL=http://host.docker.internal:11434 \
            --name open-webui \
            --restart always \
            ghcr.io/open-webui/open-webui:main",
        )?;
        Ok(())
    }
    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        // Docker must be running first (handled by menu_tools via ensure_ready)
        if !docker::is_running() {
            anyhow::bail!("Docker is not ready");
        }

        display::info("Checking Open WebUI container...");

        // Check if container exists with docker inspect (more reliable)
        // Try without sudo first
        let container_exists = docker::run("inspect open-webui");

        if container_exists {
            // Container exists - try to start it
            display::info("Starting existing container...");
            docker::run_live("start open-webui")?;
            display::success("Open WebUI container started.");
        } else {
            // Container does not exist - create it
            display::info("Pulling Open WebUI image (~500MB)...");
            let _ = Self::pull();
        }
        Ok(())
    }
    pub fn update(i18n: &I18n) -> Result<()> {
        if docker::run("inspect open-webui") {
            display::info(&i18n.t("tool.openwebui.uninstall_stop"));
            let _ = docker::run_live("stop open-webui 2>/dev/null");
            display::info(&i18n.t("tool.openwebui.uninstall_rm"));
            let _ = docker::run_live("rm -f open-webui 2>/dev/null");
            display::info(&i18n.t("tool.openwebui.uninstall_rm_volume"));
            let _ = Self::pull();
        }
        display::success(&i18n.t("tool.openwebui.updated"));
        Ok(())
    }
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new()
            .with_prompt(i18n.t("tool.openwebui.uninstall_confirm"))
            .default(false)
            .interact()?
        {
            return Ok(());
        }
        // Check si le conteneur existe avant de tenter de le supprimer
        if docker::run("inspect open-webui") {
            display::info(&i18n.t("tool.openwebui.uninstall_stop"));
            let _ = docker::run_live("stop open-webui 2>/dev/null");
            display::info(&i18n.t("tool.openwebui.uninstall_rm"));
            let _ = docker::run_live("rm -f open-webui 2>/dev/null");
            display::info(&i18n.t("tool.openwebui.uninstall_rm_volume"));
            if Confirm::new()
                .with_prompt(i18n.t("tool.openwebui.uninstall_confirm_volume"))
                .default(false)
                .interact()?
            {
                let _ = docker::run_live("volume rm open-webui 2>/dev/null");
            }
        } else {
            display::info(&i18n.t("tool.openwebui.uninstall_nothing"));
        }
        display::success(&i18n.t("tool.openwebui.uninstalled"));
        Ok(())
    }
    pub fn launch(i18n: &I18n, _state: &WzllamaState, _model: Option<&str>) -> Result<()> {
        let url = "http://localhost:8080";
        println!("🌐 Open WebUI : {}", url);
        println!("💡 {}", i18n.t("url.refresh_hint"));

        if Confirm::new()
            .with_prompt(i18n.t("url.open"))
            .default(true)
            .interact()?
        {
            shell::open_url(url);
        }
        Ok(())
    }
}
