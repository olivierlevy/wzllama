use anyhow::Result;
use dialoguer::Select;
use crate::config::{self, I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools;
use crate::tools::claude_code::ClaudeCodeTool;
use crate::tools::codex::CodexTool;
use crate::tools::droid::DroidTool;
use crate::tools::hermes::HermesTool;
use crate::tools::ollama::OllamaTool;
use crate::tools::open_webui::OpenWebUITool;
use crate::tools::openclaw::OpenClawTool;
use crate::tools::opencode::OpenCodeTool;
use crate::tools::pi::PiTool;
use crate::tools::tool_trait::Tool;

pub fn run(i18n: &I18n, state: &mut WzllamaState) -> Result<()> {
    loop {
        // Resynchroniser l'état avec la réalité à chaque itération
        sync_tools_state(state);
        let all_tools = tools::get_all_tools();
        let installed_tools: Vec<&Box<dyn Tool>> = all_tools.iter()
            .filter(|t| cleanup_is_installed(t.id()))
            .collect();

        if installed_tools.is_empty() {
            display::info(&i18n.t("cleanup.no_tools"));
            return Ok(());
        }

        let mut items: Vec<String> = installed_tools.iter()
            .map(|t| format!("🗑️  {}", t.name()))
            .collect();
        items.push(i18n.t("menu.back"));

        let sel = match Select::new()
            .with_prompt(i18n.t("cleanup.choose_tool"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact_opt()? {
            Some(s) => s,
            None => return Ok(()), // Escape pressed
        };

        if sel == installed_tools.len() { return Ok(()); }

        let tool = installed_tools[sel];

        display::section(&i18n.t("cleanup.uninstalling"));

        match tool.id() {
            "ollama" => OllamaTool::uninstall(i18n)?,
            "open_webui" => OpenWebUITool::uninstall(i18n)?,
            "openclaw" => OpenClawTool::uninstall(i18n)?,
            "claude_code" => ClaudeCodeTool::uninstall(i18n)?,
            "hermes_agent" => HermesTool::uninstall(i18n)?,
            "opencode" => OpenCodeTool::uninstall(i18n)?,
            "codex" => CodexTool::uninstall(i18n)?,
            "droid" => DroidTool::uninstall(i18n)?,
            "pi" => PiTool::uninstall(i18n)?,
            _ => display::error(&i18n.t("cleanup.manual_uninstall")),
        }
        mark_uninstalled(tool.id(), state);
        config::state::save(state)?;
    }
}

fn cleanup_is_installed(id: &str) -> bool {
    match id {
        "ollama" => shell::is_installed("ollama"),
        "open_webui" => shell::run("sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok(),
        "openclaw" => shell::is_installed("openclaw"),
        "claude_code" => shell::is_installed("claude"),
        "hermes_agent" => shell::is_installed("hermes"),
        "opencode" => shell::is_installed("opencode"),
        "codex" => shell::is_installed("codex"),
        "droid" => shell::is_installed("droid"),
        "pi" => shell::is_installed("pi"),
        "pool" => shell::is_installed("pool"),
        _ => false,
    }
}

#[allow(dead_code)]
fn npm_package(id: &str) -> &str {
    match id {
        "openclaw" => "openclaw",
        "claude_code" => "@anthropic-ai/claude-code",
        "hermes_agent" => "hermes-agent",
        "opencode" => "opencode-ai",
        "codex" => "@openai/codex",
        "droid" => "@factoryai/droid",
        "pi" => "@mariozechner/pi-coding-agent",
        _ => id,
    }
}

fn mark_uninstalled(id: &str, state: &mut WzllamaState) {
    match id {
        "ollama" => state.installed.ollama = false,
        "open_webui" => state.installed.open_webui = false,
        "openclaw" => state.installed.openclaw = false,
        "claude_code" => state.installed.claude_code = false,
        "hermes_agent" => state.installed.hermes_agent = false,
        "opencode" => state.installed.opencode = false,
        "codex" => state.installed.codex = false,
        "copilot_cli" => state.installed.copilot_cli = false,
        "droid" => state.installed.droid = false,
        "pi" => state.installed.pi = false,
        "pool" => state.installed.pool = false,
        _ => {}
    }
}

fn sync_tools_state(state: &mut WzllamaState) {
    state.installed.ollama = shell::is_installed("ollama");
    state.installed.open_webui = shell::run("sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok();
    state.installed.openclaw = shell::is_installed("openclaw");
    state.installed.claude_code = shell::is_installed("claude");
    state.installed.hermes_agent = shell::is_installed("hermes");
    state.installed.opencode = shell::is_installed("opencode");
    state.installed.codex = shell::is_installed("codex");
    state.installed.droid = shell::is_installed("droid");
    state.installed.pi = shell::is_installed("pi");
    state.installed.pool = shell::is_installed("pool");
}
