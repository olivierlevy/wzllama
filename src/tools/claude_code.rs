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
        if shell::is_installed("claude-code") { ToolStatus::Installed }
        else { ToolStatus::NotInstalled { install_cmd: "npm install -g @anthropic-ai/claude-code".into() } }
    }

    fn install(&self) -> Result<()> { shell::run("npm install -g @anthropic-ai/claude-code")?; Ok(()) }

    fn launch(&self, i18n: &I18n, _state: &WzllamaState, model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        println!("export ANTHROPIC_BASE_URL=http://localhost:11434/v1");
        println!("export ANTHROPIC_API_KEY=ollama");
        match model {
            Some(m) => println!("claude --model {}", m),
            None => {
                display::info(&i18n.t("tool.claude_code.no_model"));
                println!("claude");
            }
        }
        Ok(())
    }
}
impl ClaudeCodeTool {
    pub fn uninstall(i18n: &I18n) -> Result<()> {
        if !Confirm::new().with_prompt(i18n.t("tool.claude.uninstall_confirm")).default(false).interact()? {
            return Ok(());
        }
        // npm
        let _ = shell::run("sudo npm uninstall -g @anthropic-ai/claude-code 2>/dev/null");
        // installeur natif
        let _ = shell::run("sudo rm -f /usr/bin/claude 2>/dev/null");
        let _ = shell::run("rm -rf ~/.claude* 2>/dev/null");
        display::success(&i18n.t("tool.claude.uninstalled"));
        Ok(())
    }
}