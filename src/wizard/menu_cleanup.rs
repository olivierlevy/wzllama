//! Cleanup menu - migrated to use menu_api
//!
//! This menu uses the MenuTree/MenuHandler system with ToolAction dispatch.

use anyhow::Result;
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{
    MenuTree, MenuItem, MenuMetadata,
    enter_alternate_screen, exit_alternate_screen,
};

/// Create the cleanup menu tree structure
pub fn build_menu_tree() -> MenuTree {
    let root = MenuItem::branch("cleanup")
        .add_submenu(MenuItem::leaf("↩️ Retour"))
        .add_submenu(MenuItem::leaf("🧹 Nettoyer les outils").with_action("cleanup_tools"))
        .add_submenu(MenuItem::leaf("🧹 Nettoyer les modèles").with_action("cleanup_models"));
    
    MenuTree::new("cleanup")
        .with_metadata(MenuMetadata {
            title: Some("🧹 Nettoyage".to_string()),
            ..Default::default()
        })
        .with_root(root)
}

/// Run the cleanup menu using menu_api system
/// 
/// This is a hybrid approach - we create the MenuTree for the new system
/// but still delegate to the existing wizard functions for actual execution.
/// This maintains backward compatibility while migrating to the new architecture.
pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    use dialoguer::Select;
    use crate::wizard::{cleanup_tools, cleanup_models};
    use super::menu_header;
    
    enter_alternate_screen();
    
    loop {
        // Render header
        menu_header::render(
            i18n,
            "menu.main.cleanup",
            true,
            state.last_model.as_deref(),
            hw.ram_gb,
            hw.total_vram_mb as f64 / 1024.0
        );
        
        // Build menu items from MenuTree for consistent display
        let tree = build_menu_tree();
        let items: Vec<String> = tree.root.submenus.iter()
            .map(|m| m.label.clone())
            .collect();

        let sel = match Select::new()
            .with_prompt(i18n.t("cleanup.choose"))
            .items(&items)
            .default(0)
            .max_length(15)
            .interact_opt()? {
            Some(s) => s,
            None => break, // Escape pressed
        };

        match sel {
            0 => break,  // Retour
            1 => cleanup_tools::run(i18n, state)?,
            2 => cleanup_models::run(i18n, state)?,
            _ => break,
        }
    }
    
    exit_alternate_screen();
    Ok(())
}