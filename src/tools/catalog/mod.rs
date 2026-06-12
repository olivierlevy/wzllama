pub mod ollama_native;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static CATALOG: OnceLock<ToolCatalog> = OnceLock::new();

/// Tool category matching docs.ollama.com/integrations sections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    CodingAgent,
    Assistant,
    Ide,
    ChatRag,
    Automation,
    Notebook,
    Unknown,
}

impl ToolCategory {
    pub fn display_name(&self) -> &str {
        match self {
            ToolCategory::CodingAgent => "Coding Agents",
            ToolCategory::Assistant => "Assistants",
            ToolCategory::Ide => "IDEs & Editors",
            ToolCategory::ChatRag => "Chat & RAG",
            ToolCategory::Automation => "Automation",
            ToolCategory::Notebook => "Notebooks",
            ToolCategory::Unknown => "Other",
        }
    }
}

/// Single tool entry from the Ollama integrations catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Unique wzllama ID (e.g. "cline", "hermes-desktop")
    pub id: String,
    /// Display name (e.g. "Cline CLI")
    pub name: String,
    /// Slug used in `ollama launch <slug>` (e.g. "cline")
    pub slug: String,
    pub category: ToolCategory,
    /// Optional explicit install command (e.g. "npm install -g cline").
    /// None means `ollama launch <slug>` handles installation.
    pub install_cmd: Option<String>,
    /// English fallback description (used when i18n key is absent)
    pub description_fallback: String,
}

/// Full catalog: embedded seed + optional HTTP-refreshed cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCatalog {
    pub version: String,
    pub tools: Vec<CatalogEntry>,
}

impl ToolCatalog {
    /// Embedded seed catalog (compiled into binary)
    pub(crate) const SEED: &'static str = include_str!("catalog.json");

    /// Load catalog: prefer fresh 24h cache, fall back to embedded seed.
    /// Result is cached in process memory after first call.
    pub fn load() -> &'static ToolCatalog {
        CATALOG.get_or_init(Self::load_inner)
    }

    fn load_inner() -> ToolCatalog {
        if let Ok(Some(cached)) = crate::core::cache::read_cache("ollama_catalog", false) {
            if let Ok(catalog) = serde_json::from_str::<ToolCatalog>(&cached) {
                return catalog;
            }
        }

        serde_json::from_str(Self::SEED)
            .expect("catalog.json must be valid JSON — this is a compile-time resource")
    }

    /// Save this catalog to the 24h cache file
    pub fn save_to_cache(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        crate::core::cache::write_cache("ollama_catalog", &json)
    }

    /// Returns only the entries whose IDs are NOT already in `existing_ids`
    pub fn new_entries<'a>(&'a self, existing_ids: &[&str]) -> Vec<&'a CatalogEntry> {
        self.tools
            .iter()
            .filter(|e| !existing_ids.contains(&e.id.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{ToolCatalog, ToolCategory};

    #[test]
    fn seed_catalog_deserializes() {
        let catalog: ToolCatalog = serde_json::from_str(ToolCatalog::SEED).unwrap();

        assert_eq!(catalog.version, "2026-06-12");
        assert!(catalog.tools.iter().any(|tool| tool.id == "claude_code"));
        assert!(catalog
            .tools
            .iter()
            .any(|tool| tool.category == ToolCategory::Ide));
    }

    #[test]
    fn new_entries_filters_existing_ids() {
        let catalog: ToolCatalog = serde_json::from_str(ToolCatalog::SEED).unwrap();

        let new_entries = catalog.new_entries(&["claude_code", "codex", "marimo"]);

        assert!(!new_entries.iter().any(|entry| entry.id == "claude_code"));
        assert!(!new_entries.iter().any(|entry| entry.id == "codex"));
        assert!(!new_entries.iter().any(|entry| entry.id == "marimo"));
        assert!(new_entries.iter().any(|entry| entry.id == "copilot_cli"));
    }
}
