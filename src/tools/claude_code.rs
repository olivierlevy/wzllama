use anyhow::Result;
use dialoguer::Confirm;
use crate::config::{I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools::tool_trait::{Tool, ToolStatus};

pub struct ClaudeCodeTool;

impl Tool for ClaudeCodeTool {
    fn id(&self) -> &str { "claude_code" }
    fn name(&self) -> &str { "Claude Code" }
    fn description(&self, i18n: &I18n) -> String { i18n.t("tool.claude.description") }
    fn status(&self) -> ToolStatus {
        if shell::is_installed_with_local_bin("claude") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self, i18n: &I18n) -> Result<()> {
        ClaudeCodeTool::install(i18n)
    }
    fn uninstall(&self, i18n: &I18n) -> Result<()> {
        ClaudeCodeTool::uninstall(i18n)
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        ClaudeCodeTool::launch(i18n, state, model)
    }
}

impl ClaudeCodeTool {
    pub fn install(i18n: &I18n) -> Result<()> {
        let _ = i18n;
        shell::run_live("curl -fsSL https://claude.ai/install.sh | bash")?;
        Ok(())
    }
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.claude.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        let _ = shell::run_quiet("sudo npm uninstall -g @anthropic-ai/claude-code 2>/dev/null");
        let _ = shell::run_quiet("sudo rm -f /usr/bin/claude 2>/dev/null");
        let _ = shell::run_quiet("rm -f ~/.local/bin/claude 2>/dev/null");
        let _ = shell::run_quiet("rm -rf ~/.claude* 2>/dev/null");
        display::success(&i18n.t("tool.claude.uninstalled"));
        Ok(())
    }
    pub fn launch(i18n: &I18n, state: &WzllamaState, model: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                display::run(&i18n.t_with_vars("tool.claude_code.run_model", &[("model", &m)]));
                let cmd: String = format!("ollama launch claude --model {}", m);
                println!("{}", cmd); shell::exec(&cmd);
            }
            None => {
                display::comment(&i18n.t("tool.claude_code.no_model"));
                println!("ollama launch claude");
            }
        }
        Ok(())
    }
}