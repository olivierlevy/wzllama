use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::docker;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct OpenWebUITool;

impl Tool for OpenWebUITool {
    fn id(&self) -> &str { "open_webui" }
    fn name(&self) -> &str { "Open WebUI" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.openwebui.description") }
    fn status(&self) -> ToolStatus {
        // Vérifier si le conteneur existe avec docker inspect (plus fiable)
        // Si Docker n'est pas démarré, on retourne NotInstalled (le menu lancera ensure_ready)
        if !docker::is_running() {
            return ToolStatus::NotInstalled;
        }
        // Utiliser sudo pour les vérifications Docker
        let exists = shell::run("sudo docker inspect open-webui >/dev/null 2>&1").is_ok();
        if exists { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self) -> Result<()> {
        // Docker doit être démarré avant (géré par menu_tools via ensure_ready)
        if !docker::is_running() {
            anyhow::bail!("Docker n'est pas prêt");
        }
        
        display::info("Vérification du conteneur Open WebUI...");
        
        // Vérifier si le conteneur existe avec docker inspect (plus fiable)
        // Utiliser sudo car les permissions Docker peuvent nécessiter sudo
        let container_exists = shell::run("sudo docker inspect open-webui >/dev/null 2>&1").is_ok();
        
        if container_exists {
            // Le conteneur existe - essayer de le démarrer
            display::info("Démarrage du conteneur existant...");
            shell::run_live("sudo docker start open-webui")?;
            display::success("Conteneur Open WebUI démarré.");
        } else {
            // Le conteneur n'existe pas - le créer
            display::info("Pull de l'image Open WebUI (~500MB)...");
            shell::run_live("sudo docker run -d -p 3000:8080 --add-host=host.docker.internal:host-gateway -v open-webui:/app/backend/data --name open-webui --restart always ghcr.io/open-webui/open-webui:main")?;
        }
        Ok(())
    }
    
    fn requires_docker(&self) -> bool { true }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>) -> Result<()> {
        let url = "http://localhost:3000";
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


impl OpenWebUITool {
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new()
            .with_prompt(i18n.t("tool.openwebui.uninstall_confirm"))
            .default(false)
            .interact()?
        {
            return Ok(());
        }
        // Vérifier si le conteneur existe avant de tenter de le supprimer
        // Utiliser sudo car les permissions Docker peuvent nécessiter sudo
        if shell::run("sudo docker inspect open-webui >/dev/null 2>&1").is_ok() {
            display::info("Arrêt du conteneur Open WebUI...");
            let _ = shell::run_live("sudo docker stop open-webui 2>/dev/null");
            display::info("Suppression du conteneur...");
            let _ = shell::run_live("sudo docker rm open-webui 2>/dev/null");
            display::info("Suppression du volume...");
            let _ = shell::run_live("sudo docker volume rm open-webui 2>/dev/null");
        } else {
            display::info("Aucun conteneur Open WebUI trouvé.");
        }
        display::success(&i18n.t("tool.openwebui.uninstalled"));
        Ok(())
    }
}
