//! API-First Menu System
//!
//! This provides a MenuTree-based view over the existing wizard functions.
//! The wizard/*.rs files remain as the implementation, but menus are now
//! represented as MenuTree for API consumption.

use crate::config::{I18n, WzllamaState};
use crate::menu_api::wizard_helpers::{UseCase, get_priority_tools_for_usecase, ScientificCategory, AgenticToolInfo};
use crate::menu_api::api_service::ApiService;

/// Get the menu tree structure for API consumption - main menu with hierarchical structure
pub fn get_menu_structure(i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    let has_resume = state.last_tool.is_some() && state.last_model.is_some();
    
    let mut items = vec![];
    
    // Resume option
    if has_resume {
        if let Some(ref last_tool) = state.last_tool {
            if let Some(tool) = crate::tools::get_tool(last_tool) {
                items.push(serde_json::json!({
                    "id": "resume",
                    "label": format!("▶ Reprendre {}", tool.name()),
                    "action_id": "resume_last",
                    "type": "item",
                    "icon": "▶"
                }));
            }
        }
    }
    
    // Main menu items with hierarchical structure
    let main_items = vec![
        ("wizard", "menu.main.wizard", "🧙"),
        ("models", "menu.main.models", "🤖"),
        ("scientific", "menu.main.scientific", "🔬"),
        ("tools", "menu.main.tools", "🛠️"),
        ("cleanup", "menu.main.cleanup", "🧹"),
        ("config", "menu.main.config", "⚙️"),
        ("language", "menu.main.language", "🌍"),
        ("quit", "menu.main.quit", "❌"),
    ];
    
    for (id, label_key, icon) in main_items {
        let item = if id == "tools" || id == "wizard" || id == "scientific" {
            // These have submenus
            serde_json::json!({
                "id": id,
                "label": i18n.t(label_key),
                "action_id": id,
                "type": "submenu",
                "icon": icon,
                "children": get_submenu_items(id, i18n, state)
            })
        } else {
            serde_json::json!({
                "id": id,
                "label": i18n.t(label_key),
                "action_id": id,
                "type": "item",
                "icon": icon
            })
        };
        items.push(item);
    }
    
    serde_json::json!({
        "id": "main",
        "label": i18n.t("menu.main.title"),
        "type": "menu",
        "items": items
    })
}

/// Get submenu items for a specific menu section
fn get_submenu_items(menu_id: &str, i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    match menu_id {
        "wizard" => get_wizard_submenu(i18n, state),
        "tools" => get_tools_submenu(i18n, state),
        "scientific" => get_scientific_submenu(i18n, state),
        "models" => get_models_submenu(i18n, state),
        "cleanup" => get_cleanup_submenu(i18n),
        "config" => get_config_submenu(i18n),
        "language" => get_language_submenu(i18n),
        _ => serde_json::json!([])
    }
}

/// Get cleanup submenu
fn get_cleanup_submenu(i18n: &I18n) -> serde_json::Value {
    serde_json::json!(vec![
        serde_json::json!({"id": "cleanup_tools", "label": i18n.t("cleanup.menu_tools"), "action_id": "cleanup_tools", "type": "item"}),
        serde_json::json!({"id": "cleanup_models", "label": i18n.t("cleanup.menu_models"), "action_id": "cleanup_models", "type": "item"}),
    ])
}

/// Get config submenu
fn get_config_submenu(i18n: &I18n) -> serde_json::Value {
    serde_json::json!(vec![
        serde_json::json!({"id": "edit_performance", "label": i18n.t("config.performance"), "action_id": "edit_performance", "type": "item"}),
        serde_json::json!({"id": "edit_ollama_settings", "label": i18n.t("config.ollama_settings"), "action_id": "edit_ollama_settings", "type": "item"}),
        serde_json::json!({"id": "edit_providers", "label": i18n.t("config.providers"), "action_id": "edit_providers", "type": "item"}),
        serde_json::json!({"id": "edit_openclaw", "label": i18n.t("config.openclaw"), "action_id": "edit_openclaw", "type": "item"}),
        serde_json::json!({"id": "manage_shells", "label": i18n.t("config.shells"), "action_id": "manage_shells", "type": "item"}),
        serde_json::json!({"id": "regenerate_env", "label": i18n.t("config.regenerate_env"), "action_id": "regenerate_env", "type": "item"}),
        serde_json::json!({"id": "uninstall_wzllama", "label": i18n.t("config.uninstall_wzllama"), "action_id": "uninstall_wzllama", "type": "item"}),
    ])
}

/// Get language submenu
fn get_language_submenu(_i18n: &I18n) -> serde_json::Value {
    use crate::config::i18n;
    let languages = i18n::get_available_languages();
    // Flags mapped manually since not in LanguageMeta
    let flags = std::collections::HashMap::from([
        ("fr", "🇫🇷"), ("en", "🇬🇧"), ("es", "🇪🇸"), ("de", "🇩🇪"), ("it", "🇮🇹"),
        ("pt", "🇵🇹"), ("nl", "🇳🇱"), ("ru", "🇷🇺"), ("zh", "🇨🇳"), ("ja", "🇯🇵"),
        ("ko", "🇰🇷"), ("ar", "🇦🇷"), ("hi", "🇮🇳"), ("tr", "🇹🇷"), ("pl", "🇵🇱"),
    ]);
    serde_json::json!(languages.iter().map(|lang| {
        let flag = flags.get(lang.code.as_str()).copied().unwrap_or("🌍");
        serde_json::json!({
            "id": format!("set_language_{}", lang.code),
            "label": format!("{} {}", flag, lang.name),
            "action_id": format!("set_language_{}", lang.code),
            "type": "item"
        })
    }).collect::<Vec<_>>())
}

/// Get tools submenu
fn get_tools_submenu(i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    use crate::tools;
    
    let tools_list = tools::get_available_tools(state, i18n);
    
    let items: Vec<_> = tools_list.iter().map(|t| {
        let tool_dyn = tools::get_tool(&t.id);
        let supports_agentic = tool_dyn.as_ref().map(|x| x.supports_agentic()).unwrap_or(false);
        let icon = if supports_agentic { "🤖" } else if t.installed { "✅" } else { "📦" };
        
        serde_json::json!({
            "id": t.id,
            "label": format!("{} {} - {}", icon, t.name, t.description),
            "action_id": if t.installed { format!("launch_tool_{}", t.id) } else { format!("install_tool_{}", t.id) },
            "type": "item",
            "installed": t.installed,
            "agentic": supports_agentic
        })
    }).collect();
    
    serde_json::json!(items)
}

/// Get wizard submenu with use cases
fn get_wizard_submenu(i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    let use_cases = vec![
        ("usage.coding", UseCase::Coding, "💻"),
        ("usage.chat", UseCase::Chat, "💬"),
        ("usage.reasoning", UseCase::Reasoning, "🤔"),
        ("usage.embedding", UseCase::Embedding, "🔍"),
        ("usage.multimodal", UseCase::Multimodal, "🎨"),
    ];
    
    let items: Vec<_> = use_cases.iter().map(|(key, usecase, icon)| {
        serde_json::json!({
            "id": format!("usecase_{:?}", usecase),
            "label": i18n.t(key),
            "action_id": format!("usecase_{:?}", usecase),
            "type": "submenu",
            "icon": icon,
            "children": get_tools_for_usecase(*usecase, i18n, state)
        })
    }).collect();
    
    serde_json::json!(items)
}

/// Get tools for a specific use case
fn get_tools_for_usecase(usecase: UseCase, i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    let tool_ids = get_priority_tools_for_usecase(usecase, state);
    let agentic_tools = AgenticToolInfo::all();
    
    let items: Vec<_> = tool_ids.iter().map(|tool_id| {
        let installed = ApiService::is_tool_installed(tool_id, state);
        let tool_info = agentic_tools.iter().find(|t| t.id == tool_id);
        let icon = if installed { "✅" } else { "📦" };
        let agentic_tag = if tool_info.map(|t| t.id).is_some() { " 🤖" } else { "" };
        let name = tool_info.map(|t| t.name).unwrap_or(tool_id.as_str());
        
        serde_json::json!({
            "id": format!("launch_tool_{}", tool_id),
            "label": format!("{} {}{}", icon, name, agentic_tag),
            "action_id": if installed { format!("launch_tool_{}", tool_id) } else { format!("install_tool_{}", tool_id) },
            "type": "item",
            "installed": installed,
            "agentic": tool_info.is_some()
        })
    }).collect();
    
    serde_json::json!(items)
}

/// Get scientific submenu with categories
fn get_scientific_submenu(i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    let categories = ScientificCategory::all();
    
    let items: Vec<_> = categories.iter().map(|cat| {
        serde_json::json!({
            "id": cat.name_key,
            "label": i18n.t(cat.name_key),
            "action_id": format!("scientific_{}", cat.name_key.replace(".", "_")),
            "type": "submenu",
            "icon": "🧬",
            "children": get_scientific_tools(cat, i18n, state)
        })
    }).collect();
    
    serde_json::json!(items)
}

/// Get tools for a scientific category
fn get_scientific_tools(cat: &ScientificCategory, i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    // Placeholder - would get actual tools for category
    let skills = cat.skills;
    
    let items: Vec<_> = skills.iter().map(|id| {
        serde_json::json!({
            "id": format!("scientific_tool_{}", id),
            "label": id,
            "action_id": format!("scientific_tool_{}", id),
            "type": "item"
        })
    }).collect();
    
    serde_json::json!(items)
}

/// Get models submenu
fn get_models_submenu(i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    use crate::core::ollama_api;
    
    let local_models = ollama_api::get_models();
    
    let items: Vec<_> = local_models.iter().map(|m| {
        let default_marker = if Some(m.name.clone()) == state.last_model { " (default)" } else { "" };
        let size_info = m.details.as_ref()
            .and_then(|d| d.parameter_size.as_deref())
            .unwrap_or("");
        
        serde_json::json!({
            "id": format!("select_model_{}", m.name),
            "label": format!("{} [{}]{}", m.name, size_info, default_marker),
            "action_id": format!("select_model_{}", m.name),
            "type": "item",
            "size": m.size,
            "installed": true
        })
    }).collect();
    
    serde_json::json!(items)
}

/// Get tools menu with installed status
pub fn get_tools_menu(i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    use crate::tools;
    
    let tools_list = tools::get_available_tools(state, i18n);
    
    let items: Vec<_> = tools_list.iter().map(|t| {
        let tool_dyn = tools::get_tool(&t.id);
        let supports_agentic = tool_dyn.as_ref().map(|x| x.supports_agentic()).unwrap_or(false);
        let icon = if supports_agentic { "🤖" } else if t.installed { "✅" } else { "📦" };
        let agentic_tag = if supports_agentic { " [agentic]" } else { "" };
        
        serde_json::json!({
            "id": t.id,
            "label": format!("{} {} - {}{}", icon, t.name, t.description, agentic_tag),
            "action_id": if t.installed { format!("launch_tool_{}", t.id) } else { format!("install_tool_{}", t.id) },
            "installed": t.installed,
            "agentic": supports_agentic
        })
    }).collect();
    
    serde_json::json!({
        "id": "tools",
        "label": i18n.t("menu.main.tools"),
        "type": "menu",
        "items": items
    })
}

/// Get models menu with local installed models
pub fn get_models_menu(i18n: &I18n, state: &WzllamaState) -> serde_json::Value {
    use crate::core::ollama_api;
    
    let local_models = ollama_api::get_models();
    
    let items: Vec<_> = local_models.iter().map(|m| {
        let default_marker = if Some(m.name.clone()) == state.last_model { " (default)" } else { "" };
        let size_info = m.details.as_ref()
            .and_then(|d| d.parameter_size.as_deref())
            .unwrap_or("");
        
        serde_json::json!({
            "id": m.name,
            "label": format!("{} [{}]{}", m.name, size_info, default_marker),
            "action_id": format!("select_model_{}", m.name),
            "size": m.size
        })
    }).collect();
    
    serde_json::json!({
        "id": "models",
        "label": i18n.t("menu.main.models"),
        "type": "menu",
        "items": items
    })
}