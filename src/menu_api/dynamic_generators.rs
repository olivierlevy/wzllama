//! Dynamic menu generators for wizard menus
//!
//! These functions generate dynamic submenus on-demand for MenuHandler

use crate::config::I18n;
use crate::core::HardwareInfo;
use crate::menu_api::MenuItem;
use crate::config::WzllamaState;
use crate::tools;

/// Generate dynamic models submenu
pub fn generate_models_menu(i18n: &I18n, state: &WzllamaState, _hw: &HardwareInfo) -> Vec<MenuItem> {
    let mut items = vec![MenuItem::leaf("↩️ Retour")];
    
    // Try to get models from LLM API
    let models = get_available_models_from_api();
    
    for model in models {
        let installed = is_model_installed(&model);
        let icon = if installed { "✅" } else { "📦" };
        let label = format!("{} {}", icon, model);
        items.push(
            MenuItem::leaf(&label)
                .with_action(&format!("select_model_{}", model))
        );
    }
    
    items
}

/// Generate dynamic tools submenu
pub fn generate_tools_menu(i18n: &I18n, state: &WzllamaState, _hw: &HardwareInfo) -> Vec<MenuItem> {
    let mut items = vec![MenuItem::leaf("↩️ Retour")];
    
    let tools_list = tools::get_available_tools(state, i18n);
    
    for tool in tools_list {
        let tool_dyn = tools::get_tool(&tool.id);
        let supports_agentic = tool_dyn.as_ref().map(|x| x.supports_agentic()).unwrap_or(false);
        let icon = if supports_agentic { "🤖" } else if tool.installed { "✅" } else { "📦" };
        let agentic_tag = if supports_agentic { " [agentic]".to_string() } else { String::new() };
        let label = format!("{} {} - {}{}", icon, tool.name, tool.description, agentic_tag);
        
        items.push(
            MenuItem::leaf(&label)
                .with_action(&format!("tool_{}", tool.id))
        );
    }
    
    items
}

/// Generate dynamic use case submenu
pub fn generate_usecase_menu(i18n: &I18n, _state: &WzllamaState, _hw: &HardwareInfo) -> Vec<MenuItem> {
    vec![
        MenuItem::leaf("↩️ Retour"),
        MenuItem::leaf("💻 Coding").with_action("usecase_coding"),
        MenuItem::leaf("🔍 Research").with_action("usecase_research"),
        MenuItem::leaf("📊 Data Science").with_action("usecase_data"),
        MenuItem::leaf("🎨 Creative").with_action("usecase_creative"),
        MenuItem::leaf("💼 Business").with_action("usecase_business"),
    ]
}

/// Generate scientific tools submenu
pub fn generate_scientific_menu(i18n: &I18n, state: &WzllamaState, _hw: &HardwareInfo) -> Vec<MenuItem> {
    let categories = vec![
        ("🧬 Bio", "scientific_bio"),
        ("🔬 Chemistry", "scientific_chemistry"),
        ("🌌 Physics", "scientific_physics"),
        ("📊 Statistics", "scientific_stats"),
    ];
    
    let mut items = vec![MenuItem::leaf("↩️ Retour")];
    
    for (label, action) in categories {
        items.push(MenuItem::leaf(label).with_action(action));
    }
    
    items
}

/// Helper: Get available models from API
fn get_available_models_from_api() -> Vec<String> {
    // Placeholder - would call actual API
    vec![
        "llama3.2:latest".to_string(),
        "qwen2.5:latest".to_string(),
        "mistral:latest".to_string(),
        "gemma2:latest".to_string(),
    ]
}

/// Helper: Check if model is installed
fn is_model_installed(model: &str) -> bool {
    // Placeholder - would check actual installation
    model.ends_with(":latest")
}

/// Helper: Get model description
fn get_model_description(model: &str) -> String {
    match model {
        m if m.contains("llama") => "General purpose model",
        m if m.contains("qwen") => "Qwen series",
        m if m.contains("mistral") => "Mistral AI model",
        m if m.contains("gemma") => "Google Gemma model",
        _ => "Unknown model",
    }.to_string()
}