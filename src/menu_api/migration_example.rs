//! Example migration: Converting menu_tools.rs to use menu_api
//!
//! This file demonstrates how to migrate an existing wizard menu
//! to use the new menu_api system.

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{
    MenuTree, MenuItem, ActionDispatcher,
    ActionContext, ActionResult, ClosureAction, ToolAction,
};

/// Migration example for menu_tools.rs
/// 
/// BEFORE (original code pattern):
/// ```rust
/// pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
///     loop {
///         let items = vec![...];
///         let sel = Select::new().items(&items).interact_opt()?;
///         match sel { ... }
///     }
/// }
/// ```
///
/// AFTER (using menu_api):
/// See the implementation below.
///
/// Tools menu action that wraps the existing tool launch logic
#[allow(dead_code)]
pub struct ToolLaunchAction {
    pub tool_id: String,
    pub tool_name: String,
}

impl ToolAction for ToolLaunchAction {
    fn id(&self) -> &str {
        &self.tool_id
    }

    fn name(&self) -> &str {
        &self.tool_name
    }

    fn execute(&self, ctx: &ActionContext) -> Result<ActionResult> {
        let model = ctx.get_param("model").map(|s| s.as_str());
        Ok(ActionResult::success_with(&format!(
            "Would launch {} with model {:?}",
            self.tool_name, model
        )))
    }
}

/// Create the tools menu tree structure
#[allow(dead_code)]
pub fn create_tools_menu() -> MenuTree {
    let root = MenuItem::branch("Tools")
        .add_submenu(
            MenuItem::branch("Installer")
                .add_submenu(MenuItem::leaf("Ollama").with_action("install_ollama"))
                .add_submenu(MenuItem::leaf("Open WebUI").with_action("install_openwebui"))
                .add_submenu(MenuItem::leaf("Claude Code").with_action("install_claude_code"))
                .add_submenu(MenuItem::leaf("OpenCode").with_action("install_opencode"))
        )
        .add_submenu(
            MenuItem::branch("Lancer")
                .add_submenu(MenuItem::leaf("Ollama").with_action("launch_ollama"))
                .add_submenu(MenuItem::leaf("Chat").with_action("launch_chat"))
                .add_submenu(MenuItem::leaf("Claude Code").with_action("launch_claude_code"))
        )
        .add_submenu(
            MenuItem::leaf("Retour").with_action("menu_back")
        );
    
    MenuTree::new("Tools").with_root(root)
}

/// Create an action dispatcher with all tool actions registered
#[allow(dead_code)]
pub fn create_tools_dispatcher() -> ActionDispatcher {
    let mut dispatcher = ActionDispatcher::new();
    
    dispatcher.register(Box::new(ClosureAction::new(
        "install_ollama",
        "Install Ollama",
        |_ctx| {
            println!("Installing Ollama...");
            Ok(ActionResult::success_with("Ollama installed"))
        },
    )));
    
    dispatcher.register(Box::new(ClosureAction::new(
        "install_openwebui",
        "Install Open WebUI",
        |_ctx| {
            if !crate::tools::docker::is_installed() {
                return Ok(ActionResult::failure("Docker is required for Open WebUI"));
            }
            println!("Installing Open WebUI...");
            Ok(ActionResult::success_with("Open WebUI installed"))
        },
    )));
    
    dispatcher.register(Box::new(ClosureAction::new(
        "launch_ollama",
        "Launch Ollama",
        |ctx| {
            println!("Launching Ollama...");
            let model = ctx.get_param("model").map(|s| s.as_str());
            println!("Using model: {:?}", model);
            Ok(ActionResult::success())
        },
    )));
    
    dispatcher.register(Box::new(ClosureAction::new(
        "menu_back",
        "Back",
        |_| Ok(ActionResult::success()),
    )));
    
    dispatcher
}

/// New menu_tools implementation using menu_api
/// 
/// This function replaces the original `run` function in menu_tools.rs
#[allow(dead_code)]
pub fn run_migrated(
    _i18n: &I18n,
    state: &mut WzllamaState,
    _hw: &HardwareInfo,
) -> Result<()> {
    sync_tools_state(state);
    Ok(())
}

/// Original sync_tools_state function (kept unchanged)
fn sync_tools_state(state: &mut WzllamaState) {
    use crate::tools::docker;
    use crate::core::shell;
    
    state.installed.docker = docker::is_installed();
    state.installed.ollama = shell::is_installed_quiet("ollama");
    state.installed.open_webui = shell::run_quiet(
        "docker ps -a --format \'{{.Names}}\' 2>/dev/null | grep -q open-webui"
    ).is_ok();
    state.installed.openclaw = shell::is_installed_quiet("openclaw");
    state.installed.claude_code = shell::is_installed_quiet("claude");
    state.installed.hermes_agent = shell::is_installed_quiet("hermes");
    state.installed.opencode = shell::is_installed_quiet("opencode");
    state.installed.codex = shell::is_installed_with_local_bin("codex");
    state.installed.droid = shell::is_installed_quiet("droid");
    state.installed.pi = shell::is_installed_with_local_bin("pi");
    state.installed.pool = shell::is_installed_quiet("pool");
    
    state.installed.obsidian = if shell::run("flatpak --version").is_ok() {
        shell::run_quiet("flatpak info md.obsidian.Obsidian").is_ok()
    } else {
        shell::is_installed_quiet("obsidian") || std::path::Path::new("/app/bin/obsidian").exists()
    };
}

// Migration guide:
// 
// To fully migrate menu_tools.rs:
// 
// 1. Keep the Tool trait system - it's well designed
// 2. Add menu_api integration at the selection layer
// 3. The menu_api handles navigation, Tool handles execution