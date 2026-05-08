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

    let _usage_type = &usage_spec.params.r#type;
    install_model_if_needed(&i18n, &hardware, _usage_type)?;

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
    let installer = Installer::new(i18n, true);
    installer.install_all_tools()
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

fn format_duration(minutes: f64) -> String {
    if minutes >= 120.0 {
        format!("{:.0}h{:02.0}min", minutes / 60.0, minutes % 60.0)
    } else if minutes >= 1.0 {
        format!("{:.0}min", minutes)
    } else {
        format!("{:.0}s", minutes * 60.0)
    }
}

fn explain_usage(
    _usage_key: &str,
    spec: &UsageSpec,
    hardware: &HardwareInfo,
    i18n: &I18n,
) -> Result<()> {
    println!("\n{}", "─".repeat(40).dimmed());
    println!("{}", i18n.t("model.selection").bold());
    println!("{}", "─".repeat(40).dimmed());

    let usage_type = &spec.params.r#type;
    let usage_label = i18n.t(&spec.i18n_key);
    println!("\n📋 {} : {} (type : {})", "Usage".bold(), usage_label, usage_type.italic());

    // Estimation
    println!("\n{}", "─".repeat(40).dimmed());
    println!("{}", i18n.t("estimation.title").bold());
    println!("{}", "─".repeat(40).dimmed());

    match usage_type.as_str() {
        "book" => {
            let pages: u32 = Input::new()
                .with_prompt("Nombre de pages")
                .default(100)
                .interact()?;
            let tokens = core::estimate_tokens_book(pages);
            let chunks = if let Some(cpp) = spec.params.pages_per_chunk {
                (pages + cpp - 1) / cpp
            } else { 1 };
            let tps = core::get_performance(14, true); // moyenne
            let (min_time, max_time) = core::estimate_time_minutes(tokens, tps);
            println!("  {}", i18n.t_with_vars("estimation.tokens", &[("tokens", &format_number(tokens))]));
            println!("  {}", i18n.t_with_vars("estimation.chunks", &[("chunks", &chunks.to_string())]));
            println!("  {}", i18n.t_with_vars("estimation.time_range", &[
                ("min", &format_duration(min_time)),
                ("max", &format_duration(max_time))
            ]));
        }
        "code" => {
            let loc: u32 = Input::new()
                .with_prompt("Lignes de code")
                .default(10000)
                .interact()?;
            let tokens = core::estimate_tokens_code(loc);
            let tps = core::get_performance(14, true);
            let (min_time, max_time) = core::estimate_time_minutes(tokens, tps);
            println!("  {}", i18n.t_with_vars("estimation.tokens", &[("tokens", &format_number(tokens))]));
            println!("  {}", i18n.t_with_vars("estimation.time_range", &[
                ("min", &format_duration(min_time)),
                ("max", &format_duration(max_time))
            ]));
        }
        _ => println!("💡 Usage général"),
    }

    Ok(())
}

fn install_model_if_needed(
    i18n: &I18n,
    hardware: &HardwareInfo,
    usage_type: &str,
) -> Result<()> {
    println!("\n{}", "─".repeat(40).dimmed());
    println!("{}", "📥 Modèle Ollama".bold());
    println!("{}", "─".repeat(40).dimmed());

    // Vérifier si Ollama est lancé
    let ollama_url = match core::detect_ollama_url() {
        Some(url) => url,
        None => {
            println!("   ⚠️  {}", i18n.t("install.ollama.not_running"));
            return Ok(());
        }
    };

    // Récupérer les modèles locaux
    let local_models = match core::fetch_local_models(&ollama_url) {
        Ok(models) => models,
        Err(_) => {
            println!("   ⚠️  {}", i18n.t("install.ollama.api_error"));
            return Ok(());
        }
    };

    if local_models.is_empty() {
        println!("   📭 {}", i18n.t("install.ollama.no_models"));
        println!("   💡 {}\n", i18n.t("install.ollama.pull_hint"));
        return Ok(());
    }

    // Classer les modèles locaux par pertinence
    let ranked = core::rank_local_models(&local_models, usage_type, hardware, 8);

    if ranked.is_empty() {
        println!("   ⚠️  {}", i18n.t("install.ollama.no_compatible"));
        return Ok(());
    }

    println!("\n📊 {} :\n", i18n.t("install.ollama.recommended"));
    
    let items: Vec<String> = ranked.iter().map(|(m, score)| {
        let size_str = m.details.as_ref()
            .and_then(|d| d.parameter_size.as_deref())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}B", core::extract_size(&m.name)));
        format!(
            "{} ({} - {:.0}%)",
            m.name.bold(),
            size_str,
            score * 100.0
        )
    }).collect();

    let selection = Select::new()
        .with_prompt(i18n.t("install.ollama.choose"))
        .items(&items)
        .default(0)
        .interact()?;

    let (chosen, _) = &ranked[selection];
    
    println!("\n✅ {} : {}", i18n.t("install.ollama.selected"), chosen.name.green().bold());
    println!("   📏 {} : {}", i18n.t("install.ollama.size"), 
        chosen.details.as_ref()
            .and_then(|d| d.parameter_size.as_deref())
            .unwrap_or("?"));
    println!("   🏷️  {} : {}", i18n.t("install.ollama.family"),
        chosen.details.as_ref()
            .and_then(|d| d.family.as_deref())
            .unwrap_or("inconnue"));
    
    // Proposer d'autres modèles à télécharger
    println!("\n💡 {} : ollama pull <nom_du_modèle>", i18n.t("install.ollama.pull_other"));

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} Go", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} Mo", bytes as f64 / 1_048_576.0)
    } else {
        format!("{} o", bytes)
    }
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