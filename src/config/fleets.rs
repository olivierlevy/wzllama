#![allow(dead_code)]

use std::collections::HashMap;
use crate::config::paths;
use crate::config::state::{FleetState, WzllamaState, save};

pub fn detect_openclaw_fleets() -> HashMap<String, FleetState> {
    let home = paths::home();
    let mut fleets = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(&home) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let dirname = match path.file_name().and_then(|n| n.to_str()) {
                Some(d) if d.starts_with(".openclaw-") => d,
                _ => continue,
            };
            let profile = dirname.strip_prefix(".openclaw-").unwrap_or(dirname);
            let config_path = path.join("openclaw.json");
            if !config_path.exists() { continue; }

            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                    let orchestrator = config["agents"]["defaults"]["model"]["primary"]
                        .as_str().map(|s| s.strip_prefix("ollama/").unwrap_or(s).to_string())
                        .unwrap_or_default();
                    let agents: Vec<String> = config["agents"]["list"].as_array()
                        .map(|arr| arr.iter()
                            .filter_map(|a| a["model"]["primary"].as_str()
                                .map(|s| s.strip_prefix("ollama/").unwrap_or(s).to_string()))
                            .collect())
                        .unwrap_or_default();
                    let installed = std::process::Command::new("systemctl")
                        .args(["--user", "is-enabled", &format!("openclaw-gateway-{}.service", profile)])
                        .output().map(|o| o.status.success()).unwrap_or(false);

                    fleets.insert(profile.to_string(), FleetState {
                        profile: profile.to_string(), orchestrator, agents, openclaw_installed: installed,
                    });
                }
            }
        }
    }
    fleets
}

#[allow(dead_code)]
pub fn sync(state: &mut WzllamaState) {
    state.fleets = detect_openclaw_fleets();
    let _ = save(state);
}

pub fn delete_fleet(profile: &str, state: &mut WzllamaState) -> anyhow::Result<()> {
    let service = format!("openclaw-gateway-{}.service", profile);
    let _ = std::process::Command::new("systemctl").args(["--user", "stop", &service]).output();
    let _ = std::process::Command::new("systemctl").args(["--user", "disable", &service]).output();
    let dir = paths::home().join(format!(".openclaw-{}", profile));
    if dir.exists() { std::fs::remove_dir_all(&dir)?; }
    state.fleets.remove(profile);
    if state.last_fleet.as_deref() == Some(profile) { state.last_fleet = None; }
    save(state)?;
    Ok(())
}