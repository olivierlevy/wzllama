//! Menu tree structure for hierarchical menu management

use crate::menu_api::menu_item::MenuItem;

/// Root container for the menu hierarchy
#[derive(Debug, Clone)]
pub struct MenuTree {
    /// Root menu item (typically a branch with main categories)
    pub root: MenuItem,
    /// Metadata for the menu tree
    pub metadata: MenuMetadata,
}

/// Metadata for menu tree configuration
#[derive(Debug, Clone, Default)]
pub struct MenuMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
}

impl MenuTree {
    /// Create a new empty menu tree
    pub fn new(root_label: &str) -> Self {
        Self {
            root: MenuItem::branch(root_label),
            metadata: MenuMetadata::default(),
        }
    }

    /// Create a menu tree with a title
    pub fn with_title(root_label: &str, title: &str) -> Self {
        Self {
            root: MenuItem::branch(root_label),
            metadata: MenuMetadata {
                title: Some(title.to_string()),
                ..Default::default()
            },
        }
    }

    /// Set the root menu item
    pub fn with_root(mut self, root: MenuItem) -> Self {
        self.root = root;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: MenuMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Find a menu item by path (e.g., "Install/Logiciel A")
    pub fn find_by_path(&self, path: &str) -> Option<&MenuItem> {
        let parts: Vec<&str> = path.split('/').collect();
        self.find_by_path_parts(&parts, &self.root)
    }

    fn find_by_path_parts<'a>(
        &'a self,
        parts: &[&str],
        current: &'a MenuItem,
    ) -> Option<&'a MenuItem> {
        if parts.is_empty() {
            return Some(current);
        }

        let target = parts[0];
        for submenu in &current.submenus {
            if submenu.label == target {
                return self.find_by_path_parts(&parts[1..], submenu);
            }
        }
        None
    }

    /// Get all leaf items (terminal actions)
    pub fn get_leaf_items(&self) -> Vec<&MenuItem> {
        self.collect_leaves(&self.root)
    }

    fn collect_leaves<'a>(&'a self, item: &'a MenuItem) -> Vec<&'a MenuItem> {
        if item.submenus.is_empty() && item.has_action() {
            vec![item]
        } else if item.submenus.is_empty() {
            vec![]
        } else {
            item.submenus
                .iter()
                .flat_map(|s| self.collect_leaves(s))
                .collect()
        }
    }

    /// Get flat list of all items with their paths
    pub fn get_flat_items(&self) -> Vec<(String, &MenuItem)> {
        self.collect_flat(&self.root, String::new())
    }

    fn collect_flat<'a>(
        &'a self,
        item: &'a MenuItem,
        prefix: String,
    ) -> Vec<(String, &'a MenuItem)> {
        let current_path = if prefix.is_empty() {
            item.label.clone()
        } else {
            format!("{}/{}", prefix, item.label)
        };

        let mut result = vec![(current_path, item)];

        for submenu in &item.submenus {
            let sub_items = self.collect_flat(submenu, item.label.clone());
            result.extend(sub_items);
        }

        result
    }
}

/// Configuration structure for building menus from external sources
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MenuConfig {
    pub version: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub items: Vec<MenuConfigItem>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MenuConfigItem {
    pub label: String,
    #[serde(alias = "action")]
    pub action_id: Option<String>,
    pub children: Option<Vec<MenuConfigItem>>,
    /// Optional condition for showing this item (for dynamic menus)
    #[serde(default)]
    pub condition: Option<String>,
}

impl From<MenuConfig> for MenuTree {
    fn from(config: MenuConfig) -> Self {
        let root = MenuItem::branch("root").add_submenus(
            config
                .items
                .into_iter()
                .map(MenuItem::from_config)
                .collect(),
        );

        Self {
            root,
            metadata: MenuMetadata {
                title: config.title,
                description: config.description,
                version: config.version,
            },
        }
    }
}

impl MenuConfigItem {
    fn into_menu_item(self) -> MenuItem {
        let mut item = if self.children.is_some() || self.action_id.is_none() {
            MenuItem::branch(&self.label)
        } else {
            MenuItem::leaf(&self.label)
        };

        item.action_id = self.action_id;

        if let Some(children) = self.children {
            item = item.add_submenus(children.into_iter().map(MenuItem::from_config).collect());
        }

        item
    }
}

trait FromConfig: Sized {
    fn from_config(item: MenuConfigItem) -> Self;
}

impl FromConfig for MenuItem {
    fn from_config(item: MenuConfigItem) -> Self {
        item.into_menu_item()
    }
}
