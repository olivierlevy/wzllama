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
        // Essayer sans sudo d'abord
        let exists = docker::run("inspect open-webui");
        if exists { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self) -> Result<()> {
        // Docker doit être démarré avant (géré par menu_tools via ensure_ready)
        if !docker::is_running() {
            anyhow::bail!("Docker n'est pas prêt");
        }
        
        display::info("Vérification du conteneur Open WebUI...");
        
        // Vérifier si le conteneur existe avec docker inspect (plus fiable)
        // Essayer sans sudo d'abord
        let container_exists = docker::run("inspect open-webui");
        
        if container_exists {
            // Le conteneur existe - essayer de le démarrer
            display::info("Démarrage du conteneur existant...");
            docker::run_live("start open-webui")?;
            display::success("Conteneur Open WebUI démarré.");
        } else {
            // Le conteneur n'existe pas - le créer
            display::info("Pull de l'image Open WebUI (~500MB)...");
            let _ = OpenWebUITool::pull();
        }
        Ok(())
    }
    
    fn requires_docker(&self) -> bool { true }
    fn launch(&self, i18n: &I18n, _state: &WzllamaState, _model: Option<&str>) -> Result<()> {
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


impl OpenWebUITool {
    pub fn pull() -> Result<()> {
        docker::run_live("pull ghcr.io/open-webui/open-webui:ollama")?;
        docker::run_live("run -d \
            --network=host \
            --add-host=host.docker.internal:host-gateway \
            -v open-webui:/app/backend/data \
            -e OLLAMA_BASE_URL=http://127.0.0.1:11434 \
            --name open-webui \
            --restart always \
            ghcr.io/open-webui/open-webui:ollama")?;
        Ok(())
    }
    pub fn update(i18n: &I18n) -> Result<()> {
        if docker::run("inspect open-webui") {
            display::info(&i18n.t("tool.openwebui.uninstall_stop"));
            let _ = docker::run_live("stop open-webui 2>/dev/null");
            display::info(&i18n.t("tool.openwebui.uninstall_rm"));
            let _ = docker::run_live("rm -f open-webui 2>/dev/null");
            display::info(&i18n.t("tool.openwebui.uninstall_rm_volume"));
            let _ = OpenWebUITool::pull();
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
        // Vérifier si le conteneur existe avant de tenter de le supprimer
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
}
