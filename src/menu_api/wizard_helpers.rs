//! Wizard helpers - Extracted algorithms from wizard/*.rs for reuse in menu_api
//!
//! This module contains business logic extracted from wizard menus to enable
//! sharing between CLI and API interfaces.

use crate::config::{I18n, WzllamaState};
use crate::tools::{self};

/// Use cases for model filtering in wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseCase {
    General,
    Coding,
    Reasoning,
    Chat,
    Multimodal,
    Embedding,
}

impl UseCase {
    pub fn all() -> Vec<Self> {
        vec![
            UseCase::General,
            UseCase::Coding,
            UseCase::Reasoning,
            UseCase::Chat,
            UseCase::Multimodal,
            UseCase::Embedding,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UseCase::General => "general",
            UseCase::Coding => "coding",
            UseCase::Reasoning => "reasoning",
            UseCase::Chat => "chat",
            UseCase::Multimodal => "multimodal",
            UseCase::Embedding => "embedding",
        }
    }

    pub fn display_name(&self, i18n: &I18n) -> String {
        match self {
            UseCase::General => i18n.t("wizard.usecase.general"),
            UseCase::Coding => i18n.t("wizard.usecase.coding"),
            UseCase::Reasoning => i18n.t("wizard.usecase.reasoning"),
            UseCase::Chat => i18n.t("wizard.usecase.chat"),
            UseCase::Multimodal => i18n.t("wizard.usecase.multimodal"),
            UseCase::Embedding => i18n.t("wizard.usecase.embedding"),
        }
    }
}

/// Get priority tools for a use case (installed ones ranked by relevance)
pub fn get_priority_tools_for_usecase(use_case: UseCase, state: &WzllamaState) -> Vec<String> {
    let mut tool_ids = vec![];

    match use_case {
        UseCase::Coding => {
            if state.installed.claude_code {
                tool_ids.push("claude_code".to_string());
            }
            if state.installed.opencode {
                tool_ids.push("opencode".to_string());
            }
            if state.installed.droid {
                tool_ids.push("droid".to_string());
            }
            if state.installed.codex {
                tool_ids.push("codex".to_string());
            }
        }
        UseCase::Reasoning => {
            if state.installed.openclaw {
                tool_ids.push("openclaw".to_string());
            }
            if state.installed.hermes_agent {
                tool_ids.push("hermes_agent".to_string());
            }
        }
        UseCase::Chat => {
            if state.installed.goose {
                tool_ids.push("goose".to_string());
            }
            if state.installed.pool {
                tool_ids.push("pool".to_string());
            }
            if state.installed.pi {
                tool_ids.push("pi".to_string());
            }
        }
        UseCase::Multimodal => {
            if state.installed.openclaw {
                tool_ids.push("openclaw".to_string());
            }
            if state.installed.goose {
                tool_ids.push("goose".to_string());
            }
        }
        UseCase::General | UseCase::Embedding => {
            if state.installed.openclaw {
                tool_ids.push("openclaw".to_string());
            }
            if state.installed.goose {
                tool_ids.push("goose".to_string());
            }
        }
    }

    // Add ollama as last resort (always available)
    tool_ids.push("ollama".to_string());

    tool_ids
}

/// Scientific categories from scientific-agent-skills
pub struct ScientificCategory {
    pub name_key: &'static str,
    pub skills: &'static [&'static str],
}

impl ScientificCategory {
    pub fn all() -> Vec<Self> {
        vec![
            ScientificCategory {
                name_key: "scientific.bioinformatics",
                skills: &[
                    "biopython",
                    "bioservices",
                    "gget",
                    "scanpy",
                    "anndata",
                    "cellxgene-census",
                ],
            },
            ScientificCategory {
                name_key: "scientific.cheminformatics",
                skills: &["rdkit", "deepchem", "datamol", "diffdock"],
            },
            ScientificCategory {
                name_key: "scientific.proteomics",
                skills: &["pyzeroconf", "dhdna-profiler", "mdanalysis"],
            },
            ScientificCategory {
                name_key: "scientific.clinical",
                skills: &["clinical-decision-support", "primekg"],
            },
            ScientificCategory {
                name_key: "scientific.genomics",
                skills: &["gget", "bioservices", "aeon", "arboreto"],
            },
            ScientificCategory {
                name_key: "scientific.ml",
                skills: &[
                    "scikit-learn",
                    "pytorch-lightning",
                    "pennylane",
                    "qiskit",
                    "cirq",
                ],
            },
        ]
    }
}

/// Agentic tool info for scientific workflows
pub struct AgenticToolInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description_key: &'static str,
    pub best_for: &'static str,
}

impl AgenticToolInfo {
    pub fn all() -> Vec<Self> {
        vec![
            AgenticToolInfo {
                id: "claude_code",
                name: "Claude Code",
                description_key: "scientific.agentic.claude_code",
                best_for: "Débogage, refactoring, documentation",
            },
            AgenticToolInfo {
                id: "opencode",
                name: "OpenCode",
                description_key: "scientific.agentic.opencode",
                best_for: "Développement multi-modèles",
            },
            AgenticToolInfo {
                id: "droid",
                name: "Droid",
                description_key: "scientific.agentic.droid",
                best_for: "Workflows ML/AI complexes",
            },
            AgenticToolInfo {
                id: "codex",
                name: "Codex",
                description_key: "scientific.agentic.codex",
                best_for: "Génération de code scientifique",
            },
        ]
    }
}

/// Check if a scientific skill is installed
pub fn is_skill_installed(skill: &str) -> bool {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let skill_path = home
        .join(".wzllama")
        .join("scientific-skills")
        .join(skill)
        .join("SKILL.md");
    skill_path.exists()
}

/// Get install command for a tool
pub fn get_install_cmd(tool_id: &str) -> &'static str {
    match tool_id {
        "claude_code" => "curl -fsSL https://claude.ai/install.sh | bash",
        "opencode" => "curl -fsSL https://opencode.ai/install | bash",
        "droid" => "curl -fsSL https://app.factory.ai/cli | sh",
        "codex" => "npm install -g @openai/codex",
        _ => "See tool documentation",
    }
}

/// Sync tools state with reality (from menu_tools.rs and cleanup_tools.rs)
pub fn sync_tools_state(state: &mut WzllamaState) {
    use crate::core::shell;
    use crate::tools::docker;

    state.installed.docker = docker::is_installed();
    state.installed.ollama = shell::is_installed_quiet("ollama");
    state.installed.open_webui = shell::run_quiet("docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui || sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok();
    state.installed.openclaw = shell::is_installed_quiet("openclaw");
    state.installed.claude_code = shell::is_installed_quiet("claude");
    state.installed.hermes_agent = shell::is_installed_quiet("hermes");
    state.installed.opencode = shell::is_installed_quiet("opencode");
    state.installed.codex = shell::is_installed_with_local_bin("codex");
    state.installed.droid = shell::is_installed_quiet("droid");
    state.installed.pi = shell::is_installed_with_local_bin("pi");
    state.installed.pool = shell::is_installed_quiet("pool");

    // Obsidian - check flatpak first, then binary
    state.installed.obsidian = if shell::run("flatpak --version").is_ok() {
        shell::run_quiet("flatpak info md.obsidian.Obsidian").is_ok()
    } else {
        shell::is_installed_quiet("obsidian") || std::path::Path::new("/app/bin/obsidian").exists()
    };
}

/// Check if a tool is installed (cleanup logic)
pub fn cleanup_is_installed(id: &str) -> bool {
    use crate::core::shell;

    match id {
        "ollama" => shell::is_installed_quiet("ollama"),
        "open_webui" => shell::run_quiet("docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui || sudo docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q open-webui").is_ok(),
        "openclaw" => shell::is_installed_quiet("openclaw"),
        "claude_code" => shell::is_installed_quiet("claude"),
        "hermes_agent" => shell::is_installed_quiet("hermes"),
        "opencode" => shell::is_installed_quiet("opencode"),
        "codex" => shell::is_installed_with_local_bin("codex"),
        "droid" => shell::is_installed_quiet("droid"),
        "pi" => shell::is_installed_with_local_bin("pi"),
        "pool" => shell::is_installed_quiet("pool"),
        _ => false,
    }
}

/// Mark a tool as uninstalled in state
pub fn mark_uninstalled(id: &str, state: &mut WzllamaState) {
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

/// Cache validation helper (from menu_models.rs)
pub fn is_cache_from_today() -> bool {
    let home = dirs::home_dir().unwrap_or_default();
    let cache_path = home.join(".wzllama/cache/localmax_tree.json");
    if let Ok(metadata) = std::fs::metadata(&cache_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(age) = std::time::SystemTime::now().duration_since(modified) {
                // Cache is valid for 7 days
                return age < std::time::Duration::from_secs(7 * 24 * 3600);
            }
        }
    }
    false
}

/// Terminal screen control (from menu_main.rs and menu_models.rs)
pub fn enter_alternate_screen() {
    print!("\x1b[?1049h");
    use std::io::Write;
    std::io::stdout().flush().ok();
}

pub fn exit_alternate_screen() {
    print!("\x1b[?1049l");
    use std::io::Write;
    std::io::stdout().flush().ok();
}

/// Calculate menu indices for main menu based on resume availability
pub struct MenuIndices {
    pub has_resume: bool,
    pub wizard_idx: usize,
    pub models_idx: usize,
    pub scientific_idx: usize,
    pub tools_idx: usize,
    pub cleanup_idx: usize,
    pub config_idx: usize,
    pub language_idx: usize,
    pub quit_idx: usize,
}

impl MenuIndices {
    pub fn calculate(has_resume: bool) -> Self {
        let base_offset = has_resume as usize;
        Self {
            has_resume,
            wizard_idx: base_offset,
            models_idx: 1 + base_offset,
            scientific_idx: 2 + base_offset,
            tools_idx: 3 + base_offset,
            cleanup_idx: 4 + base_offset,
            config_idx: 5 + base_offset,
            language_idx: 6 + base_offset,
            quit_idx: 7 + base_offset,
        }
    }
}

/// Get resume label if available
pub fn get_resume_label(state: &WzllamaState, i18n: &I18n) -> Option<String> {
    if state.last_tool.is_some() && state.last_model.is_some() {
        if let Some(ref last_tool) = state.last_tool {
            if let Some(tool) = tools::get_tool(last_tool) {
                let tool_name = tool.name();
                return Some(i18n.t_with_vars(
                    "menu.main.resume",
                    &[
                        ("tool", tool_name),
                        ("model", state.last_model.as_ref().unwrap()),
                    ],
                ));
            }
        }
    }
    None
}

/// Model helpers from menu_models.rs
/// Convert an OllamaModel to a LocalMaxModel by finding matching entry in models list
/// or creating a minimal one for local-only models
pub fn ollama_to_localmax_model(
    ollama_model: &crate::core::ollama_api::OllamaModel,
    models: &[crate::core::localmax_models::LocalMaxModel],
) -> crate::core::localmax_models::LocalMaxModel {
    let ollama_name = &ollama_model.name;

    // Try to find matching model in localmaxxing database
    let matching_model = models.iter().find(|m| {
        // Check if hf_id directly matches (already ollama name)
        if m.hf_id == *ollama_name {
            return true;
        }
        // Check if ollama_name conversion matches
        let converted_name = m.to_ollama_name();
        converted_name == *ollama_name
    });

    match matching_model {
        Some(m) => m.clone(),
        None => {
            // Create a minimal LocalMaxModel for local-only models
            crate::core::localmax_models::LocalMaxModel {
                hf_id: ollama_name.clone(),
                display_name: Some(ollama_name.clone()),
                organization: "local".to_string(),
                ..Default::default()
            }
        }
    }
}

/// Language selection helpers from menu_main.rs
/// Language info structure for selection
pub struct LanguageInfo {
    pub code: &'static str,
    pub name: &'static str,
}

impl LanguageInfo {
    pub fn all() -> Vec<Self> {
        vec![
            LanguageInfo {
                code: "fr",
                name: "Français",
            },
            LanguageInfo {
                code: "en",
                name: "English",
            },
            LanguageInfo {
                code: "es",
                name: "Español",
            },
            LanguageInfo {
                code: "de",
                name: "Deutsch",
            },
            LanguageInfo {
                code: "it",
                name: "Italiano",
            },
            LanguageInfo {
                code: "pt",
                name: "Português",
            },
            LanguageInfo {
                code: "ru",
                name: "Русский",
            },
            LanguageInfo {
                code: "zh",
                name: "中文",
            },
            LanguageInfo {
                code: "ja",
                name: "日本語",
            },
            LanguageInfo {
                code: "ko",
                name: "한국어",
            },
        ]
    }
}

/// Get default language index based on system detection
pub fn get_default_language_index() -> usize {
    let languages = LanguageInfo::all();
    let system_lang = crate::config::i18n::detect_system_language();
    languages
        .iter()
        .position(|l| l.code == system_lang)
        .unwrap_or(0)
}

/// Get language info for display
pub fn get_language_items() -> Vec<(String, String)> {
    LanguageInfo::all()
        .into_iter()
        .map(|l| (l.code.to_string(), format!("{} ({})", l.name, l.code)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usecase_all() {
        let all = UseCase::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&UseCase::General));
        assert!(all.contains(&UseCase::Coding));
    }

    #[test]
    fn test_usecase_as_str() {
        assert_eq!(UseCase::General.as_str(), "general");
        assert_eq!(UseCase::Coding.as_str(), "coding");
        assert_eq!(UseCase::Reasoning.as_str(), "reasoning");
        assert_eq!(UseCase::Chat.as_str(), "chat");
    }

    #[test]
    fn test_menu_indices_without_resume() {
        let indices = MenuIndices::calculate(false);
        assert_eq!(indices.wizard_idx, 0);
        assert_eq!(indices.models_idx, 1);
        assert_eq!(indices.scientific_idx, 2);
        assert_eq!(indices.tools_idx, 3);
        assert_eq!(indices.cleanup_idx, 4);
        assert_eq!(indices.config_idx, 5);
        assert_eq!(indices.language_idx, 6);
        assert_eq!(indices.quit_idx, 7);
    }

    #[test]
    fn test_menu_indices_with_resume() {
        let indices = MenuIndices::calculate(true);
        assert_eq!(indices.wizard_idx, 1);
        assert_eq!(indices.models_idx, 2);
        assert_eq!(indices.scientific_idx, 3);
        assert_eq!(indices.tools_idx, 4);
        assert_eq!(indices.cleanup_idx, 5);
        assert_eq!(indices.config_idx, 6);
        assert_eq!(indices.language_idx, 7);
        assert_eq!(indices.quit_idx, 8);
    }

    #[test]
    fn test_get_priority_tools_coding() {
        let state = WzllamaState::default();
        let tools = get_priority_tools_for_usecase(UseCase::Coding, &state);
        // ollama is always added as last resort
        assert!(tools.contains(&"ollama".to_string()));
    }
}
