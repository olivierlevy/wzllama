use anyhow::Result;
use std::os::unix::process::CommandExt;
use std::process::Command;
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
        if shell::is_installed("claude") { ToolStatus::Installed } else { ToolStatus::NotInstalled }
    }
    fn install(&self) -> Result<()> {
        shell::run_live("curl -fsSL https://claude.ai/install.sh | bash")?;
        Ok(())
    }
    fn launch(&self, i18n: &I18n, state: &WzllamaState, model: Option<&str>, _fleet: Option<&str>) -> Result<()> {
        let model = model.or(state.last_model.as_deref());
        match model {
            Some(m) => {
                println!("ollama launch claude --model {}", m);
                Command::new("ollama").args(["launch", "claude", "--model", m]).exec();
            }
            None => {
                display::comment(&i18n.t("tool.claude_code.no_model"));
                println!("ollama launch claude");
                Command::new("ollama").args(["launch", "claude"]).exec();
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
        let _ = shell::run("sudo npm uninstall -g @anthropic-ai/claude-code 2>/dev/null");
        let _ = shell::run("sudo rm -f /usr/bin/claude 2>/dev/null");
        let _ = shell::run("rm -f ~/.local/bin/claude 2>/dev/null");
        let _ = shell::run("rm -rf ~/.claude* 2>/dev/null");
        display::success(&i18n.t("tool.claude.uninstalled"));
        Ok(())
    }
}