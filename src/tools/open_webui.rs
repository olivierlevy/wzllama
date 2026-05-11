use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::tools::docker;
use crate::{display, tools};
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenWebUITool;

impl Tool for OpenWebUITool {
    fn id(&self) -> &str { "open_webui" }
    fn name(&self) -> &str { "Open WebUI" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.openwebui.description") }
    fn status(&self) -> ToolStatus {
        if !docker::is_running() {
            let _ = docker::start();
            let _ = docker::startup();
        }
        let exists = shell::run("sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$'").is_ok();
        if exists { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self) -> Result<()> {
        // Vérifications Docker déjà faites par menu_tools
        let exists = shell::run("sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$'").is_ok();
        if exists {
            shell::run("sudo docker start open-webui")?;
        } else {
            shell::run_live("sudo docker run -d -p 3000:8080 --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data --name open-webui --restart always ghcr.io/open-webui/open-webui:main")?;
        }
        Ok(())
    }
    fn requires_docker(&self) -> bool { true }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>) -> Result<()> {
        let url = "http://localhost:3000";
        println!("🌐 Open WebUI : {}", url);
    
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


impl OpenWebUITool {
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new()
            .with_prompt(i18n.t("tool.openwebui.uninstall_confirm"))
            .default(false)
            .interact()?
        {
            return Ok(());
        }
        let _ = shell::run("sudo docker stop open-webui 2>/dev/null");
        let _ = shell::run("sudo docker rm open-webui 2>/dev/null");
        let _ = shell::run("sudo docker volume rm open-webui 2>/dev/null");
        display::success(&i18n.t("tool.openwebui.uninstalled"));
        Ok(())
    }
}
