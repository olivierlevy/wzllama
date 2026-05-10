use anyhow::Result;
use colored::*;
use dialoguer::{Select, Confirm, Input};
use crate::config::{self, I18n, WzllamaState};
use crate::display;

pub fn run(i18n: &I18n, _state: &mut WzllamaState) -> Result<()> {
    let mut config = config::env::EnvConfig::load();

    // Vérifier les modèles configurés
    check_configured_models(i18n, &config)?;

    loop {
        display::header(&i18n.t("menu.main.config"));
        
        // Afficher résumé
        display_config_summary(i18n, &config);
        
        let items = vec![
            i18n.t("config.models"),
            i18n.t("config.performance"),
            i18n.t("config.shells"),
            i18n.t("config.regenerate_env"),
            i18n.t("config.uninstall_wzllama"),
            i18n.t("menu.back"),
        ];

        let sel = Select::new()
            .with_prompt(i18n.t("config.choose"))
            .items(&items)
            .default(0)
            .interact()?;

        match sel {
            0 => edit_models(i18n, &mut config)?,
            1 => edit_performance(i18n, &mut config)?,
            2 => manage_shells(i18n)?,
            3 => {
                config.generate_env_file()?;
                display::success(&i18n.t("config.env_regenerated"));
            }
            4 => uninstall_wzllama(i18n)?,
            _ => return Ok(()),
        }
        config.save()?;
    }
}


fn check_configured_models(i18n: &I18n, config: &config::env::EnvConfig) -> Result<()> {
    let local_models = match crate::core::ollama_api::detect_url() {
        Some(url) => crate::core::ollama_api::fetch_local_models(&url).unwrap_or_default(),
        None => return Ok(()),
    };
    
    let local_names: Vec<&str> = local_models.iter().map(|m| m.name.as_str()).collect();
    
    let configured = [
        ("code", &config.models.code),
        ("book", &config.models.book),
        ("agent", &config.models.agent),
        ("chat", &config.models.chat),
    ];
    
    for (usage, model) in &configured {
        if !local_names.contains(&model.as_str()) {
            display::warning(&i18n.t_with_vars("config.model_not_found", &[
                ("usage", usage),
                ("model", model),
            ]));
            if Confirm::new()
                .with_prompt(i18n.t_with_vars("config.model_download", &[("model", model)]))
                .default(true)
                .interact()?
            {
                crate::core::ollama_api::pull_model(model)?;
            }
        }
    }
    
    Ok(())
}

fn display_config_summary(i18n: &I18n, config: &config::env::EnvConfig) {
    println!();
    println!("   {}", i18n.t_with_vars("config.summary.ollama", &[
        ("host", &config.ollama.host),
        ("keep", &config.ollama.keep_alive.to_string()),
        ("cloud", if config.ollama.no_cloud { "❌" } else { "✅" }),
        ("ctx", &config.ollama.context_length.to_string()),
    ]));
    println!("   {}", i18n.t_with_vars("config.summary.models", &[
        ("code", &config.models.code),
        ("book", &config.models.book),
        ("agent", &config.models.agent),
        ("chat", &config.models.chat),
    ]));
    println!();
}

fn edit_models(i18n: &I18n, config: &mut config::env::EnvConfig) -> Result<()> {
    let fields = vec![
        ("code", &mut config.models.code),
        ("book", &mut config.models.book),
        ("agent", &mut config.models.agent),
        ("chat", &mut config.models.chat),
    ];
    
    for (label, field) in fields {
        let new: String = Input::new()
            .with_prompt(i18n.t_with_vars("config.edit_model", &[("usage", label)]))
            .default(field.clone())
            .interact()?;
        *field = new;
    }
    Ok(())
}

fn edit_performance(i18n: &I18n, config: &mut config::env::EnvConfig) -> Result<()> {
    loop {
        let items = vec![
            i18n.t_with_vars("config.context", &[("ctx", &config.ollama.context_length.to_string())]),
            i18n.t_with_vars("config.kv_cache", &[("kv", &config.ollama.kv_cache_type)]),
            i18n.t_with_vars("config.flash_attn", &[("status", if config.ollama.flash_attention { "✅" } else { "❌" })]),
            i18n.t_with_vars("config.cloud", &[("status", if config.ollama.no_cloud { "❌ BLOQUÉS" } else { "✅ AUTORISÉS" })]),
            i18n.t("menu.back"),
        ];

        let sel = Select::new()
            .with_prompt(i18n.t("config.perf_choose"))
            .items(&items)
            .default(0)
            .interact()?;

        match sel {
            0 => {
                let options = vec!["4096", "8192", "16384", "32768", "65536"];
                let s = Select::new().with_prompt("Contexte").items(&options).default(2).interact()?;
                config.ollama.context_length = options[s].parse().unwrap_or(16384);
            }
            1 => {
                let options = vec!["f16", "q8_0", "q4_0"];
                let s = Select::new().with_prompt("Cache KV").items(&options).default(1).interact()?;
                config.ollama.kv_cache_type = options[s].to_string();
            }
            2 => config.ollama.flash_attention = !config.ollama.flash_attention,
            3 => config.ollama.no_cloud = !config.ollama.no_cloud,
            _ => return Ok(()),
        }
    }
}

fn manage_shells(i18n: &I18n) -> Result<()> {
    let statuses = config::shells::get_shells_status(i18n);
    println!("\n   📂 Shells :");
    for s in &statuses { println!("   {}", s); }
    
    let items = vec![
        i18n.t("config.shells_install_all"),
        i18n.t("config.shells_uninstall_all"),
        i18n.t("menu.back"),
    ];

    let sel = Select::new()
        .with_prompt(i18n.t("config.shells_choose"))
        .items(&items)
        .default(0)
        .interact()?;

    match sel {
        0 => config::shells::install_all_shells(i18n)?,
        1 => config::shells::uninstall_all_shells(i18n)?,
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
