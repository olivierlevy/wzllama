//! Menu item definition with hierarchical structure support

use std::collections::HashMap;

/// A menu item that can be either a leaf (with action) or a branch (has submenus)
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Display label for this menu item
    pub label: String,
    /// Optional action to execute when selected
    pub action_id: Option<String>,
    /// Child menu items (submenus)
    pub submenus: Vec<MenuItem>,
    /// Dynamic label parts (for interpolation)
    pub label_vars: HashMap<String, String>,
}

impl MenuItem {
    /// Create a new leaf menu item (no submenus)
    pub fn leaf(label: &str) -> Self {
        Self {
            label: label.to_string(),
            action_id: None,
            submenus: Vec::new(),
            label_vars: HashMap::new(),
        }
    }

    /// Create a new branch menu item (has submenus)
    pub fn branch(label: &str) -> Self {
        Self {
            label: label.to_string(),
            action_id: None,
            submenus: Vec::new(),
            label_vars: HashMap::new(),
        }
    }

    /// Set the action ID for this menu item (from &str)
    pub fn with_action(mut self, action_id: &str) -> Self {
        self.action_id = Some(action_id.to_string());
        self
    }

    /// Set the action ID for this menu item (from String)
    pub fn with_action_string(mut self, action_id: String) -> Self {
        self.action_id = Some(action_id);
        self
    }

    /// Add a submenu item
    pub fn add_submenu(mut self, item: MenuItem) -> Self {
        self.submenus.push(item);
        self
    }

    /// Add multiple submenu items
    pub fn add_submenus(mut self, items: Vec<MenuItem>) -> Self {
        self.submenus.extend(items);
        self
    }

    /// Check if this is a leaf node (no submenus)
    pub fn is_leaf(&self) -> bool {
        self.submenus.is_empty()
    }

    /// Check if this item has an action
    pub fn has_action(&self) -> bool {
        self.action_id.is_some()
    }

    /// Get formatted label with variable interpolation
    pub fn formatted_label(&self) -> String {
        let mut label = self.label.clone();
        for (key, value) in &self.label_vars {
            label = label.replace(&format!("{{{}}}", key), value);
        }
        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_creation() {
        let item = MenuItem::leaf("Test");
        assert!(item.is_leaf());
        assert!(!item.has_action());

        let item = MenuItem::leaf("Test").with_action("test_action");
        assert!(item.has_action());
    }
}
