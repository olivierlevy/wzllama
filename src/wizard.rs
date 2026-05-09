use crate::config::{self, I18n, UsageSpec, get_available_languages, detect_system_language};
use crate::core::{self, detect_hardware, HardwareInfo, OllamaModel, TaskType};
use crate::installers::Installer;
use anyhow::Result;
use colored::*;
use dialoguer::{Select, Input, Confirm};
use std::path::PathBuf;

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
    _hardware: &HardwareInfo,
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

    // Récupérer les modèles locaux
    let local_models = match core::detect_ollama_url() {
        Some(url) => core::fetch_local_models(&url).unwrap_or_default(),
        None => vec![],
    };

    // Récupérer le catalogue distant
    println!("   {}", i18n.t("install.ollama.searching"));
    let remote_models = core::fetch_remote_catalog().unwrap_or_default();
    
    if remote_models.is_empty() && local_models.is_empty() {
        println!("   ⚠️  {}", i18n.t("install.ollama.no_models_at_all"));
        println!("   {}", i18n.t("install.ollama.check_connection"));
        return Ok(());
    }

    // Fusionner
    let all_models = core::get_all_available_models(&local_models, &remote_models);
    
    // Classer par pertinence
    let scored: Vec<(OllamaModel, bool, f32)> = all_models.iter()
        .filter(|(m, _)| {
            let size = core::extract_size(&m.name);
            // Filtrer les modèles trop gros (> VRAM * 2) et les cloud
            size > 0 && !m.name.to_lowercase().contains("cloud")
        })
        .map(|(m, downloaded)| {
            let score = core::score_model_dynamic(m, usage_type, hardware);
            (m.clone(), *downloaded, score)
        })
        .filter(|(_, _, s)| *s > 0.0)
        .collect();

    let mut ranked = scored;
    ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(15);

    if ranked.is_empty() {
        println!("   ⚠️  {}", i18n.t("install.ollama.no_compatible"));
        return Ok(());
    }

    println!("\n📊 {} :\n", i18n.t("install.ollama.recommended"));
    
    let items: Vec<String> = ranked.iter().map(|(m, downloaded, score)| {
        let status = if *downloaded { "✅" } else { "⬇️ " };
        let size_str = format_size_from_bytes(m.size.unwrap_or(0));
        format!(
            "{} {} ({} - {:.0}%)",
            status,
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

    let (chosen, _downloaded, _) = &ranked[selection];
    
    let task = TaskType::from_str(usage_type);
    let config = core::recommend_config_i18n(hardware, &task, chosen, i18n);
    let custom_name = format!("wzllama-{}", task.to_str());
    
    println!("\n✅ {} : {}", i18n.t("install.ollama.selected"), chosen.name.green().bold());
    println!("   📏 {} : {}", i18n.t("install.ollama.size"), format_size_from_bytes(chosen.size.unwrap_or(0)));
    
    // Afficher la configuration recommandée
    core::display_config_summary_i18n(&config, i18n);
    
    // Interaction : que faire maintenant ?
    println!("\n💡 {}", i18n.t("config.launch_now"));
    let items = vec![
        i18n.t("config.launch_option_env"),
        i18n.t("config.launch_option_create"),
        i18n.t("config.launch_option_tools"),
        i18n.t("config.launch_option_fleet"),
        i18n.t("config.launch_option_quit"),
    ];

    let launch = Select::new()
        .with_prompt(i18n.t("config.launch_choose"))
        .items(&items)
        .default(0)
        .interact()?;

    match launch {
        0 => {
            println!("\n{}", config.env_vars_display().cyan());
            println!("ollama run {}", chosen.name);
        }
        1 => {
            let custom_name = format!("wzllama-{}", task.to_str());
            match config.write_and_create(&custom_name) {
                Ok(cmd) => {
                    println!("   ⚙️  {}", i18n.t("config.creating"));
                    match core::run_command(&cmd) {
                        Ok(_) => {
                            println!("   {}", i18n.t_with_vars("config.created", &[("name", &custom_name)]));
                            println!("   {}", i18n.t_with_vars("config.launch", &[("name", &custom_name)]));
                        }
                        Err(e) => {
                            println!("   {}: {}", i18n.t("config.create_error"), e);
                            println!("   {}", cmd.cyan());
                        }
                    }
                }
                Err(e) => {
                    println!("   {}: {}", i18n.t("config.create_error"), e);
                }
            }
        }
        2 => {
            // Lancer avec les outils installés
            launch_with_tools(i18n, &custom_name)?;
        }
        3 => {
            create_agent_fleet(i18n, hardware, chosen, &custom_name, usage_type)?;
        }
        _ => {}
    }
    Ok(())
}

fn format_size_from_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} Go", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} Mo", bytes as f64 / 1_048_576.0)
    } else if bytes == 0 {
        "?".to_string()
    } else {
        format!("{} o", bytes)
    }
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

fn launch_with_tools(
    i18n: &I18n,
    model_name: &str,
) -> Result<()> {
    println!("\n{}", "─".repeat(40).dimmed());
    println!("{}", i18n.t("config.tools.title").bold());
    println!("{}", "─".repeat(40).dimmed());

    let tools = vec![
        ("open-webui", "Open WebUI", "http://localhost:3000", true),
        ("claude-code", "Claude Code", "https://github.com/anthropics/claude-code", false),
        ("openclaw", "OpenClaw", "https://github.com/openclaw/openclaw", false),
        ("hermes-agent", "Hermes Agent", "https://github.com/hermes-agent/hermes-agent", false),
    ];

    let items: Vec<String> = tools.iter().map(|(cmd, name, url, docker)| {
        let installed = if *docker {
            // Vérifier le conteneur Docker
            core::run_command(&format!("sudo docker ps --format '{{{{.Names}}}}' 2>/dev/null | grep -q {}", cmd))
                .is_ok()
        } else {
            core::run_command(&format!("command -v {} 2>/dev/null", cmd)).is_ok()
        };
        let status = if installed { "✅" } else { "❌" };
        format!("{} {} ({})", status, name, url.dimmed())
    }).collect();

    let selection = Select::new()
        .with_prompt(i18n.t("config.tools.choose"))
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            println!("\n🌐 {}", i18n.t("config.tools.open_webui"));
            println!("   http://localhost:3000");
            println!("   {} {} {}", i18n.t("config.tools.model_available"), model_name.cyan(), i18n.t("config.tools.in_interface"));
        }
        1 => {
            println!("\n💻 {}", i18n.t("config.tools.claude_code"));
            println!("   export ANTHROPIC_BASE_URL=http://localhost:11434/v1");
            println!("   export ANTHROPIC_API_KEY=ollama");
            println!("   claude-code --model {}", model_name.cyan());
        }
        2 => {
            println!("\n🦞 {}", i18n.t("config.tools.openclaw"));
            println!("   openclaw --model ollama/{}", model_name.cyan());
        }
        3 => {
            println!("\n🤖 {}", i18n.t("config.tools.hermes"));
            println!("   hermes-agent --model ollama/{}", model_name.cyan());
        }
        _ => {}
    }

    Ok(())
}

fn create_agent_fleet(
    i18n: &I18n,
    hardware: &HardwareInfo,
    chosen: &OllamaModel,
    orchestrator_name: &str,
    usage_type: &str,
) -> Result<()> {
    println!("\n{}", "═".repeat(50).cyan());
    println!("{}", i18n.t("fleet.title").bold());
    println!("{}", "═".repeat(50).cyan());
    
    let wizard_model = &chosen.model;
    let mut fleet = get_fleet_templates(usage_type, wizard_model, i18n);
    
    // Afficher les ressources
    let capacity = core::calculate_fleet_capacity(hardware, chosen);
    
    println!("\n{}", i18n.t("fleet.resources"));
    println!("   💾 RAM : {:.1} Go", capacity.ram_total_gb);
    println!("   🎮 VRAM : {:.1} Go", capacity.vram_total_gb);
    println!();
    println!("   🎯 {} (orchestrateur) : {} - {} tokens", 
        fleet.orchestrator.model.cyan(), 
        i18n.t("fleet.ctx_long"),
        fleet.orchestrator.num_ctx);
    println!("   🧠 {} (réflexion) : {} - {} tokens",
        wizard_model.cyan(),
        i18n.t("fleet.ctx_short"),
        fleet.reflexion_agents.first().map(|a| a.num_ctx).unwrap_or(4096));
    println!("   🤖 Experts : {} agents max en RAM", capacity.max_experts_ram);
    println!();
    
    // Étape 1 : Éditer l'orchestrateur
    println!("{}", i18n.t("fleet.step_orchestrator").bold());
    let keep_orch = Confirm::new()
        .with_prompt(i18n.t("fleet.keep_orchestrator"))
        .default(true)
        .interact()?;
    
    if !keep_orch {
        println!("   {}", i18n.t("fleet.skipping_orchestrator"));
    }
    
    // Étape 2 : Éditer les agents de réflexion
    println!("\n{}", i18n.t("fleet.step_reflexion").bold());
    for (i, agent) in fleet.reflexion_agents.iter_mut().enumerate() {
        edit_agent_template(i18n, agent, i, "réflexion")?;
    }
    
    // Étape 3 : Éditer les agents experts
    println!("\n{}", i18n.t("fleet.step_experts").bold());
    for (i, agent) in fleet.expert_agents.iter_mut().enumerate() {
        edit_agent_template(i18n, agent, i, "expert")?;
    }
    
    // Étape 4 : Ajouter des agents experts personnalisés
    println!("\n{}", i18n.t("fleet.add_more"));
    let add_more = Confirm::new()
        .with_prompt(i18n.t("fleet.add_more_confirm"))
        .default(false)
        .interact()?;
    
    while add_more {
        let role: String = Input::new()
            .with_prompt(i18n.t("fleet.custom_role"))
            .interact()?;
        let prompt: String = Input::new()
            .with_prompt(i18n.t("fleet.custom_prompt"))
            .interact()?;
        
        fleet.expert_agents.push(AgentTemplate {
            name: format!("wzllama-expert-custom-{}", fleet.expert_agents.len() + 1),
            role,
            model: "qwen2.5:3b".into(),
            num_ctx: 4096,
            temperature: 0.5,
            system_prompt: prompt,
            enabled: true,
        });
        
        let more = Confirm::new()
            .with_prompt(i18n.t("fleet.add_another"))
            .default(false)
            .interact()?;
        if !more { break; }
    }
    
    // Étape 5 : Création effective
    println!("\n{}", "═".repeat(50).cyan());
    println!("{}", i18n.t("fleet.creating_fleet").bold());
    println!("{}", "═".repeat(50).cyan());
    
    let mut created = Vec::new();
    
    // Orchestrateur
    if keep_orch {
        println!("\n🎯 {}", i18n.t("fleet.creating_orchestrator"));
        if create_single_agent(
            &fleet.orchestrator.model,
            orchestrator_name,
            fleet.orchestrator.num_ctx,
            0.7,
            &fleet.orchestrator.system_prompt,
        ).is_ok() {
            created.push((orchestrator_name.to_string(), "🎯".to_string()));
        }
    }
    
    // Réflexion
    for agent in &fleet.reflexion_agents {
        if agent.enabled {
            println!("\n🧠 {}", agent.role);
            if create_single_agent(
                &agent.model,
                &agent.name,
                agent.num_ctx,
                agent.temperature,
                &agent.system_prompt,
            ).is_ok() {
                created.push((agent.name.clone(), "🧠".to_string()));
            }
        }
    }
    
    // Experts
    for agent in &fleet.expert_agents {
        if agent.enabled {
            println!("\n🤖 {}", agent.role);
            if create_single_agent(
                &agent.model,
                &agent.name,
                agent.num_ctx,
                agent.temperature,
                &agent.system_prompt,
            ).is_ok() {
                created.push((agent.name.clone(), "🤖".to_string()));
            }
        }
    }
    
    // Résumé final
    println!("\n{}", "═".repeat(50).cyan());
    println!("{}", i18n.t("fleet.summary").bold());
    println!("{}", "═".repeat(50).cyan());
    
    if created.is_empty() {
        println!("   {}", i18n.t("fleet.nothing_created"));
        return Ok(());
    }
    
    for (name, emoji) in &created {
        println!("   {} {}", emoji, name.cyan());
    }

    // Demander le nom du projet
    let project_name: String = Input::new()
        .with_prompt(i18n.t("fleet.project_name"))
        .default(usage_type.to_string())
        .interact()?;
    
    // Créer le dossier ~/.openclaw-{projet}/
    let openclaw_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(format!(".openclaw-{}", project_name));
    std::fs::create_dir_all(&openclaw_dir)?;
    
    // Construire le openclaw.json
    let mut agents_json = String::new();
    
    // Orchestrateur
    agents_json.push_str(&format!(
        "      {{ \"id\": \"orchestrator\", \"identity\": {{ \"name\": \"{}\" }} }},\n",
        fleet.orchestrator.system_prompt.lines().next().unwrap_or("Coordinateur")
    ));
    
    // Agents de réflexion
    for agent in &fleet.reflexion_agents {
        if agent.enabled {
            let id = agent.name
                .strip_prefix("wzllama-reflexion-")
                .unwrap_or(&agent.name);
            agents_json.push_str(&format!(
                "      {{ \"id\": \"{}\", \"model\": {{ \"primary\": \"ollama/{}\" }}, \"identity\": {{ \"name\": \"{}\" }} }},\n",
                id, agent.name, agent.role
            ));
        }
    }
    
    // Agents experts
    for agent in &fleet.expert_agents {
        if agent.enabled {
            let id = agent.name
                .strip_prefix("wzllama-expert-")
                .unwrap_or(&agent.name);
            agents_json.push_str(&format!(
                "      {{ \"id\": \"{}\", \"model\": {{ \"primary\": \"ollama/{}\" }}, \"identity\": {{ \"name\": \"{}\" }} }},\n",
                id, agent.name, agent.role
            ));
        }
    }
    
    // Enlever la dernière virgule
    if agents_json.ends_with(",\n") {
        agents_json.pop(); // \n
        agents_json.pop(); // ,
        agents_json.push('\n');
    }
    
    let openclaw_config = format!(r#"{{
  "gateway": {{
    "mode": "local"
  }},
  "agents": {{
    "defaults": {{
      "model": {{ "primary": "ollama/{}" }}
    }},
    "list": [
{}
    ]
  }}
}}"#, orchestrator_name, agents_json);
    
    let config_path = openclaw_dir.join("openclaw.json");
    std::fs::write(&config_path, &openclaw_config)?;
    
    // Instructions
    println!("\n💡 {}", i18n.t("fleet.usage_hint"));
    println!();
    println!("   {}", i18n.t("fleet.openclaw_launch"));
    println!("   openclaw --profile {}", project_name.cyan());
    println!();
    println!("   {}", i18n.t("fleet.openclaw_agents"));
    println!("   agents");
    println!();
    println!("   {}", i18n.t("fleet.openclaw_switch"));
    println!("   /agent style");
    println!("   /agent plot");
    println!();
    println!("   📄 Config : {}", config_path.display().to_string().cyan());

    Ok(())
}

fn create_single_agent(
    model: &str,
    name: &str,
    num_ctx: u32,
    temperature: f32,
    system_prompt: &str,
) -> Result<()> {
    // Vérifier si le modèle de base est disponible
    let installed = core::run_command(&format!("ollama list 2>/dev/null | grep -q {}", model)).is_ok();
    if !installed {
        println!("   ⬇️  Téléchargement de {}...", model);
        core::run_command(&format!("ollama pull {}", model))?;
    }
    
    let modelfile = format!(
        "FROM {}\nPARAMETER num_ctx {}\nPARAMETER temperature {:.1}\nSYSTEM \"{}\"",
        model, num_ctx, temperature, system_prompt
    );
    
    let tmp_file = format!("/tmp/wzllama_{}", name);
    std::fs::write(&tmp_file, &modelfile)?;
    
    let create_cmd = format!("ollama create {} -f {}", name, tmp_file);
    
    match core::run_command(&create_cmd) {
        Ok(_) => {
            println!("   ✅ {} créé", name.cyan());
        }
        Err(e) => {
            println!("   ❌ Erreur : {}", e);
        }
    }
    
    Ok(())
}


#[derive(Debug, Clone)]
struct FleetConfig {
    orchestrator: OrchestratorConfig,
    reflexion_agents: Vec<AgentTemplate>,
    expert_agents: Vec<AgentTemplate>,
}

#[derive(Debug, Clone)]
struct OrchestratorConfig {
    model: String,
    num_ctx: u32,
    system_prompt: String,
}

#[derive(Debug, Clone)]
struct AgentTemplate {
    name: String,
    role: String,
    model: String,
    num_ctx: u32,
    temperature: f32,
    system_prompt: String,
    enabled: bool,  // l'utilisateur peut décocher
}

/// Retourne les templates selon le type d'usage
fn get_fleet_templates(usage_type: &str, wizard_model: &str, i18n: &I18n) -> FleetConfig {
    match usage_type {
        "book" => FleetConfig {
            orchestrator: OrchestratorConfig {
                model: "qwen2.5:7b".into(),
                num_ctx: 32768,
                system_prompt: i18n.t("fleet.template.orchestrator_book"),
            },
            reflexion_agents: vec![
                AgentTemplate {
                    name: "wzllama-reflexion-style".into(),
                    role: i18n.t("fleet.template.reflexion_style"),
                    model: wizard_model.into(),
                    num_ctx: 8192,
                    temperature: 0.7,
                    system_prompt: i18n.t("fleet.template.reflexion_style_prompt"),
                    enabled: true,
                },
                AgentTemplate {
                    name: "wzllama-reflexion-plot".into(),
                    role: i18n.t("fleet.template.reflexion_plot"),
                    model: wizard_model.into(),
                    num_ctx: 8192,
                    temperature: 0.5,
                    system_prompt: i18n.t("fleet.template.reflexion_plot_prompt"),
                    enabled: true,
                },
            ],
            expert_agents: vec![
                AgentTemplate {
                    name: "wzllama-expert-grammar".into(),
                    role: i18n.t("fleet.template.expert_grammar"),
                    model: "qwen2.5:3b".into(),
                    num_ctx: 4096,
                    temperature: 0.3,
                    system_prompt: i18n.t("fleet.template.expert_grammar_prompt"),
                    enabled: true,
                },
                AgentTemplate {
                    name: "wzllama-expert-research".into(),
                    role: i18n.t("fleet.template.expert_research"),
                    model: "qwen2.5:3b".into(),
                    num_ctx: 4096,
                    temperature: 0.6,
                    system_prompt: i18n.t("fleet.template.expert_research_prompt"),
                    enabled: true,
                },
                AgentTemplate {
                    name: "wzllama-expert-dialogue".into(),
                    role: i18n.t("fleet.template.expert_dialogue"),
                    model: "qwen2.5:3b".into(),
                    num_ctx: 4096,
                    temperature: 0.9,
                    system_prompt: i18n.t("fleet.template.expert_dialogue_prompt"),
                    enabled: true,
                },
            ],
        },
        "code" => FleetConfig {
            orchestrator: OrchestratorConfig {
                model: "qwen2.5:7b".into(),
                num_ctx: 32768,
                system_prompt: i18n.t("fleet.template.orchestrator_code"),
            },
            reflexion_agents: vec![
                AgentTemplate {
                    name: "wzllama-reflexion-arch".into(),
                    role: i18n.t("fleet.template.reflexion_arch"),
                    model: wizard_model.into(),
                    num_ctx: 8192,
                    temperature: 0.3,
                    system_prompt: i18n.t("fleet.template.reflexion_arch_prompt"),
                    enabled: true,
                },
                AgentTemplate {
                    name: "wzllama-reflexion-review".into(),
                    role: i18n.t("fleet.template.reflexion_review"),
                    model: wizard_model.into(),
                    num_ctx: 8192,
                    temperature: 0.4,
                    system_prompt: i18n.t("fleet.template.reflexion_review_prompt"),
                    enabled: true,
                },
            ],
            expert_agents: vec![
                AgentTemplate {
                    name: "wzllama-expert-lint".into(),
                    role: i18n.t("fleet.template.expert_lint"),
                    model: "qwen2.5:1.5b".into(),
                    num_ctx: 2048,
                    temperature: 0.1,
                    system_prompt: i18n.t("fleet.template.expert_lint_prompt"),
                    enabled: true,
                },
                AgentTemplate {
                    name: "wzllama-expert-doc".into(),
                    role: i18n.t("fleet.template.expert_doc"),
                    model: "qwen2.5:3b".into(),
                    num_ctx: 4096,
                    temperature: 0.4,
                    system_prompt: i18n.t("fleet.template.expert_doc_prompt"),
                    enabled: true,
                },
                AgentTemplate {
                    name: "wzllama-expert-test".into(),
                    role: i18n.t("fleet.template.expert_test"),
                    model: "qwen2.5:3b".into(),
                    num_ctx: 4096,
                    temperature: 0.5,
                    system_prompt: i18n.t("fleet.template.expert_test_prompt"),
                    enabled: true,
                },
            ],
        },
        _ => FleetConfig {
            orchestrator: OrchestratorConfig {
                model: "qwen2.5:7b".into(),
                num_ctx: 16384,
                system_prompt: i18n.t("fleet.template.orchestrator_generic"),
            },
            reflexion_agents: vec![
                AgentTemplate {
                    name: "wzllama-reflexion".into(),
                    role: i18n.t("fleet.template.reflexion_generic"),
                    model: wizard_model.into(),
                    num_ctx: 8192,
                    temperature: 0.5,
                    system_prompt: i18n.t("fleet.template.reflexion_generic_prompt"),
                    enabled: true,
                },
            ],
            expert_agents: vec![
                AgentTemplate {
                    name: "wzllama-expert-fast".into(),
                    role: i18n.t("fleet.template.expert_fast"),
                    model: "qwen2.5:1.5b".into(),
                    num_ctx: 2048,
                    temperature: 0.7,
                    system_prompt: i18n.t("fleet.template.expert_fast_prompt"),
                    enabled: true,
                },
            ],
        },
    }
}

/// Affiche et permet de modifier un template d'agent
fn edit_agent_template(
    i18n: &I18n,
    template: &mut AgentTemplate,
    index: usize,
    agent_type: &str,
) -> Result<()> {
    println!("\n{} {}/{} : {}", "─".repeat(40).dimmed(), index + 1, agent_type, template.role.bold());
    println!("   {}: {}", i18n.t("fleet.edit.model"), template.model.dimmed());
    println!("   {}: {}", i18n.t("fleet.edit.ctx"), template.num_ctx);
    
    let items = vec![
        format!("{} {}", if template.enabled { "✅" } else { "❌" }, i18n.t("fleet.edit.toggle")),
        i18n.t("fleet.edit.role"),
        i18n.t("fleet.edit.system_prompt"),
        i18n.t("fleet.edit.keep"),
    ];
    
    let selection = Select::new()
        .with_prompt(i18n.t("fleet.edit.choose"))
        .items(&items)
        .default(3)
        .interact()?;
    
    match selection {
        0 => {
            template.enabled = !template.enabled;
            println!("   {} {}", i18n.t("fleet.edit.enabled"), if template.enabled { "✅" } else { "❌" });
        }
        1 => {
            let new_role: String = Input::new()
                .with_prompt(i18n.t("fleet.edit.new_role"))
                .default(template.role.clone())
                .interact()?;
            template.role = new_role;
        }
        2 => {
            let new_prompt: String = Input::new()
                .with_prompt(i18n.t("fleet.edit.new_prompt"))
                .default(template.system_prompt.clone())
                .interact()?;
            template.system_prompt = new_prompt;
        }
        _ => {}
    }
    
    Ok(())
}

