//! Dynamic menu provider trait for runtime menu generation
//!
//! Allows menus to be built dynamically based on context (installed tools, available models, etc.)

use crate::menu_api::MenuItem;

/// Trait for providing dynamic menu items
/// 
/// Implement this trait when menu items depend on runtime state like:
/// - Available models from API
/// - Installed tools
/// - User configuration
pub trait DynamicMenuProvider: Send + Sync {
    /// Unique identifier for this provider
    fn id(&self) -> &'static str;
    
    /// Generate menu items based on current context
    fn build_items(&self) -> Vec<MenuItem>;
}

/// Factory for creating dynamic menu providers
pub struct DynamicMenuFactory;

impl DynamicMenuFactory {
    /// Create a menu item with dynamic submenus
    pub fn dynamic_branch<F>(label: &str, provider_factory: F) -> MenuItem
    where
        F: Fn() -> Box<dyn DynamicMenuProvider> + Send + Sync + 'static,
    {
        // Store the factory in label_vars for later resolution
        let mut item = MenuItem::branch(label);
        item.label_vars.insert("__dynamic_factory".to_string(), "true".to_string());
        item
    }
}