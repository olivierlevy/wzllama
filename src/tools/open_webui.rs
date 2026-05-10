use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenWebUITool;

impl Tool for OpenWebUITool {
    fn id(&self) -> &str { "open_webui" }
    fn name(&self) -> &str { "Open WebUI" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.openwebui.description") }

    fn status(&self) -> ToolStatus {
        let ok = shell::run("sudo docker ps --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok();
        if ok { ToolStatus::Running }
        else if shell::run("sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok() {
            ToolStatus::Installed
        } else {
            ToolStatus::NotInstalled {
                install_cmd: "sudo docker run -d -p 3000:8080 --name open-webui --restart always ghcr.io/open-webui/open-webui:main".into()
            }
        }
    }

    fn install(&self) -> Result<()> {
        let cmd = "sudo docker run -d -p 3000:8080 --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data --name open-webui --restart always ghcr.io/open-webui/open-webui:main";
        shell::run(cmd)?;
        Ok(())
    }

    fn launch(&self, _i18n: &I18n, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("🌐 Open WebUI : http://localhost:3000");
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
