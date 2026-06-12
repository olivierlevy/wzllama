use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};
use anyhow::Result;
use dialoguer::Confirm;

pub struct OpenCodeTool;

impl Tool for OpenCodeTool {
    fn id(&self) -> &str {
        "opencode"
    }
    fn name(&self) -> &str {
        "OpenCode"
    }
    fn description(&self, i18n: &I18n) -> String {
        i18n.t("tool.opencode.description")
    }
    fn status(&self, _state: &WzllamaState) -> ToolStatus {
        if shell::is_installed_with_local_bin("opencode") {
            ToolStatus::Installed
        } else {
            ToolStatus::NotInstalled
        }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        OpenCodeTool::install(i18n)
    }
    fn update(&self, i18n: &I18n) -> Result<()> {
        OpenCodeTool::update(i18n)
    }
    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        OpenCodeTool::uninstall(i18n)
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        OpenCodeTool::launch(i18n, state, model)
    }

    fn supports_agentic(&self) -> bool {
        true
    }
}

impl OpenCodeTool {
    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        #[cfg(unix)]
        shell::run_live("curl -fsSL https://opencode.ai/install | bash")?;
        #[cfg(not(unix))]
        {
            display::info("Installing OpenCode via npm...");
            shell::run_live("npm install -g @opencode-ai/cli")?;
        }
        Ok(())
    }
    pub fn update(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        display::info("Updating OpenCode...");
        #[cfg(unix)]
        shell::run_live("curl -fsSL https://opencode.ai/install | bash")?;
        #[cfg(not(unix))]
        shell::run_live("npm update -g @opencode-ai/cli")?;
        display::success("✅ OpenCode updated");
        Ok(())
    }
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new()
            .with_prompt(i18n.t("tool.opencode.uninstall_confirm"))
            .default(false)
            .interact()?
        {
            return Ok(());
        }
        #[cfg(unix)]
        {
            let _ =
                shell::run_quiet("rm -f /usr/local/bin/opencode ~/.local/bin/opencode 2>/dev/null");
            let _ = shell::run_quiet("rm -rf ~/.opencode* 2>/dev/null");
        }
        #[cfg(not(unix))]
        {
            let _ = shell::run_quiet("npm uninstall -g @opencode-ai/cli");
            let home = dirs::home_dir().unwrap_or_default();
            let _ = std::fs::remove_dir_all(home.join(".opencode"));
        }
        display::success(&i18n.t("tool.opencode.uninstalled"));
        Ok(())
    }
    pub fn launch(i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        display::info(&i18n.t("tool.opencode.auth"));
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::run(&i18n.t_with_vars("tool.opencode.run_model", &[("model", m)]));
                let cmd: String = format!("ollama launch opencode --model {}", m);
                println!("{}", cmd);
                shell::exec(&cmd);
            }
            None => {
                display::comment(&i18n.t("tool.opencode.no_model"));
                println!("ollama launch opencode");
            }
        }
        Ok(())
    }
}
