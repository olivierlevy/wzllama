#![allow(dead_code)]

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const CACHE_DIR: &str = ".wzllama/cache";
const CACHE_TTL_HOURS: u64 = 24;

/// Get the cache directory path
fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let cache_path = home.join(CACHE_DIR);
    fs::create_dir_all(&cache_path)?;
    Ok(cache_path)
}

/// Check if a file was modified today
fn is_from_today(path: &PathBuf) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(age) = SystemTime::now().duration_since(modified) {
                // Check if modified within the last 24 hours
                return age < Duration::from_secs(24 * 3600);
            }
        }
    }
    false
}

/// Read cached JSON data from file (checks if from today for daily cache)
pub fn read_cache(key: &str, daily: bool) -> Result<Option<String>> {
    let cache_path = cache_dir()?.join(format!("{}.json", key));

    if !cache_path.exists() {
        return Ok(None);
    }

    // For daily cache, check if file is from today
    if daily && !is_from_today(&cache_path) {
        let _ = fs::remove_file(&cache_path);
        return Ok(None);
    }

    // Check if cache is expired (non-daily)
    if !daily {
        let metadata = fs::metadata(&cache_path)?;
        let modified = metadata.modified()?;
        let age = SystemTime::now().duration_since(modified)?;

        if age > Duration::from_secs(CACHE_TTL_HOURS * 3600) {
            let _ = fs::remove_file(&cache_path);
            return Ok(None);
        }
    }

    let content = fs::read_to_string(&cache_path)?;
    Ok(Some(content))
}

/// Write JSON data to cache with a timestamp for the filename
pub fn write_cache(key: &str, data: &str) -> Result<()> {
    let cache_path = cache_dir()?.join(format!("{}.json", key));
    fs::write(&cache_path, data)?;
    Ok(())
}

/// Clear all cache files
pub fn clear_cache() -> Result<()> {
    let cache = cache_dir()?;
    if cache.exists() {
        for entry in fs::read_dir(&cache)? {
            let entry = entry?;
            if entry
                .path()
                .extension()
                .map(|e| e == "json")
                .unwrap_or(false)
            {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}

/// Update daily models cache - fetches tree and search results, replaces old if successful
pub fn update_daily_models_cache() -> Result<()> {
    use reqwest::blocking::Client;

    let cache = cache_dir()?;
    let client = Client::new();

    // Fetch tree models
    let tree_url = "https://localmaxxing.com/api/models?tree=true";
    let tree_response = client
        .get(tree_url)
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .map_err(|e| anyhow::anyhow!("Network error for tree: {}", e))?;

    if !tree_response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch tree models: status {}",
            tree_response.status()
        );
    }

    let tree_data = tree_response.text()?;

    // Fetch search models for code
    let search_url = "https://localmaxxing.com/api/models?tree=true&search=code";
    let search_response = client
        .get(search_url)
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .map_err(|e| anyhow::anyhow!("Network error for search: {}", e))?;

    if !search_response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch search models: status {}",
            search_response.status()
        );
    }

    let search_data = search_response.text()?;

    // All succeeded - now replace old files atomically using temp files
    let tree_path = cache.join("localmax_tree.json");
    let search_path = cache.join("localmax_search_code.json");
    let tree_temp = cache.join("localmax_tree.json.tmp");
    let search_temp = cache.join("localmax_search_code.json.tmp");

    // Write to temp files first
    fs::write(&tree_temp, &tree_data)?;
    fs::write(&search_temp, &search_data)?;

    // Atomically replace old files
    let _ = fs::remove_file(&tree_path);
    let _ = fs::remove_file(&search_path);
    fs::rename(&tree_temp, &tree_path)?;
    fs::rename(&search_temp, &search_path)?;

    Ok(())
}
