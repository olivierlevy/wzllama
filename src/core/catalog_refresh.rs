//! Fetches the Ollama integrations catalog from docs.ollama.com and updates the local cache.

use anyhow::Result;
use scraper::{Html, Selector};
use crate::tools::catalog::{CatalogEntry, ToolCatalog, ToolCategory};

const INTEGRATIONS_URL: &str = "https://docs.ollama.com/integrations";
const TOOL_PAGE_BASE: &str = "https://docs.ollama.com/integrations/";

pub struct CatalogRefresher;

impl CatalogRefresher {
    /// Spawn a background thread to refresh the catalog if the cache is stale.
    /// Returns immediately; any errors are logged, not surfaced to the caller.
    pub fn spawn_background_check() {
        let cache_fresh = crate::core::cache::read_cache("ollama_catalog", false)
            .map(|v| v.is_some())
            .unwrap_or(false);
        if cache_fresh {
            return;
        }
        std::thread::Builder::new()
            .name("catalog-refresh".into())
            .spawn(|| {
                // Use hot-swappable i18n when logging so messages reflect current language
                let mut prev_lang = crate::config::i18n::get_current().meta.code.clone();
                match Self::fetch_and_update(false) {
                    Ok(catalog) => {
                        let current_lang = crate::config::i18n::get_current().meta.code.clone();
                        if current_lang != prev_lang {
                            log::info!("Language changed to {} during catalog refresh", current_lang);
                            prev_lang = current_lang;
                        }
                        log::info!(
                            "Catalog refreshed: {} tools found (version {})",
                            catalog.tools.len(),
                            catalog.version
                        )
                    }
                    Err(e) => log::warn!("Catalog refresh failed (offline?): {}", e),
                }
            })
            .ok();
    }

    /// Force a refresh right now, display progress.
    /// Used by `wzllama catalog refresh`.
    pub fn force_refresh() -> Result<ToolCatalog> {
        println!("🔄 Fetching https://docs.ollama.com/integrations …");
        let catalog = Self::fetch_and_update(true)?;
        println!(
            "✅ Catalog updated: {} tools (version {})",
            catalog.tools.len(),
            catalog.version
        );
        Ok(catalog)
    }

    /// Fetch the integrations index, parse tools, write cache, return new catalog.
    /// `fetch_pages`: if true, fetch each tool's page to extract install_cmd.
    fn fetch_and_update(fetch_pages: bool) -> Result<ToolCatalog> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let html = client.get(INTEGRATIONS_URL).send()?.text()?;
        let entries = Self::parse_integrations_page(&html, &client, fetch_pages)?;

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let catalog = ToolCatalog {
            version: today,
            tools: entries,
        };
        catalog.save_to_cache()?;
        Ok(catalog)
    }

    /// Parse the integrations index HTML → Vec<CatalogEntry>
    fn parse_integrations_page(
        html: &str,
        client: &reqwest::blocking::Client,
        fetch_pages: bool,
    ) -> Result<Vec<CatalogEntry>> {
        let document = Html::parse_document(html);
        let body_sel = Selector::parse("body").unwrap();
        let body = document
            .select(&body_sel)
            .next()
            .ok_or_else(|| anyhow::anyhow!("No <body> element in page"))?;

        let mut entries: Vec<CatalogEntry> = Vec::new();
        let mut current_category = ToolCategory::Unknown;

        for child in body.descendants() {
            let el = match child.value().as_element() {
                Some(e) => e,
                None => continue,
            };
            match el.name() {
                "h2" => {
                    let text = scraper::ElementRef::wrap(child)
                        .map(|er| er.text().collect::<String>())
                        .unwrap_or_default();
                    current_category = Self::category_from_heading(&text);
                }
                "a" => {
                    let href = match el.attr("href") {
                        Some(h) => h,
                        None => continue,
                    };
                    if !href.starts_with("/integrations/") || href == "/integrations/" {
                        continue;
                    }
                    let page_slug = href.trim_start_matches("/integrations/");
                    if page_slug.is_empty() {
                        continue;
                    }

                    let name_node = scraper::ElementRef::wrap(child)
                        .map(|er| er.text().collect::<String>().trim().to_string())
                        .unwrap_or_else(|| page_slug.to_string());
                    if name_node.is_empty() {
                        continue;
                    }

                    let id = match page_slug {
                        "hermes" => "hermes_agent".to_string(),
                        other => other.replace('-', "_"),
                    };

                    let (launch_slug, install_cmd) = if fetch_pages {
                        Self::fetch_tool_page(client, page_slug)
                            .unwrap_or_else(|_| (page_slug.to_string(), None))
                    } else {
                        (page_slug.to_string(), None)
                    };

                    entries.push(CatalogEntry {
                        id,
                        name: name_node,
                        slug: launch_slug,
                        category: current_category.clone(),
                        install_cmd,
                        description_fallback: String::new(),
                    });
                }
                _ => {}
            }
        }

        // Merge with embedded seed to preserve description_fallback and install_cmd
        let seed = serde_json::from_str::<ToolCatalog>(ToolCatalog::SEED).unwrap_or(ToolCatalog {
            version: "seed".into(),
            tools: vec![],
        });
        for entry in &mut entries {
            if let Some(seed_entry) = seed.tools.iter().find(|s| s.id == entry.id) {
                if entry.description_fallback.is_empty() {
                    entry.description_fallback = seed_entry.description_fallback.clone();
                }
                if entry.install_cmd.is_none() && seed_entry.install_cmd.is_some() {
                    entry.install_cmd = seed_entry.install_cmd.clone();
                }
            }
        }

        if entries.is_empty() {
            anyhow::bail!("Parsing returned 0 tools — page structure may have changed");
        }
        Ok(entries)
    }

    /// Fetch a single tool page, extract ollama launch slug and install_cmd
    fn fetch_tool_page(
        client: &reqwest::blocking::Client,
        page_slug: &str,
    ) -> Result<(String, Option<String>)> {
        let url = format!("{}{}", TOOL_PAGE_BASE, page_slug);
        let html = client.get(&url).send()?.text()?;
        let document = Html::parse_document(&html);

        let code_sel = Selector::parse("code").unwrap();
        let mut launch_slug = page_slug.to_string();
        let mut install_cmd: Option<String> = None;

        for code in document.select(&code_sel) {
            let text = code.text().collect::<String>();
            let text = text.trim();

            if text.starts_with("ollama launch ") {
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() >= 3 {
                    launch_slug = parts[2].to_string();
                }
            }

            if install_cmd.is_none() && text.starts_with("npm install -g ") {
                install_cmd = Some(text.to_string());
            }
        }
        Ok((launch_slug, install_cmd))
    }

    fn category_from_heading(text: &str) -> ToolCategory {
        let t = text.to_lowercase();
        if t.contains("coding") || t.contains("agent") {
            ToolCategory::CodingAgent
        } else if t.contains("assistant") {
            ToolCategory::Assistant
        } else if t.contains("ide") || t.contains("editor") {
            ToolCategory::Ide
        } else if t.contains("chat") || t.contains("rag") {
            ToolCategory::ChatRag
        } else if t.contains("automat") {
            ToolCategory::Automation
        } else if t.contains("notebook") {
            ToolCategory::Notebook
        } else {
            ToolCategory::Unknown
        }
    }
}
