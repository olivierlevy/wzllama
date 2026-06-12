//! API Service - Business logic for HTTP API endpoints
//!
//! Centralizes all menu-related business logic that was previously scattered
//! across api_server.rs and wizard modules.

use crate::config::{I18n, WzllamaState};
use crate::core::hardware;
use crate::tools::{self, tool_trait::ToolStatus};
use serde_json::Value;

/// Hardware information for API
#[derive(serde::Serialize)]
pub struct HardwareInfo {
    pub ram_gb: f64,
    pub has_gpu: bool,
    pub gpus: Vec<GpuInfo>,
}

#[derive(serde::Serialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
}

/// System status
#[derive(serde::Serialize)]
pub struct SystemStatus {
    pub status: String,
    pub ollama: String,
}

/// Tool information for API responses
#[derive(serde::Serialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub status: String,
    pub supports_agentic: bool,
    pub requires_docker: bool,
}

/// Action response
#[derive(serde::Serialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

/// API Service - centralized business logic
pub struct ApiService;

impl ApiService {
    /// Get the current state
    pub fn get_state() -> WzllamaState {
        WzllamaState::load()
    }

    /// Get i18n for current state
    pub fn get_i18n(lang: Option<&str>) -> std::sync::Arc<crate::config::i18n::I18n> {
        if let Some(lang) = lang {
            // Load requested language explicitly (not affecting global store)
            match crate::config::i18n::load(lang) {
                Ok(i) => std::sync::Arc::new(i),
                Err(_) => crate::config::i18n::get_current(),
            }
        } else {
            crate::config::i18n::get_current()
        }
    }

    /// Get menu structure using menu_api
    pub fn get_menu_structure(i18n: &I18n, state: &WzllamaState) -> Value {
        crate::menu_api::get_menu_structure(i18n, state)
    }

    /// Get tools menu using menu_api
    pub fn get_tools_menu(i18n: &I18n, state: &WzllamaState) -> Value {
        crate::menu_api::get_tools_menu(i18n, state)
    }

    /// Get models menu using menu_api
    pub fn get_models_menu(i18n: &I18n, state: &WzllamaState) -> Value {
        crate::menu_api::get_models_menu(i18n, state)
    }

    /// List all tools with their status
    pub fn list_tools(state: &WzllamaState, i18n: &I18n) -> Vec<ToolInfo> {
        let tools_list = tools::get_available_tools(state, i18n);

        tools_list
            .into_iter()
            .map(|t| {
                let tool_dyn = tools::get_tool(&t.id);
                ToolInfo {
                    id: t.id,
                    name: t.name,
                    description: t.description,
                    installed: t.installed,
                    status: if t.installed {
                        "installed".to_string()
                    } else {
                        "not_installed".to_string()
                    },
                    supports_agentic: tool_dyn
                        .as_ref()
                        .map(|x| x.supports_agentic())
                        .unwrap_or(false),
                    requires_docker: tool_dyn
                        .as_ref()
                        .map(|x| x.requires_docker())
                        .unwrap_or(false),
                }
            })
            .collect()
    }

    /// Get a specific tool by ID
    pub fn get_tool(id: &str, i18n: &I18n, state: &WzllamaState) -> Option<ToolInfo> {
        tools::get_tool(id).map(|tool| {
            let installed = Self::is_tool_installed(id, state);
            ToolInfo {
                id: id.to_string(),
                name: tool.name().to_string(),
                description: tool.description(i18n),
                installed,
                status: if installed {
                    "installed".to_string()
                } else {
                    "not_installed".to_string()
                },
                supports_agentic: tool.supports_agentic(),
                requires_docker: tool.requires_docker(),
            }
        })
    }

    /// Check if a tool is installed
    pub fn is_tool_installed(id: &str, state: &WzllamaState) -> bool {
        match id {
            "docker" => state.installed.docker,
            "ollama" => state.installed.ollama,
            "open_webui" => state.installed.open_webui,
            "openclaw" => state.installed.openclaw,
            "claude_code" => state.installed.claude_code,
            "hermes_agent" => state.installed.hermes_agent,
            "opencode" => state.installed.opencode,
            "codex" => state.installed.codex,
            "copilot_cli" => state.installed.copilot_cli,
            "droid" => state.installed.droid,
            "pi" => state.installed.pi,
            "pool" => state.installed.pool,
            "obsidian" => state.installed.obsidian,
            "goose" => state.installed.goose,
            "llmfit" => state.installed.llmfit,
            _ => false,
        }
    }

    /// Install a tool
    pub fn install_tool(id: &str, i18n: &I18n) -> Result<ActionResponse, anyhow::Error> {
        if let Some(tool) = tools::get_tool(id) {
            tool.install(i18n)?;
            Ok(ActionResponse {
                success: true,
                message: format!("{} installed successfully", tool.name()),
            })
        } else {
            Ok(ActionResponse {
                success: false,
                message: format!("Tool '{}' not found", id),
            })
        }
    }

    /// Update a tool
    pub fn update_tool(id: &str, i18n: &I18n) -> Result<ActionResponse, anyhow::Error> {
        if let Some(tool) = tools::get_tool(id) {
            tool.update(i18n)?;
            Ok(ActionResponse {
                success: true,
                message: format!("{} updated successfully", tool.name()),
            })
        } else {
            Ok(ActionResponse {
                success: false,
                message: format!("Tool '{}' not found", id),
            })
        }
    }

    /// Uninstall a tool
    pub fn uninstall_tool(id: &str, i18n: &I18n) -> Result<ActionResponse, anyhow::Error> {
        if let Some(tool) = tools::get_tool(id) {
            tool.uninstall(i18n)?;
            Ok(ActionResponse {
                success: true,
                message: format!("{} uninstalled successfully", tool.name()),
            })
        } else {
            Ok(ActionResponse {
                success: false,
                message: format!("Tool '{}' not found", id),
            })
        }
    }

    /// Get system status
    pub fn get_system_status() -> SystemStatus {
        let ollama_running = tools::ollama::OllamaTool::is_running();
        SystemStatus {
            status: "running".to_string(),
            ollama: if ollama_running {
                "connected".to_string()
            } else {
                "disconnected".to_string()
            },
        }
    }

    /// Get hardware information
    pub fn get_hardware_info() -> HardwareInfo {
        let hw = hardware::detect();
        HardwareInfo {
            ram_gb: hw.ram_gb,
            has_gpu: hw.has_gpu(),
            gpus: hw
                .gpus
                .iter()
                .map(|g| GpuInfo {
                    name: g.name.clone(),
                    vram_mb: g.vram_mb,
                })
                .collect(),
        }
    }

    /// Sync tools state
    pub fn sync_tools_state(state: &mut WzllamaState) {
        crate::menu_api::wizard_helpers::sync_tools_state(state);
    }
}

/// Re-export wizard_helpers for API use
pub use crate::menu_api::wizard_helpers::{
    get_default_language_index, get_install_cmd, get_language_items,
    get_priority_tools_for_usecase, get_resume_label, is_cache_from_today, is_skill_installed,
    AgenticToolInfo, LanguageInfo, MenuIndices, ScientificCategory, UseCase,
};
