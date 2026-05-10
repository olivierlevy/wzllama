use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::{display, tools};
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenWebUITool;

impl Tool for OpenWebUITool {
    fn id(&self) -> &str { "open_webui" }
    fn name(&self) -> &str { "Open WebUI" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.openwebui.description") }

    fn status(&self) -> ToolStatus {
        let exists = shell::run("sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$'").is_ok();
        let running = shell::run("sudo docker ps --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$'").is_ok();
        
        if running {
            ToolStatus::Running
        } else if exists {
            ToolStatus::Installed
        } else {
            ToolStatus::NotInstalled {
                install_cmd: "sudo docker run -d -p 3000:8080 --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data --name open-webui --restart always ghcr.io/open-webui/open-webui:main".into()
            }
        }
    }

    fn install(&self) -> Result<()> {
        tools::docker::start()?;
        // Vérifier si le conteneur existe déjà
        let exists = shell::run("sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$'").is_ok();
        
        if exists {
            let running = shell::run("sudo docker ps --format '{{.Names}}' 2>/dev/null | grep -q '^open-webui$'").is_ok();
            if running {
                display::success("Open WebUI est déjà en cours d'exécution");
                return Ok(());
            }
            // Conteneur arrêté : proposer de le supprimer et recréer
            display::warning("Un conteneur 'open-webui' existe déjà mais est arrêté.");
            if Confirm::new()
                .with_prompt("Supprimer l'ancien conteneur et le recréer ?")
                .default(true)
                .interact()?
            {
                shell::run("sudo docker rm open-webui")?;
            } else {
                // Juste redémarrer l'ancien
                shell::run("sudo docker start open-webui")?;
                display::success("Open WebUI redémarré");
                return Ok(());
            }
        }
        
        // Créer le nouveau conteneur
        let cmd = "sudo docker run -d -p 3000:8080 --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data --name open-webui --restart always ghcr.io/open-webui/open-webui:main";
        println!("{}", cmd);
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
