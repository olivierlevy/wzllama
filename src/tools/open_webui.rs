use anyhow::Result;
use crate::config::WzllamaState;
use crate::core::shell;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenWebUITool;

impl Tool for OpenWebUITool {
    fn id(&self) -> &str { "open_webui" }
    fn name(&self) -> &str { "Open WebUI" }
    fn description(&self) -> &str { "Interface web pour vos modèles IA" }

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

    fn launch(&self, _state: &WzllamaState, _model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("🌐 Open WebUI : http://localhost:3000");
        Ok(())
    }
}