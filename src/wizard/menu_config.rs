//! Config menu - migrated to use menu_api
//!
//! This menu uses the MenuTree/MenuHandler system with ToolAction dispatch.

use anyhow::Result;
use colored::*;
use dialoguer::{Select, Confirm, Input};
use crate::config::{self, I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::display;
use crate::menu_api::{MenuTree, MenuItem, MenuMetadata};
use super::menu_header;

/// Create the config menu tree structure
pub fn build_menu_tree() -> MenuTree {
    let root = MenuItem::branch("config")
        .add_submenu(MenuItem::leaf("↩️ Retour"))
        .add_submenu(MenuItem::leaf("⚙️ Performance").with_action("edit_performance"))
        .add_submenu(MenuItem::leaf("⚙️ Ollama Settings").with_action("edit_ollama_settings"))
        .add_submenu(MenuItem::leaf("🔌 Providers").with_action("edit_providers"))
        .add_submenu(MenuItem::leaf("🔓 Openclaw").with_action("edit_openclaw"))
        .add_submenu(MenuItem::leaf("🐚 Shells").with_action("manage_shells"))
        .add_submenu(MenuItem::leaf("🔄 Regenerate env file").with_action("regenerate_env"))
        .add_submenu(MenuItem::leaf("🗑️ Uninstall wzllama").with_action("uninstall_wzllama"));
    
    MenuTree::new("config")
        .with_metadata(MenuMetadata {
            title: Some("⚙️ Configuration".to_string()),
            ..Default::default()
        })
        .with_root(root)
}

pub fn run(i18n: &I18n, _state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    let mut config = config::env::EnvConfig::load();
    
    loop {
        // Affiche le header avec ressources comme le menu principal
        menu_header::render(
            i18n,
            "menu.main.config",
            true,
            _state.last_model.as_deref(),
            hw.ram_gb,
            hw.total_vram_mb as f64 / 1024.0
        );
        
        // Build menu items from MenuTree for consistent display
        // Retour en premier item (selon TODO.md ligne 72)
        let items = vec![
            i18n.t("menu.back"),
            i18n.t("config.performance"),
            i18n.t("config.ollama_settings"),
            i18n.t("config.providers"),
            i18n.t("config.openclaw"),
            i18n.t("config.shells"),
            i18n.t("config.regenerate_env"),
            i18n.t("config.uninstall_wzllama"),
        ];

        let sel = match Select::new()
            .with_prompt(i18n.t("config.choose"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact_opt()? {
            Some(s) => s,
            None => return Ok(()), // Escape pressed
        };

        match sel {
            0 => return Ok(()),  // Retour en position 0
            1 => edit_performance(i18n, &mut config)?,
            2 => edit_ollama_settings(i18n, &mut config)?,
            3 => edit_providers(i18n, &mut config)?,
            4 => edit_openclaw(i18n, &mut config)?,
            5 => manage_shells(i18n)?,
            6 => {
                config.generate_env_file()?;
                display::success(&i18n.t("config.env_regenerated"));
            }
            7 => uninstall_wzllama(i18n)?,
            _ => return Ok(()),
        }
        config.save()?;
    }
}

fn edit_performance(i18n: &I18n, config: &mut config::env::EnvConfig) -> Result<()> {
    loop {
        // Retour en premier item (selon TODO.md ligne 72)
        let items = vec![
            i18n.t("menu.back").to_string(),
            format!("📐 {} tokens", config.ollama.context_length),
            format!("💾 Cache KV: {}", config.ollama.kv_cache_type),
            format!("⚡ Flash Attention: {}", if config.ollama.flash_attention { "✅" } else { "❌" }),
            format!("☁️  Cloud: {}", if config.ollama.no_cloud { "❌ Bloqué" } else { "✅ Autorisé" }),
        ];

        let sel = match Select::new()
            .with_prompt(i18n.t("config.perf_choose"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact_opt()? {
            Some(s) => s,
            None => return Ok(()), // Escape pressed
        };

        match sel {
            0 => return Ok(()),  // Retour en position 0
            1 => {
                let options = [("4K", "4096"), ("8K", "8192"), ("16K", "16384"), ("32K", "32768"), ("64K", "65536")];
                let labels: Vec<&str> = options.iter().map(|(l, _)| *l).collect();
                let s = match Select::new().with_prompt("Context").items(&labels).default(2).max_length(10).interact_opt()? {
                    Some(v) => v,
                    None => continue, // Escape - redo menu
                };
                config.ollama.context_length = options[s].1.parse().unwrap_or(16384);
            }
            2 => {
                let options = [("f16", "f16"), ("q8_0", "q8_0"), ("q4_0", "q4_0")];
                let labels: Vec<&str> = options.iter().map(|(l, _)| *l).collect();
                let s = match Select::new().with_prompt("KV Cache").items(&labels).default(1).max_length(10).interact_opt()? {
                    Some(v) => v,
                    None => continue, // Escape - redo menu
                };
                config.ollama.kv_cache_type = options[s].1.to_string();
            }
            3 => config.ollama.flash_attention = !config.ollama.flash_attention,
            4 => config.ollama.no_cloud = !config.ollama.no_cloud,
            _ => return Ok(()),
        }
    }
}

fn edit_ollama_settings(i18n: &I18n, config: &mut config::env::EnvConfig) -> Result<()> {
    loop {
        // Retour en premier item (selon TODO.md ligne 72)
        let items = vec![
            i18n.t("menu.back").to_string(),
            format!("📍 Host: {}", config.ollama.host),
            format!("🌐 Origins: {}", config.ollama.origins),
            format!("⏱️  Keep alive: {}s", config.ollama.keep_alive),
            format!("🔀 Parallel: {}", config.ollama.num_parallel),
            format!("📦 Max loaded: {}", config.ollama.max_loaded_models),
            format!("💾 Max VRAM: {} MB", if config.ollama.max_vram > 0 { config.ollama.max_vram.to_string() } else { "auto".to_string() }),
            format!("🎮 CUDA: {}", if config.ollama.cuda_visible_devices.is_empty() { "all" } else { &config.ollama.cuda_visible_devices }),
        ];

        let sel = match Select::new()
            .with_prompt(i18n.t("config.ollama_settings"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact_opt()? {
            Some(s) => s,
            None => return Ok(()),
        };

        match sel {
            0 => return Ok(()),  // Retour en position 0
            1 => {
                let new: String = Input::new()
                    .with_prompt(i18n.t("config.ollama.host"))
                    .default(config.ollama.host.clone())
                    .interact()?;
                config.ollama.host = new;
            }
            2 => {
                let new: String = Input::new()
                    .with_prompt(i18n.t("config.ollama.origins"))
                    .default(config.ollama.origins.clone())
                    .interact()?;
                config.ollama.origins = new;
            }
            3 => {
                let new: i32 = Input::new()
                    .with_prompt(i18n.t("config.ollama.keep_alive"))
                    .default(config.ollama.keep_alive)
                    .interact()?;
                config.ollama.keep_alive = new;
            }
            4 => {
                let new: u32 = Input::new()
                    .with_prompt(i18n.t("config.ollama.num_parallel"))
                    .default(config.ollama.num_parallel)
                    .interact()?;
                config.ollama.num_parallel = new;
            }
            5 => {
                let new: u32 = Input::new()
                    .with_prompt(i18n.t("config.ollama.max_loaded"))
                    .default(config.ollama.max_loaded_models)
                    .interact()?;
                config.ollama.max_loaded_models = new;
            }
            6 => {
                let new: u64 = Input::new()
                    .with_prompt(i18n.t("config.ollama.max_vram"))
                    .default(config.ollama.max_vram)
                    .interact()?;
                config.ollama.max_vram = new;
            }
            7 => {
                let new: String = Input::new()
                    .with_prompt(i18n.t("config.ollama.cuda_devices"))
                    .default(config.ollama.cuda_visible_devices.clone())
                    .interact()?;
                config.ollama.cuda_visible_devices = new;
            }
            _ => return Ok(()),
        }
    }
}

fn edit_providers(i18n: &I18n, config: &mut config::env::EnvConfig) -> Result<()> {
    loop {
        // Retour en premier item (selon TODO.md ligne 72)
        let items = vec![
            i18n.t("menu.back").to_string(),
            format!("🔓 OpenAI: {}", if config.providers.openai.api_key.is_empty() || config.providers.openai.api_key == "ollama" { "not set" } else { "***" }),
            format!("🔓 Anthropic: {}", if config.providers.anthropic.api_key.is_empty() || config.providers.anthropic.api_key == "ollama" { "not set" } else { "***" }),
            format!("🔗 OpenAI URL: {}", config.providers.openai.base_url),
            format!("🔗 Anthropic URL: {}", config.providers.anthropic.base_url),
        ];

        let sel = match Select::new()
            .with_prompt(i18n.t("config.providers"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact_opt()? {
            Some(s) => s,
            None => return Ok(()),
        };

        match sel {
            0 => return Ok(()),  // Retour en position 0
            1 => {
                let new: String = Input::new()
                    .with_prompt(i18n.t("config.provider.openai"))
                    .default(config.providers.openai.api_key.clone())
                    .interact()?;
                config.providers.openai.api_key = new;
            }
            2 => {
                let new: String = Input::new()
                    .with_prompt(i18n.t("config.provider.anthropic"))
                    .default(config.providers.anthropic.api_key.clone())
                    .interact()?;
                config.providers.anthropic.api_key = new;
            }
            3 => {
                let new: String = Input::new()
                    .with_prompt(i18n.t("config.provider.openai_url"))
                    .default(config.providers.openai.base_url.clone())
                    .interact()?;
                config.providers.openai.base_url = new;
            }
            4 => {
                let new: String = Input::new()
                    .with_prompt(i18n.t("config.provider.anthropic_url"))
                    .default(config.providers.anthropic.base_url.clone())
                    .interact()?;
                config.providers.anthropic.base_url = new;
            }
            _ => return Ok(()),
        }
    }
}

fn edit_openclaw(i18n: &I18n, config: &mut config::env::EnvConfig) -> Result<()> {
    loop {
        // Retour en premier item (selon TODO.md ligne 72)
        let items = vec![
            i18n.t("menu.back").to_string(),
            format!("🔑 API Key: {}", if config.openclaw.api_key.is_empty() || config.openclaw.api_key == "ollama-local" { "default (ollama-local)" } else { "***" }),
        ];

        let sel = match Select::new()
            .with_prompt(i18n.t("config.openclaw"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact_opt()? {
            Some(s) => s,
            None => return Ok(()),
        };

        match sel {
            0 => return Ok(()),  // Retour en position 0
            1 => {
                let new: String = Input::new()
                    .with_prompt(i18n.t("config.openclaw.api_key"))
                    .default(config.openclaw.api_key.clone())
                    .interact()?;
                config.openclaw.api_key = new;
            }
            _ => return Ok(()),
        }
    }
}

fn manage_shells(i18n: &I18n) -> Result<()> {
    let statuses = config::shells::get_shells_status(i18n);
    display::section(&i18n.t("config.shells"));
    for s in &statuses { println!("   {}", s); }
    
    // Retour en premier item (selon TODO.md ligne 72)
    let items = vec![
        i18n.t("menu.back"),
        i18n.t("config.shells_install_all"),
        i18n.t("config.shells_uninstall_all"),
    ];

    let sel = Select::new()
        .with_prompt(i18n.t("config.shells_choose"))
        .items(&items)
        .default(0)
        .max_length(15)
        .interact()?;

    match sel {
        0 => return Ok(()),  // Retour en position 0
        1 => config::shells::install_all_shells(i18n)?,
        2 => config::shells::uninstall_all_shells(i18n)?,
        _ => {}
    }
    Ok(())
}

fn uninstall_wzllama(i18n: &I18n) -> Result<()> {
    display::warning(&i18n.t("config.uninstall_warning"));
    
    if !Confirm::new().with_prompt(i18n.t("config.uninstall_confirm")).default(false).interact()? {
        return Ok(());
    }

    let confirm2 = Confirm::new()
        .with_prompt(i18n.t("config.uninstall_final"))
        .default(false)
        .interact()?;
    
    if !confirm2 { return Ok(()); }

    display::section(&i18n.t("config.uninstalling"));

    // 1. Shells
    config::shells::uninstall_all_shells(i18n)?;

    // 2. Modèles wzllama
    let models = crate::core::ollama_api::list_wzllama_models();
    for m in &models { let _ = crate::core::ollama_api::delete_model(m); }
    display::success(&i18n.t("config.uninstall_models_removed"));

    // 3. Dossiers
    let _ = std::fs::remove_dir_all(config::paths::wzllama_dir());
    display::success(&i18n.t("config.uninstall_config_removed"));

    // 4. Binaire
    let _ = crate::core::shell::run("sudo rm -f /usr/local/bin/wzllama");
    display::success(&i18n.t("config.uninstall_binary_removed"));

    println!("\n{}", i18n.t("config.uninstall_bye").green().bold());
    std::process::exit(0);
}

pub fn uninstall_wzllama_cli() -> Result<()> {
    println!("⚠️  This will remove wzllama, all its models and configuration.");
    println!("   Type 'YES' to confirm:");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim() != "YES" {
        println!("Cancelled.");
        return Ok(());
    }

    println!("🗑️  Uninstalling...");

    // Shells
    config::shells::uninstall_all_shells_cli()?;

    // Models
    let models = crate::core::ollama_api::list_wzllama_models();
    for m in &models { let _ = crate::core::ollama_api::delete_model(m); }
    println!("   ✅ {} models removed", models.len());

    // Config
    let _ = std::fs::remove_dir_all(config::paths::wzllama_dir());
    println!("   ✅ Configuration removed");

    // Binary
    let _ = crate::core::shell::run("sudo rm -f /usr/local/bin/wzllama");
    println!("   ✅ Binary removed");

    println!("\n👋 wzllama has been uninstalled. Goodbye!");
    std::process::exit(0);
}
