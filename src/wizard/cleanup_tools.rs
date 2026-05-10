use anyhow::Result;
use dialoguer::{Select, Confirm};
use crate::config::{self, I18n, WzllamaState};
use crate::core::shell;
use crate::display;
use crate::tools;
use crate::tools::ollama::OllamaTool;

pub fn run(i18n: &I18n, state: &mut WzllamaState) -> Result<()> {
    loop {
        let tools = tools::get_all_tools();
        let mut items: Vec<String> = tools.iter().map(|t| {
            let installed = match t.id() {
                "ollama" => state.installed.ollama,
                "openclaw" => state.installed.openclaw,
                "claude_code" => state.installed.claude_code,
                "hermes_agent" => state.installed.hermes_agent,
                "opencode" => state.installed.opencode,
                "codex" => state.installed.codex,
                "copilot_cli" => state.installed.copilot_cli,
                "droid" => state.installed.droid,
                "pi" => state.installed.pi,
                "pool" => state.installed.pool,
                _ => false,
            };
            format!("{} {}", if installed { "✅" } else { "  " }, t.name())
        }).collect();
        items.push(i18n.t("menu.back"));

        let sel = Select::new()
            .with_prompt(i18n.t("cleanup.choose_tool"))
            .items(&items)
            .default(0)
            .interact()?;

        if sel == tools.len() { return Ok(()); }

        let tool = &tools[sel];
        let installed = is_installed(tool.id());

        if !installed {
            display::info(&i18n.t("cleanup.not_installed"));
            continue;
        }

        if !Confirm::new()
            .with_prompt(i18n.t_with_vars("cleanup.uninstall_confirm", &[("tool", &tool.name())]))
            .default(false)
            .interact()?
        {
            continue;
        }

        display::section(&i18n.t("cleanup.uninstalling"));

        match tool.id() {
            "ollama" => {
                OllamaTool::uninstall(i18n)?;
                state.installed.ollama = false;
            }
            "open_webui" => {
                let _ = shell::run("sudo docker stop open-webui 2>/dev/null");
                let _ = shell::run("sudo docker rm open-webui 2>/dev/null");
                display::success(&i18n.t("cleanup.openwebui_uninstalled"));
                state.installed.open_webui = false;
            }
            _ => {
                let pkg = npm_package(tool.id());
                let cmd = format!("sudo npm uninstall -g {}", pkg);
                if shell::run(&cmd).is_ok() {
                    display::success(&i18n.t_with_vars("cleanup.uninstalled", &[("tool", &tool.name())]));
                    mark_uninstalled(tool.id(), state);
                } else {
                    display::warning(&i18n.t("cleanup.manual_uninstall"));
                }
            }
        }
        config::state::save(state)?;
    }
}

fn is_installed(id: &str) -> bool {
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