use crate::config::{self, I18n, UsageSpec, get_available_languages, detect_system_language};
use crate::core::{self, detect_hardware, HardwareInfo};
use crate::installers::Installer;
use anyhow::Result;
use colored::*;
use dialoguer::{Select, Input};

pub fn run_wizard() -> Result<()> {
    println!("{}", "═".repeat(50).cyan());
    println!("{}", "wzllama - Assistant IA Locale".bold().cyan());
    println!("{}", "═".repeat(50).cyan());

    let i18n = select_language()?;
    
    println!("\n{}", i18n.t("app.welcome").bold());

    println!("\n{}", i18n.t("system.detecting").dimmed());
    let hardware = detect_hardware();
    display_hardware(&hardware, &i18n);

    install_tools_if_needed(&i18n)?;

    let (usage_key, usage_spec) = launch_usage_wizard(&i18n)?;
    explain_usage(&usage_key, &usage_spec, &hardware, &i18n)?;

    println!("\n{}", i18n.t("app.goodbye").bold().green());
    Ok(())
}

fn select_language() -> Result<I18n> {
    let languages = get_available_languages();
    
    if languages.is_empty() {
        println!("⚠️  Aucune langue trouvée. Installation des templates...");
        config::ensure_user_templates()?;
    }

    let languages = get_available_languages();
    let system_lang = detect_system_language();
    
    let default_idx = languages
        .iter()
        .position(|l| l.code == system_lang)
        .unwrap_or(0);

    // Créer les items d'affichage
    let items: Vec<String> = languages
        .iter()
        .map(|l| {
            if let Some(name_en) = &l.name_en {
                format!("{} | {}", l.name, name_en)
            } else {
                l.name.clone()
            }
        })
        .collect();

    println!("{}", "═".repeat(50).cyan());
    println!("{}", "🌍 Choose your language / Choisissez votre langue".bold());

    let selection = Select::new()
        .with_prompt("Langue / Language / Sprache / Idioma :")
        .items(&items)
        .default(default_idx)
        .interact()?;

    let selected_lang = &languages[selection];
    let i18n = config::load_i18n(&selected_lang.code)?;

    println!("\n{} ({}) ✓", 
        i18n.t("menu.language.selected").green(),
        selected_lang.name
    );

    Ok(i18n)
}

fn display_hardware(hardware: &HardwareInfo, i18n: &I18n) {
    println!("\n{}", "─".repeat(40).dimmed());
    println!("{}", "Matériel détecté".bold());
    println!("{}", "─".repeat(40).dimmed());

    println!("  {}: {}", i18n.t("system.os").dimmed(), hardware.os.bold());
    println!("  {}: {:.1} Go", i18n.t("system.ram").dimmed(), hardware.ram_gb);

    if hardware.has_gpu() {
        for (i, gpu) in hardware.gpus.iter().enumerate() {
            println!("  {} #{}: {} ({}: {} Mo)",
                i18n.t("system.gpu").dimmed(), i + 1, gpu.name,
                i18n.t("system.vram").dimmed(), gpu.vram_mb);
        }
    } else {
        println!("  {}: {}", i18n.t("system.gpu").dimmed(), i18n.t("system.no_gpu").yellow());
    }
}

fn install_tools_if_needed(i18n: &I18n) -> Result<()> {
    println!("\n{}", "─".repeat(40).dimmed());
    println!("{}", "Vérification des outils".bold());
    println!("{}", "─".repeat(40).dimmed());

    let installer = Installer::new(i18n, true);
    for tool in &["ollama", "open-webui", "hermes", "openclaw"] {
        installer.check_and_install(tool)?;
    }
    Ok(())
}

fn launch_usage_wizard(i18n: &I18n) -> Result<(String, UsageSpec)> {
    println!("\n{}", "─".repeat(40).dimmed());
    println!("{}", i18n.t("menu.usage.title").bold());
    println!("{}", "─".repeat(40).dimmed());
    println!("{}\n", i18n.t("menu.usage.subtitle"));

    let usages_config = config::load_usages();
    let mut usages: Vec<(&String, &UsageSpec)> = usages_config.usages.iter().collect();

    usages.sort_by(|a, b| {
        let weight_a = a.1.weights.get("default").unwrap_or(&0.0);
        let weight_b = b.1.weights.get("default").unwrap_or(&0.0);
        weight_b.partial_cmp(weight_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    let items: Vec<String> = usages.iter().map(|(_, spec)| {
        let label = i18n.t(&spec.i18n_key);
        if let Some(desc_key) = &spec.description_key {
            format!("{}\n  {}", label.bold(), i18n.t(desc_key).dimmed())
        } else {
            label
        }
    }).collect();

    println!("{}:", i18n.t("menu.usage.sorted_by"));
    let selection = Select::new()
        .with_prompt("Choisissez votre usage")
        .items(&items)
        .interact()?;

    let key = usages[selection].0.clone();
    let spec = usages[selection].1.clone();
    Ok((key, spec))
}

fn explain_usage(
    usage_key: &str,
    spec: &UsageSpec,
    hardware: &HardwareInfo,
    i18n: &I18n,
) -> Result<()> {
    println!("\n{}", "─".repeat(40).dimmed());
    println!("{}", i18n.t("model.selection").bold());
    println!("{}", "─".repeat(40).dimmed());

    let usage_type = &spec.params.r#type;
    let (model_size, use_gpu) = core::recommend_model_size(usage_type, hardware);

    let usage_label = i18n.t(&spec.i18n_key);
    println!("\n📋 {} : {} (type : {})", "Usage".bold(), usage_label, usage_type.italic());
    let _ = usage_key;

    println!("\n{}: qwen2.5:{}b",
        i18n.t_with_vars("model.recommended", &[("model", "qwen2.5")]),
        model_size
    );
    println!("{}: {}B", 
        i18n.t_with_vars("model.size", &[("size", &model_size.to_string())]),
        model_size
    );

    if use_gpu {
        println!("✅ {}", i18n.t("model.vram_only").green());
    } else {
        let ram_needed = (model_size as f64) * 2.0;
        println!("⚠️  {}", i18n.t_with_vars("model.ram_needed", &[("gb", &format!("{:.0}", ram_needed))]).yellow());
    }

    println!("\n{}", "─".repeat(40).dimmed());
    println!("{}", i18n.t("estimation.title").bold());
    println!("{}", "─".repeat(40).dimmed());

    match usage_type.as_str() {
        "book" => {
            let pages: u32 = Input::new()
                .with_prompt("Nombre de pages de votre document")
                .default(100)
                .interact()?;

            let tokens = core::estimate_tokens_book(pages);
            let chunks = if let Some(cpp) = spec.params.pages_per_chunk {
                (pages + cpp - 1) / cpp
            } else { 1 };

            let tps = core::get_performance(model_size, use_gpu);
            let (min_time, max_time) = core::estimate_time_minutes(tokens, tps);

            println!("  📊 {}: {}", 
                i18n.t_with_vars("estimation.tokens", &[("tokens", &format_number(tokens))]),
                format_number(tokens)
            );
            println!("  📚 {}: {}", 
                i18n.t_with_vars("estimation.chunks", &[("chunks", &chunks.to_string())]),
                chunks
            );
            println!("  ⏱️  {}: {:.1} - {:.1} min", 
                i18n.t_with_vars("estimation.time_range", &[
                    ("min", &format!("{:.1}", min_time)),
                    ("max", &format!("{:.1}", max_time))
                ]),
                min_time, max_time
            );

            if model_size < 14 {
                println!("\n  {}", i18n.t("estimation.warning").yellow());
            }
        }
        "code" => {
            let loc: u32 = Input::new()
                .with_prompt("Nombre de lignes de code (approximatif)")
                .default(10000)
                .interact()?;

            let tokens = core::estimate_tokens_code(loc);
            let tps = core::get_performance(model_size, use_gpu);
            let (min_time, max_time) = core::estimate_time_minutes(tokens, tps);

            println!("  📊 {}: {}", 
                i18n.t_with_vars("estimation.tokens", &[("tokens", &format_number(tokens))]),
                format_number(tokens)
            );
            println!("  ⏱️  {}: {:.1} - {:.1} min", 
                i18n.t_with_vars("estimation.time_range", &[
                    ("min", &format!("{:.1}", min_time)),
                    ("max", &format!("{:.1}", max_time))
                ]),
                min_time, max_time
            );
        }
        _ => {
            println!("💡 Usage général - performance adaptative");
        }
    }

    Ok(())
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let len = s.len();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
    }
    result
}