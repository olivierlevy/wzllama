//! Scientific Agent Skills menu
//! Integrates with K-Dense-AI/scientific-agent-skills for local AI scientist capabilities

use anyhow::Result;
use colored::*;
use dialoguer::Select;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::display;
use super::menu_header;

/// Categories from scientific-agent-skills
pub struct ScientificCategory {
    pub name_key: &'static str,
    pub skills: &'static [&'static str],
}

/// Agentic tool info for scientific workflows
pub struct AgenticToolInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description_key: &'static str,
    pub best_for: &'static str,
}

impl AgenticToolInfo {
    pub fn all() -> Vec<Self> {
        vec![
            AgenticToolInfo {
                id: "claude_code",
                name: "Claude Code",
                description_key: "scientific.agentic.claude_code",
                best_for: "Débogage, refactoring, documentation",
            },
            AgenticToolInfo {
                id: "opencode",
                name: "OpenCode",
                description_key: "scientific.agentic.opencode",
                best_for: "Développement multi-modèles",
            },
            AgenticToolInfo {
                id: "droid",
                name: "Droid",
                description_key: "scientific.agentic.droid",
                best_for: "Workflows ML/AI complexes",
            },
            AgenticToolInfo {
                id: "codex",
                name: "Codex",
                description_key: "scientific.agentic.codex",
                best_for: "Génération de code scientifique",
            },
        ]
    }
}

impl ScientificCategory {
    pub fn all() -> Vec<Self> {
        vec![
            ScientificCategory {
                name_key: "scientific.bioinformatics",
                skills: &["biopython", "bioservices", "gget", "scanpy", "anndata", "cellxgene-census"],
            },
            ScientificCategory {
                name_key: "scientific.cheminformatics",
                skills: &["rdkit", "deepchem", "datamol", "diffdock"],
            },
            ScientificCategory {
                name_key: "scientific.proteomics",
                skills: &["pyzeroconf", "dhdna-profiler", "mdanalysis"],
            },
            ScientificCategory {
                name_key: "scientific.clinical",
                skills: &["clinical-decision-support", "primekg"],
            },
            ScientificCategory {
                name_key: "scientific.genomics",
                skills: &["gget", "bioservices", "aeon", "arboreto"],
            },
            ScientificCategory {
                name_key: "scientific.ml",
                skills: &["scikit-learn", "pytorch-lightning", "pennylane", "qiskit", "cirq"],
            },
        ]
    }
}

/// Main scientific skills menu
pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    loop {
        // Affiche le header avec ressources comme le menu principal
        menu_header::render(
            i18n,
            "scientific.title",
            true,
            state.last_model.as_deref(),
            hw.ram_gb,
            hw.total_vram_mb as f64 / 1024.0
        );
        
        let categories = ScientificCategory::all();
        let agentic_tools = AgenticToolInfo::all();
        let display_names: Vec<String> = categories.iter()
            .map(|c| i18n.t(c.name_key))
            .collect();
        
        // Add agentic tools option at the end
        let back_option = i18n.t("menu.back");
        let mut all_items = display_names.clone();
        all_items.push(i18n.t("scientific.agentic_tools"));
        all_items.push(back_option.clone());
        
        let sel = Select::new()
            .with_prompt("")
            .items(&all_items)
            .default(0)
            .interact_opt()?;
        
        match sel {
            Some(s) if s < categories.len() => {
                handle_category(i18n, state, hw, &categories[s])?;
            }
            Some(s) if s == categories.len() => {
                show_agentic_tools_info(i18n, &agentic_tools)?;
            }
            _ => return Ok(()),
        }
    }
}

/// Handle category selection
fn handle_category(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo, category: &ScientificCategory) -> Result<()> {
    // Show available skills in this category
    display::info(&i18n.t("scientific.category_skills"));
    for skill in category.skills {
        let installed = is_skill_installed(skill);
        let status = if installed { "✅" } else { "📄" };
        println!("  {} {}", status, skill);
    }
    
    // Offer to install missing skills
    let missing: Vec<&str> = category.skills.iter()
        .filter(|s| !is_skill_installed(s))
        .copied()
        .collect();
    
    if !missing.is_empty() {
        let install = dialoguer::Confirm::new()
            .with_prompt(i18n.t("scientific.install_missing"))
            .default(true)
            .interact()?;
        
        if install {
            install_scientific_skills(i18n, &missing)?;
        }
    }
    
    // Launch the scientific agent
    launch_scientific_agent(i18n, state, hw, category)?;
    
    Ok(())
}

/// Check if a skill is installed
fn is_skill_installed(skill: &str) -> bool {
    // Check if the skill directory exists in ~/.wzllama/scientific-skills/
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let skill_path = format!("{}/.wzllama/scientific-skills/{}/SKILL.md", home, skill);
    std::path::Path::new(&skill_path).exists()
}

/// Install scientific skills
fn install_scientific_skills(i18n: &I18n, skills: &[&str]) -> Result<()> {
    display::info(&format!("{} {} {}", i18n.t("scientific.downloading"), skills.len(), i18n.t("scientific.skills")));
    
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let base_path = format!("{}/.wzllama/scientific-skills", home);
    
    // Create directory
    std::fs::create_dir_all(&base_path)?;
    
    // Download skills from GitHub
    for skill in skills {
        let url = format!("https://raw.githubusercontent.com/K-Dense-AI/scientific-agent-skills/main/scientific-skills/{}/SKILL.md", skill);
        let dest = format!("{}/{}/SKILL.md", base_path, skill);
        
        if let Ok(content) = reqwest::blocking::get(&url) {
            if let Ok(text) = content.text() {
                let skill_dir = format!("{}/{}", base_path, skill);
                std::fs::create_dir_all(&skill_dir)?;
                std::fs::write(&dest, text)?;
                println!("  ✅ {}", skill);
            }
        }
    }
    
    display::success(&i18n.t("scientific.skills_installed"));
    Ok(())
}

/// Launch scientific agent with appropriate model
fn launch_scientific_agent(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo, category: &ScientificCategory) -> Result<()> {
    // Recommend model based on category
    let model = recommend_model_for_category(category);
    
    display::info(&i18n.t_with_vars("scientific.recommended_model", &[("model", model)]));
    
    // Check if model is installed
    let local_models = crate::core::ollama_api::get_models();
    let has_model = local_models.iter().any(|m| m.name == model);
    
    if !has_model {
        let install = dialoguer::Confirm::new()
            .with_prompt(i18n.t_with_vars("scientific.download_model", &[("model", model)]))
            .default(true)
            .interact()?;
        
        if install {
            if let Err(e) = crate::core::ollama_api::pull_model(model) {
                display::warning(&format!("{}: {}", i18n.t("models.localmaxxing_download_error"), e));
                return Ok(());
            }
        }
    }
    
    // Save model for display in header
    state.last_model = Some(model.to_string());
    crate::config::state::save(state)?;
    
    // Show agentic launcher menu
    run_agentic_launcher_menu(i18n, state, hw, category)?;
    
    Ok(())
}

/// Recommend model based on category
fn recommend_model_for_category(_category: &ScientificCategory) -> &'static str {
    // For scientific work, we recommend models good at reasoning and analysis
    "qwen2.5:7b"
}

/// Run agentic launcher submenu for scientific skills
fn run_agentic_launcher_menu(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo, category: &ScientificCategory) -> Result<()> {
    let agentic_tools = AgenticToolInfo::all();
    let skills_path = format!("{}/.wzllama/scientific-skills", 
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
    
    loop {
        // Affiche le header avec ressources
        menu_header::render(
            i18n,
            "scientific.launcher",
            true,
            state.last_model.as_deref(),
            hw.ram_gb,
            hw.total_vram_mb as f64 / 1024.0
        );
        
        // Show available skills
        display::info(&i18n.t("scientific.available_skills"));
        for skill in category.skills {
            let installed = is_skill_installed(skill);
            let status = if installed { "✅" } else { "📄" };
            println!("  {} {}", status, skill);
        }
        println!();
        
        // Build tools menu
        let mut items = vec![i18n.t("scientific.launcher_back")];
        for tool in &agentic_tools {
            let installed = match tool.id {
                "claude_code" => crate::core::shell::is_installed_with_local_bin("claude"),
                "opencode" => crate::core::shell::is_installed_with_local_bin("opencode"),
                "droid" => crate::core::shell::is_installed_with_local_bin("droid"),
                "codex" => crate::core::shell::is_installed_with_local_bin("codex"),
                _ => false,
            };
            let icon = if installed { "🤖" } else { "📦" };
            items.push(format!("{} {}", icon, tool.name));
        }
        
        let sel = Select::new()
            .with_prompt(i18n.t("scientific.launcher_prompt"))
            .items(&items)
            .default(0)
            .interact_opt()?;
        
        match sel {
            Some(0) | None => return Ok(()), // Back (index 0 or Escape)
            Some(idx) if idx <= agentic_tools.len() => {
                let tool = &agentic_tools[idx - 1];
                show_tool_launch_instructions(i18n, tool, &skills_path, state)?;
            }
            _ => {}
        }
    }
}

/// Show launch instructions for a specific tool with scientific skills
fn show_tool_launch_instructions(i18n: &I18n, tool: &AgenticToolInfo, skills_path: &str, state: &mut WzllamaState) -> Result<()> {
    display::clear_screen();
    display::section(&format!("{} {}", tool.name, i18n.t("scientific.launcher_instructions")));
    
    // Create symlinks for tool-specific skills directories
    let symlinks_created = create_skill_symlinks(tool, skills_path)?;
    if symlinks_created {
        display::success(&i18n.t("scientific.skills_linked"));
    }
    
    // Tool-specific instructions
    match tool.id {
        "claude_code" => {
            println!("📋 Claude Code Scientific Agent Guide:");
            println!();
            println!("1. Install Claude Code:");
            println!("   $ curl -fsSL https://claude.ai/install.sh | bash");
            println!();
            println!("2. In Claude Code session, type:");
            println!("   /skills");
            println!();
            println!("3. The scientific skills are available in:");
            println!("   ~/.claude/skills/ -> {}", skills_path);
            println!();
            println!("4. Example prompts:");
            println!("   > /skills analyze this dataset");
            println!("   > /skills run bioinformatics workflow");
        }
        "opencode" => {
            println!("📋 OpenCode Scientific Agent Guide:");
            println!();
            println!("1. Install OpenCode:");
            println!("   $ curl -fsSL https://opencode.ai/install | bash");
            println!();
            println!("2. Launch with skills directory:");
            println!("   $ opencode --cwd {}", skills_path);
            println!();
            println!("3. Or in OpenCode session:");
            println!("   > Load skills from {}", skills_path);
        }
        "droid" => {
            println!("📋 Droid Scientific Agent Guide:");
            println!();
            println!("1. Install Droid:");
            println!("   $ curl -fsSL https://app.factory.ai/cli | sh");
            println!();
            println!("2. Launch with skills directory:");
            println!("   $ droid --agent=researcher --knowledge={}", skills_path);
        }
        "codex" => {
            println!("📋 Codex Scientific Agent Guide:");
            println!();
            println!("1. Install Codex:");
            println!("   $ npm install -g @openai/codex");
            println!();
            println!("2. Launch with project context:");
            println!("   $ codex --model gpt-4-turbo");
            println!();
            println!("3. Then reference skills in prompts:");
            println!("   > Use {} for bioinformatics workflows", skills_path);
        }
        _ => {
            println!("Instructions coming soon for {}", tool.name);
        }
    }
    
    println!();
    
    // Check if tool is installed
    let is_installed = match tool.id {
        "claude_code" => crate::core::shell::is_installed_with_local_bin("claude"),
        "opencode" => crate::core::shell::is_installed_with_local_bin("opencode"),
        "droid" => crate::core::shell::is_installed_with_local_bin("droid"),
        "codex" => crate::core::shell::is_installed_with_local_bin("codex"),
        _ => false,
    };
    
    let primary_option = if is_installed {
        format!("🚀 {}", i18n.t("scientific.launch_tool"))
    } else {
        format!("📦 {}", i18n.t("scientific.install_tool"))
    };
    let back_option = i18n.t("scientific.launcher_back");
    
    let options = [primary_option.as_str(), back_option.as_str()];
    let sel = Select::new()
        .with_prompt(i18n.t("scientific.launcher_action"))
        .items(options)
        .default(1)
        .interact_opt()?;
    
    // Handle selection - None means Escape was pressed (treat as back)
    if let Some(0) = sel {
        if let Some(tool_instance) = crate::tools::get_tool(tool.id) {
            if is_installed {
                tool_instance.launch(i18n, state, state.last_model.as_deref())?;
                state.set_last_tool(tool.id);
            } else {
                display::info(&format!("{}...", i18n.t("scientific.installing_tool")));
                tool_instance.install(i18n)?;
            }
        }
    }
    // None (Escape) falls through to return naturally
    
    Ok(())
}

/// Create symlinks from ~/.wzllama/scientific-skills/* to tool-specific skills directories
fn create_skill_symlinks(tool: &AgenticToolInfo, skills_path: &str) -> Result<bool> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let tool_skills_path = match tool.id {
        "claude_code" => format!("{}/.claude/skills", home),
        "opencode" => format!("{}/.opencode/skills", home),
        "droid" => format!("{}/.droid/knowledge", home),
        "codex" => format!("{}/.codex/knowledge", home),
        _ => return Ok(false),
    };
    
    // Check if source directory exists and has skills
    let source_path = std::path::Path::new(skills_path);
    if !source_path.exists() {
        return Ok(false);
    }
    
    // Create target directory if needed
    let target_dir = std::path::Path::new(&tool_skills_path);
    if let Some(parent) = target_dir.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    
    // Create symlink (remove existing first)
    let _ = std::fs::remove_file(target_dir);
    match std::os::unix::fs::symlink(source_path, target_dir) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Show agentic tools information for scientific workflows
fn show_agentic_tools_info(i18n: &I18n, tools: &[AgenticToolInfo]) -> Result<()> {
    display::section(&i18n.t("scientific.agentic_tools"));
    
    for tool in tools {
        display::info(&format!("{} - {}", tool.name, i18n.t(tool.description_key)));
        println!("  {} {}", "🎯".dimmed(), i18n.t_with_vars("scientific.agentic_best_for", &[("use", tool.best_for)]));
        
        // Show installation status
        let is_installed = match tool.id {
            "claude_code" => crate::core::shell::is_installed_with_local_bin("claude"),
            "opencode" => crate::core::shell::is_installed_with_local_bin("opencode"),
            "droid" => crate::core::shell::is_installed_with_local_bin("droid"),
            "codex" => crate::core::shell::is_installed_with_local_bin("codex"),
            _ => false,
        };
        if !is_installed {
            display::warning(&format!("{}: {}", i18n.t("tool.not_installed"), get_install_cmd(tool.id)));
        }
        println!();
    }
    
    display::info(&i18n.t("scientific.agentic_hint"));
    display::info(&i18n.t("scientific.press_enter_to_continue"));
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    Ok(())
}

fn get_install_cmd(tool_id: &str) -> &'static str {
    match tool_id {
        "claude_code" => "curl -fsSL https://claude.ai/install.sh | bash",
        "opencode" => "curl -fsSL https://opencode.ai/install | bash",
        "droid" => "curl -fsSL https://app.factory.ai/cli | sh",
        "codex" => "npm install -g @openai/codex",
        _ => "See tool documentation",
    }
}