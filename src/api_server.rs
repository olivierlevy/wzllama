//! HTTP API Server for wzllama
//!
//! Provides REST API endpoints on port 1133 for:
//! - Menu tree navigation
//! - Tool installation (install/update/uninstall/status/launch)
//! - Model management
//! - Configuration

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use serde_json::Value;
use axum::{
    routing::{get, post, delete},
    Router,
    Json,
    extract::Path,
    response::Html,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{CorsLayer, Any};

use crate::menu_api::api_service::ApiService;
use crate::menu_api::ActionResponse;
use crate::config::{WzllamaState, I18n};

/// API state shared between handlers
#[derive(Clone)]
pub struct ApiState {
    pub shutdown_requested: Arc<AtomicBool>,
}

/// Global shutdown flag for the API server
pub static API_SHUTDOWN: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();

pub fn get_shutdown_flag() -> Arc<AtomicBool> {
    API_SHUTDOWN.get_or_init(|| Arc::new(AtomicBool::new(false))).clone()
}

/// Menu tree response
#[derive(Serialize, Deserialize)]
pub struct MenuTreeResponse {
    pub id: String,
    pub label: String,
    pub action_id: Option<String>,
    pub children: Vec<MenuTreeResponse>,
}

/// Create the API router
pub fn create_router() -> Router {
    let shutdown_flag = get_shutdown_flag();
    
    Router::new()
        // Web UI - serve HTML at root
        .route("/", get(serve_web_ui))
        
        // Menu endpoints
        .route("/api/v1/menu", get(get_menu_tree))
        .route("/api/v1/menu/{id}", get(get_menu_item))
        .route("/api/v1/menu/{id}/select", post(select_menu_item))
        
        // Tool endpoints
        .route("/api/v1/tools", get(list_tools))
        .route("/api/v1/tools/{id}", get(get_tool))
        .route("/api/v1/tools/{id}/install", post(install_tool))
        .route("/api/v1/tools/{id}/update", post(update_tool))
        .route("/api/v1/tools/{id}/uninstall", post(uninstall_tool))
        .route("/api/v1/tools/{id}/status", get(get_tool_status))
        .route("/api/v1/tools/{id}/launch", post(launch_tool))
        
        // Model endpoints
        .route("/api/v1/models", get(list_models))
        .route("/api/v1/models/{name}/pull", post(pull_model))
        .route("/api/v1/models/{name}/delete", delete(delete_model))
        
        // System endpoints
        .route("/api/v1/status", get(get_system_status))
        .route("/api/v1/hardware", get(get_hardware_info))
        
        // Health check
        .route("/health", get(health_check))
        
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(ApiState { shutdown_requested: shutdown_flag })
}

/// Signal the API server to shutdown gracefully
pub fn request_shutdown() {
    if let Some(flag) = API_SHUTDOWN.get() {
        flag.store(true, Ordering::SeqCst);
    }
}

/// Check if shutdown has been requested
pub fn is_shutdown_requested() -> bool {
    API_SHUTDOWN.get()
        .map(|flag| flag.load(Ordering::SeqCst))
        .unwrap_or(false)
}

/// Start the API server with graceful shutdown support
pub async fn start_server(addr: SocketAddr) {
    let app = create_router();
    
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("🚀 wzllama API server listening on http://{}", addr);
            
            // Create shutdown signal
            let shutdown_flag = get_shutdown_flag();
            
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    // Wait for shutdown signal
                    while !shutdown_flag.load(Ordering::SeqCst) {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                })
                .await
                .ok();
                
            println!("👋 API server shutdown gracefully");
        }
        Err(e) => {
            eprintln!("❌ Failed to bind to {}: {}", addr, e);
            eprintln!("💡 Try killing any existing process on port 1133:");
            eprintln!("   sudo lsof -ti:1133 | xargs kill -9");
            eprintln!("💡 Or use a different port:");
            eprintln!("   wzllama serve --port 1134");
            std::process::exit(1);
        }
    }
}

// ============ Handler implementations ============

async fn health_check() -> &'static str {
    "OK"
}

async fn get_menu_tree() -> Json<Value> {
    let state = ApiService::get_state();
    let lang = state.language.clone().unwrap_or_else(|| "en".to_string());
    let i18n = ApiService::get_i18n(Some(&lang));
    Json(ApiService::get_menu_structure(&i18n, &state))
}

async fn get_menu_item(
    Path(id): Path<String>,
) -> Json<Value> {
    let state = ApiService::get_state();
    let lang = state.language.clone().unwrap_or_else(|| "en".to_string());
    let i18n = ApiService::get_i18n(Some(&lang));
    
    // Get the main menu structure
    let menu = ApiService::get_menu_structure(&i18n, &state);
    
    // Check if the item has children
    if let Some(items) = menu.get("items").and_then(|i| i.as_array()) {
        if let Some(item) = items.iter().find(|i| i.get("id").and_then(|v| v.as_str()) == Some(&id)) {
            if let Some(children) = item.get("children") {
                // Return submenu with children
                return Json(serde_json::json!({
                    "id": id,
                    "label": item.get("label").and_then(|v| v.as_str()).unwrap_or(&id),
                    "type": "menu",
                    "items": children
                }));
            }
        }
    }
    
    Json(serde_json::json!({ "id": id, "label": "Menu Item", "type": "item", "items": [] }))
}

/// Execute an action by ID (for menu items and tool actions)
async fn select_menu_item(
    Path(id): Path<String>,
) -> Json<Value> {
    // Map menu item IDs to actual actions
    let result = match id.as_str() {
        "wizard" | "models" | "tools" | "scientific" | "cleanup" | "config" | "language" => {
            // For submenu items, return submenu action
            serde_json::json!({ "action": "submenu", "target": id })
        }
        "quit" => serde_json::json!({ "action": "quit" }),
        "resume_last" => serde_json::json!({ "action": "resume" }),
        _ => {
            // Check if it's a tool action
            if id.starts_with("install_tool_") {
                let tool_id = id.strip_prefix("install_tool_").unwrap_or(&id);
                serde_json::json!({ "action": "install_tool", "tool_id": tool_id })
            } else if id.starts_with("launch_tool_") {
                let tool_id = id.strip_prefix("launch_tool_").unwrap_or(&id);
                serde_json::json!({ "action": "launch_tool", "tool_id": tool_id })
            } else if id.starts_with("select_model_") {
                let model_name = id.strip_prefix("select_model_").unwrap_or(&id);
                serde_json::json!({ "action": "select_model", "model": model_name })
            } else if id.starts_with("usecase_") {
                // Use case selection - return as submenu action
                serde_json::json!({ "action": "submenu", "target": id })
            } else {
                serde_json::json!({ "selected": id, "info": "Action registered" })
            }
        }
    };
    Json(result)
}

async fn list_tools() -> Json<Value> {
    let state = ApiService::get_state();
    let lang = state.language.clone().unwrap_or_else(|| "en".to_string());
    let i18n = ApiService::get_i18n(Some(&lang));
    Json(ApiService::get_tools_menu(&i18n, &state))
}

async fn get_tool(
    Path(id): Path<String>,
) -> Json<Value> {
    let state = ApiService::get_state();
    let i18n = I18n::default();
    
    if let Some(tool_info) = ApiService::get_tool(&id, &i18n, &state) {
        Json(serde_json::json!({
            "id": tool_info.id,
            "name": tool_info.name,
            "description": tool_info.description,
            "installed": tool_info.installed,
            "status": tool_info.status,
            "supports_agentic": tool_info.supports_agentic,
            "requires_docker": tool_info.requires_docker,
        }))
    } else {
        Json(serde_json::json!({
            "id": id,
            "name": "Unknown",
            "description": "Tool not found",
            "installed": false,
            "status": "not_found",
            "supports_agentic": false,
            "requires_docker": false,
        }))
    }
}

async fn install_tool(
    Path(id): Path<String>,
) -> Json<ActionResponse> {
    let result = ApiService::install_tool(&id, &I18n::default()).unwrap_or(ActionResponse {
        success: false,
        message: "Installation failed".to_string(),
    });
    Json(result)
}

async fn update_tool(
    Path(id): Path<String>,
) -> Json<ActionResponse> {
    let result = ApiService::update_tool(&id, &I18n::default()).unwrap_or(ActionResponse {
        success: false,
        message: "Update failed".to_string(),
    });
    Json(result)
}

async fn uninstall_tool(
    Path(id): Path<String>,
) -> Json<ActionResponse> {
    let result = ApiService::uninstall_tool(&id, &I18n::default()).unwrap_or(ActionResponse {
        success: false,
        message: "Uninstall failed".to_string(),
    });
    Json(result)
}

async fn get_tool_status(
    Path(id): Path<String>,
) -> Json<Value> {
    let state = ApiService::get_state();
    let installed = ApiService::is_tool_installed(&id, &state);
    
    Json(serde_json::json!({
        "id": id,
        "installed": installed,
        "status": if installed { "installed" } else { "not_installed" }
    }))
}

async fn launch_tool(
    Path(id): Path<String>,
) -> Json<ActionResponse> {
    // Launch is interactive in CLI mode - in API mode we return info
    Json(ActionResponse {
        success: true,
        message: format!("To launch {} interactively, use wzllama wizard or wzllama tools menu", id),
    })
}

async fn list_models() -> Json<Value> {
    let state = ApiService::get_state();
    let lang = state.language.clone().unwrap_or_else(|| "en".to_string());
    let i18n = ApiService::get_i18n(Some(&lang));
    Json(ApiService::get_models_menu(&i18n, &state))
}

async fn pull_model(
    Path(name): Path<String>,
) -> Json<ActionResponse> {
    // Pull is interactive - return info
    Json(ActionResponse {
        success: true,
        message: format!("To pull model {}, run: ollama pull {}", name, name),
    })
}

async fn delete_model(
    Path(name): Path<String>,
) -> Json<ActionResponse> {
    // Delete is interactive - return info
    Json(ActionResponse {
        success: true,
        message: format!("To delete model {}, run: ollama rm {}", name, name),
    })
}

async fn get_system_status() -> Json<Value> {
    let status = ApiService::get_system_status();
    Json(serde_json::json!({
        "status": status.status,
        "ollama": status.ollama
    }))
}

async fn get_hardware_info() -> Json<Value> {
    let hw = ApiService::get_hardware_info();
    Json(serde_json::json!({
        "ram_gb": hw.ram_gb,
        "has_gpu": hw.has_gpu,
        "gpus": hw.gpus.iter().map(|g| serde_json::json!({
            "name": g.name,
            "vram_mb": g.vram_mb
        })).collect::<Vec<_>>()
    }))
}

/// Serve the web UI HTML page
async fn serve_web_ui() -> Html<String> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>wzllama - AI Tools Manager</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { 
            font-family: 'Segoe UI', 'Monaco', 'Menlo', monospace;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #eee; min-height: 100vh; padding: 20px;
        }
        .container { max-width: 900px; margin: 0 auto; }
        .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
        h1 { color: #4ecca3; }
        .hardware-info { background: #16213e; border-radius: 12px; padding: 15px; margin-bottom: 20px; font-size: 14px; }
        .hardware-info span { margin-right: 20px; }
        
        /* Breadcrumb navigation */
        .breadcrumb { 
            display: flex; align-items: center; gap: 8px; 
            margin-bottom: 15px; padding: 10px 15px; 
            background: #0f172a; border-radius: 8px; font-size: 14px;
        }
        .breadcrumb-item { color: #888; cursor: pointer; }
        .breadcrumb-item:hover { color: #4ecca3; }
        .breadcrumb-item.active { color: #4ecca3; font-weight: bold; }
        .breadcrumb-separator { color: #555; }
        
        .menu { background: #16213e; border-radius: 12px; padding: 20px; margin-bottom: 20px; }
        .menu h2 { color: #4ecca3; margin-bottom: 15px; border-bottom: 1px solid #333; padding-bottom: 10px; }
        .menu-item { 
            padding: 12px 16px; margin: 8px 0; background: #1a1a2e; border-radius: 8px;
            cursor: pointer; transition: all 0.2s; border-left: 3px solid transparent;
            display: flex; justify-content: space-between; align-items: center;
        }
        .menu-item:hover { background: #1f2a44; border-left-color: #4ecca3; }
        .menu-item.agentic { border-left-color: #ff6b6b; }
        .menu-item.installed { opacity: 1; }
        .menu-item:not(.installed) { opacity: 0.6; }
        .back-btn { background: #333; padding: 10px 20px; border-radius: 8px; cursor: pointer; }
        .back-btn:hover { background: #444; }
        .loading { text-align: center; padding: 40px; }
        .status { text-align: center; padding: 10px; color: #888; font-size: 14px; }
        .badge { padding: 2px 8px; border-radius: 4px; font-size: 12px; }
        .badge.installed { background: #4ecca3; color: #1a1a2e; }
        .badge.agentic { background: #ff6b6b; color: white; }
        .badge.children { background: #334155; color: #aaa; font-size: 11px; }
        .menu-subtitle { color: #888; font-size: 13px; margin-bottom: 15px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🦙 wzllama</h1>
            <div id="connection-status">🟢</div>
        </div>
        
        <div class="hardware-info" id="hardware-info">
            Loading hardware info...
        </div>
        
        <div class="breadcrumb" id="breadcrumb">
            <span class="breadcrumb-item active">Menu</span>
        </div>
        
        <div id="menu-container"></div>
        <div id="tools-container" style="display:none;"></div>
        <div id="models-container" style="display:none;"></div>
        
        <div class="status" id="status">Loading...</div>
    </div>
    
    <script>
        let menuStack = [];
        
        async function loadHardwareInfo() {
            try {
                const res = await fetch('/api/v1/hardware');
                const data = await res.json();
                const hwHtml = `💾 RAM: ${data.ram_gb}GB | 🎮 GPU: ${data.has_gpu ? 'Yes' : 'No'}`;
                document.getElementById('hardware-info').innerHTML = hwHtml;
            } catch (e) {
                document.getElementById('hardware-info').innerHTML = 'Hardware info unavailable';
            }
        }
        
        async function loadMenu(path = '', label = 'Menu') {
            try {
                const url = path ? `/api/v1/menu/${path}` : '/api/v1/menu';
                const res = await fetch(url);
                const data = await res.json();
                menuStack.push({ path, label, data });
                updateBreadcrumb();
                renderMenu(data, label);
            } catch (e) {
                document.getElementById('status').textContent = 'Error loading menu';
            }
        }
        
        function updateBreadcrumb() {
            const breadcrumb = document.getElementById('breadcrumb');
            let html = '';
            menuStack.forEach((item, index) => {
                const isLast = index === menuStack.length - 1;
                html += `<span class="breadcrumb-item ${isLast ? 'active' : ''}" onclick="goToBreadcrumb(${index})">${item.label}</span>`;
                if (!isLast) html += `<span class="breadcrumb-separator">›</span>`;
            });
            breadcrumb.innerHTML = html || '<span class="breadcrumb-item active">Menu</span>';
        }
        
        function goToBreadcrumb(index) {
            if (index < menuStack.length - 1) {
                menuStack = menuStack.slice(0, index + 1);
                updateBreadcrumb();
                renderMenu(menuStack[menuStack.length - 1].data, menuStack[menuStack.length - 1].label);
            }
        }
        
        async function loadTools() {
            try {
                const res = await fetch('/api/v1/tools');
                const data = await res.json();
                let html = '<div class="menu"><h2>🛠 Tools</h2>';
                data.forEach(item => {
                    const badges = [];
                    if (item.installed) badges.push('<span class="badge installed">installed</span>');
                    if (item.supports_agentic) badges.push('<span class="badge agentic">agentic</span>');
                    html += `<div class="menu-item" onclick="executeAction('${item.id}', 'install')">
                        ${item.name} ${badges.join('')}
                    </div>`;
                });
                html += '</div>';
                document.getElementById('tools-container').innerHTML = html;
            } catch (e) {
                document.getElementById('tools-container').innerHTML = '<div class="menu">Error loading tools</div>';
            }
        }
        
        async function loadModels() {
            try {
                const res = await fetch('/api/v1/models');
                const data = await res.json();
                let html = '<div class="menu"><h2>🤖 Models</h2>';
                if (data.items) {
                    data.items.forEach(item => {
                        html += `<div class="menu-item" onclick="executeAction('${item.id}', 'select')">
                            ${item.label}
                        </div>`;
                    });
                }
                html += '</div>';
                document.getElementById('models-container').innerHTML = html;
            } catch (e) {
                document.getElementById('models-container').innerHTML = '<div class="menu">Error loading models</div>';
            }
        }
        
        function renderMenu(data, label = 'Menu') {
            document.getElementById('status').textContent = label;
            
            let html = `<div class="menu"><h2>${label}</h2>`;
            
            // Add description/subtitle if available
            if (data.description) {
                html += `<div class="menu-subtitle">${data.description}</div>`;
            }
            
            if (menuStack.length > 1) {
                html += `<div class="menu-item back-btn" onclick="goBack()">↩️ Retour</div>`;
            }
            
            if (data.items && data.items.length > 0) {
                data.items.forEach(item => {
                    const classes = [];
                    if (item.installed) classes.push('installed');
                    if (item.agentic) classes.push('agentic');
                    if (item.children && item.children.length > 0) classes.push('has-children');
                    
                    const childrenCount = item.children ? item.children.length : 0;
                    const childBadge = childrenCount > 0 ? `<span class="badge children">${childrenCount}</span>` : '';
                    
                    html += `<div class="menu-item ${classes.join(' ')}" onclick="selectItem('${item.id}', '${item.label.replace(/'/g, "\\'")}')">
                        <span>${item.label}</span>${childBadge}
                    </div>`;
                });
            }
            
            html += '</div>';
            document.getElementById('menu-container').innerHTML = html;
        }
        
        async function selectItem(id, label) {
            // First check if the item has children in the current menu data
            const currentMenu = menuStack[menuStack.length - 1].data;
            const item = currentMenu.items?.find(i => i.id === id);
            
            if (item && item.children && item.children.length > 0) {
                // Navigate into submenu using children data from API
                menuStack.push({ path: id, label: label, data: { label: label, items: item.children } });
                updateBreadcrumb();
                renderMenu({ label: label, items: item.children }, label);
                return;
            }
            
            // Try to fetch submenu from API if no local children
            try {
                const submenuRes = await fetch(`/api/v1/menu/${id}`);
                const submenuData = await submenuRes.json();
                if (submenuData.items && submenuData.items.length > 0) {
                    menuStack.push({ path: id, label: label, data: submenuData });
                    updateBreadcrumb();
                    renderMenu(submenuData, label);
                    return;
                }
            } catch (e) {
                // Continue to action handling
            }
            
            // Otherwise use the API endpoint for terminal actions
            const res = await fetch(`/api/v1/menu/${id}/select`, { method: 'POST' });
            const data = await res.json();
            
            if (data.action === 'install_tool') {
                await installTool(data.tool_id);
            } else if (data.action === 'launch_tool') {
                await launchTool(data.tool_id);
            } else if (data.action === 'select_model') {
                alert(`Selected model: ${data.model}`);
            } else if (data.action === 'quit') {
                alert('Goodbye!');
            } else if (data.action === 'submenu') {
                // Load submenu dynamically
                try {
                    const submenuRes = await fetch(`/api/v1/menu/${data.target}`);
                    const submenuData = await submenuRes.json();
                    if (submenuData.items && submenuData.items.length > 0) {
                        menuStack.push({ path: data.target, label: label, data: submenuData });
                        updateBreadcrumb();
                        renderMenu(submenuData, label);
                    }
                } catch (e) {
                    alert(`Could not load submenu: ${data.target}`);
                }
            } else {
                alert(`Selected: ${id}`);
            }
        }
        
        async function executeAction(id, action) {
            if (action === 'install') {
                await installTool(id);
            } else if (action === 'select') {
                alert(`Selected model: ${id}`);
            }
        }
        
        async function installTool(toolId) {
            const res = await fetch(`/api/v1/tools/${toolId}/install`, { method: 'POST' });
            const data = await res.json();
            alert(data.message || `Installing ${toolId}`);
            loadTools();
        }
        
        async function launchTool(toolId) {
            const res = await fetch(`/api/v1/tools/${toolId}/launch`, { method: 'POST' });
            const data = await res.json();
            alert(data.message || `Launching ${toolId}`);
        }
        
        function goBack() {
            if (menuStack.length > 1) {
                menuStack.pop();
                updateBreadcrumb();
                const prev = menuStack[menuStack.length - 1];
                renderMenu(prev.data, prev.label);
            }
        }
        
        function showTab(tab) {
            document.getElementById('menu-container').style.display = tab === 'menu' ? 'block' : 'none';
            document.getElementById('tools-container').style.display = tab === 'tools' ? 'block' : 'none';
            document.getElementById('models-container').style.display = tab === 'models' ? 'block' : 'none';
            
            if (tab === 'tools') loadTools();
            if (tab === 'models') loadModels();
        }
        
        // Load initial data
        loadHardwareInfo();
        loadMenu();
    </script>
</body>
</html>"#.to_string())
}