//! Updates all installed tools in background or on demand.

use anyhow::Result;
use std::path::PathBuf;
use crate::config::{I18n, WzllamaState};
use crate::tools::{get_all_tools, tool_trait::ToolStatus};
use crate::display;

const TIMESTAMP_FILE: &str = "last_update.txt";
const UPDATE_INTERVAL_HOURS: u64 = 24;

pub struct ToolUpdater;

/// Summary of an update-all run
pub struct UpdateSummary {
    pub updated: Vec<String>,
    pub failed: Vec<(String, String)>, // (tool_name, error_message)
    pub skipped: Vec<String>,          // not installed
}

impl ToolUpdater {
    /// Non-blocking: spawn background update if last update > 24h ago.
    pub fn spawn_background_check(state: WzllamaState) {
        if !Self::is_update_needed() {
            return;
        }
        std::thread::Builder::new()
            .name("tool-updater".into())
            .spawn(move || {
                // Use hot-swappable i18n during updates so messages reflect current language.
                let prev_lang = crate::config::i18n::get_current().meta.code.clone();
                match Self::update_all_silent(&state, &I18n::default()) {
                    Ok(summary) => {
                        let current_lang = crate::config::i18n::get_current().meta.code.clone();
                        if current_lang != prev_lang {
                            log::info!("Language changed to {} during background update", current_lang);
                        }
                        log::info!(
                            "Background update: {} updated, {} failed, {} skipped",
                            summary.updated.len(),
                            summary.failed.len(),
                            summary.skipped.len()
                        );
                        Self::mark_updated();
                    }
                    Err(e) => log::warn!("Background update error: {}", e),
                }
            })
            .ok();
    }

    /// Blocking: update all installed tools with progress output.
    /// Used by `wzllama update-all`.
    pub fn update_all_verbose(state: &WzllamaState, _i18n: &I18n) -> Result<UpdateSummary> {
        let tools = get_all_tools();
        let mut summary = UpdateSummary {
            updated: vec![],
            failed: vec![],
            skipped: vec![],
        };

        let mut prev_lang = crate::config::i18n::get_current().meta.code.clone();

        for tool in &tools {
            let name = tool.name().to_string();
            let is_installed = matches!(tool.status(state), ToolStatus::Installed);
            if !is_installed {
                summary.skipped.push(name);
                continue;
            }

            // Pickup latest i18n for each tool iteration
            let i18n = crate::config::i18n::get_current();
            let current_lang = i18n.meta.code.clone();
            if current_lang != prev_lang {
                display::info(&format!("✅ Language changed to {}", current_lang));
                prev_lang = current_lang.clone();
            }

            display::info(&format!("Updating {}…", tool.name()));
            match tool.update(&*i18n) {
                Ok(_) => {
                    display::success(&format!("✅ {} updated", tool.name()));
                    summary.updated.push(name);
                }
                Err(e) => {
                    display::warning(&format!("⚠️  {} update failed: {}", tool.name(), e));
                    summary.failed.push((name, e.to_string()));
                }
            }
        }
        Self::mark_updated();
        Ok(summary)
    }

    /// Silent version for background use (no stdout).
    fn update_all_silent(state: &WzllamaState, _i18n: &I18n) -> Result<UpdateSummary> {
        let tools = get_all_tools();
        let mut summary = UpdateSummary {
            updated: vec![],
            failed: vec![],
            skipped: vec![],
        };

        let mut prev_lang = crate::config::i18n::get_current().meta.code.clone();

        for tool in &tools {
            let is_installed = matches!(tool.status(state), ToolStatus::Installed);
            if !is_installed {
                summary.skipped.push(tool.name().into());
                continue;
            }

            // Use current i18n during silent updates
            let i18n = crate::config::i18n::get_current();
            let current_lang = i18n.meta.code.clone();
            if current_lang != prev_lang {
                log::info!("Language changed to {} during silent update", current_lang);
                prev_lang = current_lang.clone();
            }

            match tool.update(&*i18n) {
                Ok(_) => summary.updated.push(tool.name().into()),
                Err(e) => summary.failed.push((tool.name().into(), e.to_string())),
            }
        }
        Ok(summary)
    }

    /// Returns true if the last update was > 24h ago or never ran.
    pub fn is_update_needed() -> bool {
        let ts_file = Self::timestamp_path();
        let Ok(meta) = std::fs::metadata(&ts_file) else {
            return true;
        };
        let Ok(modified) = meta.modified() else {
            return true;
        };
        let Ok(age) = std::time::SystemTime::now().duration_since(modified) else {
            return true;
        };
        age.as_secs() > UPDATE_INTERVAL_HOURS * 3600
    }

    /// Write current timestamp to mark a successful update run.
    pub fn mark_updated() {
        let ts_file = Self::timestamp_path();
        if let Some(parent) = ts_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let now = chrono::Local::now().to_rfc3339();
        let _ = std::fs::write(&ts_file, now);
    }

    fn timestamp_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".wzllama")
            .join(TIMESTAMP_FILE)
    }
}
