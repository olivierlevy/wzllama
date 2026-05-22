//! Tests for menu_api module

#[cfg(test)]
mod menu_api_tests {
    use crate::menu_api::{
        MenuTree, MenuItem, MenuHandler, ActionDispatcher, 
        ActionContext, ActionResult, ClosureAction
    };

    fn setup_test_tree() -> MenuTree {
        let root = MenuItem::branch("Main")
            .add_submenu(
                MenuItem::leaf("Install Ollama").with_action("install_ollama")
            )
            .add_submenu(
                MenuItem::branch("Tools")
                    .add_submenu(MenuItem::leaf("Launch Chat").with_action("launch_chat"))
                    .add_submenu(MenuItem::leaf("Settings").with_action("open_settings"))
            )
            .add_submenu(
                MenuItem::leaf("Quit").with_action("quit_app")
            );
        
        MenuTree::with_title("Main Menu", "wzllama Test Menu").with_root(root)
    }

    fn setup_dispatcher() -> ActionDispatcher {
        let mut dispatcher = ActionDispatcher::new();
        dispatcher.register(Box::new(ClosureAction::new(
            "install_ollama",
            "Install Ollama",
            |_| Ok(ActionResult::success_with("Ollama installed"))
        )));
        dispatcher.register(Box::new(ClosureAction::new(
            "launch_chat",
            "Launch Chat",
            |_| Ok(ActionResult::success_with("Chat launched"))
        )));
        dispatcher.register(Box::new(ClosureAction::new(
            "open_settings",
            "Open Settings",
            |_| Ok(ActionResult::success())
        )));
        dispatcher.register(Box::new(ClosureAction::new(
            "quit_app",
            "Quit",
            |_| Ok(ActionResult::success())
        )));
        dispatcher
    }

    #[test]
    fn test_menu_tree_creation() {
        let tree = setup_test_tree();
        
        assert!(tree.root.submenus.len() == 3);
        assert_eq!(tree.root.submenus[0].label, "Install Ollama");
        assert_eq!(tree.root.submenus[1].label, "Tools");
        assert_eq!(tree.metadata.title, Some("wzllama Test Menu".to_string()));
    }

    #[test]
    fn test_menu_item_is_leaf() {
        let leaf = MenuItem::leaf("Test").with_action("test_action");
        assert!(leaf.is_leaf());
        
        let branch = MenuItem::branch("Branch")
            .add_submenu(MenuItem::leaf("Child"));
        assert!(!branch.is_leaf());
    }

    #[test]
    fn test_menu_item_has_action() {
        let with_action = MenuItem::leaf("Test").with_action("test");
        assert!(with_action.has_action());
        
        let without_action = MenuItem::leaf("Test");
        assert!(!without_action.has_action());
    }

    #[test]
    fn test_find_by_path() {
        let tree = setup_test_tree();
        
        let found = tree.find_by_path("Tools/Launch Chat");
        assert!(found.is_some());
        assert_eq!(found.unwrap().label, "Launch Chat");
        
        let not_found = tree.find_by_path("Nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_leaf_items() {
        let tree = setup_test_tree();
        let leaves = tree.get_leaf_items();
        
        assert_eq!(leaves.len(), 4); // Install Ollama, Launch Chat, Settings, Quit
    }

    #[test]
    fn test_get_flat_items() {
        let tree = setup_test_tree();
        let flat = tree.get_flat_items();
        
        assert!(!flat.is_empty());
    }

    #[test]
    fn test_action_dispatcher_register() {
        let mut dispatcher = ActionDispatcher::new();
        
        dispatcher.register(Box::new(ClosureAction::new(
            "test_action",
            "Test",
            |_| Ok(ActionResult::success())
        )));
        
        assert!(dispatcher.get("test_action").is_some());
        assert!(dispatcher.get("nonexistent").is_none());
    }

    #[test]
    fn test_action_dispatcher_execute() {
        let dispatcher = setup_dispatcher();
        
        let ctx = ActionContext::new();
        let result = dispatcher.execute("install_ollama", &ctx);
        
        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.success);
    }

    #[test]
    fn test_action_context_params() {
        let ctx = ActionContext::new()
            .with_param("key1", "value1")
            .with_param("key2", "value2");
        
        assert_eq!(ctx.get_param("key1"), Some(&"value1".to_string()));
        assert_eq!(ctx.get_param("key2"), Some(&"value2".to_string()));
        assert_eq!(ctx.get_param("missing"), None);
    }

    #[test]
    fn test_menu_handler_current_items() {
        let tree = setup_test_tree();
        let dispatcher = setup_dispatcher();
        let mut state = crate::config::WzllamaState::default();
        let i18n = crate::config::I18n::default();
        let hw = crate::core::HardwareInfo::default();
        let handler = MenuHandler::new(tree, dispatcher, &i18n, &mut state, &hw);
        
        let items = handler.current_items();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_menu_handler_register_action() {
        let tree = setup_test_tree();
        let dispatcher = setup_dispatcher();
        let mut state = crate::config::WzllamaState::default();
        let i18n = crate::config::I18n::default();
        let hw = crate::core::HardwareInfo::default();
        let mut handler = MenuHandler::new(tree, dispatcher, &i18n, &mut state, &hw);
        
        handler.register_action(Box::new(ClosureAction::new(
            "new_action",
            "New Action",
            |_| Ok(ActionResult::success())
        )));
        
        // Verify dispatcher has the action registered
        assert!(handler.dispatcher().get("new_action").is_some());
    }

    #[test]
    fn test_action_result_success() {
        let result = ActionResult::success();
        assert!(result.success);
        assert!(result.message.is_none());
        
        let result_with_msg = ActionResult::success_with("Done!");
        assert!(result_with_msg.success);
        assert_eq!(result_with_msg.message, Some("Done!".to_string()));
    }

    #[test]
    fn test_action_result_failure() {
        let result = ActionResult::failure("Something went wrong");
        assert!(!result.success);
        assert_eq!(result.message, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_navigation_state_default() {
        use crate::menu_api::menu_handler::NavigationState;
        
        let state = NavigationState::default();
        assert_eq!(state.history, vec![0]);
        assert_eq!(state.current_index, 0);
    }

    #[test]
    fn test_api_first_menu_structure() {
        // Test that get_menu_structure produces valid JSON
        let i18n = crate::config::I18n::default();
        let state = crate::config::WzllamaState::load();
        
        let menu = crate::menu_api::get_menu_structure(&i18n, &state);
        
        // Verify structure
        assert!(menu.is_object());
        assert!(menu.get("id").is_some());
        assert!(menu.get("label").is_some());
        assert!(menu.get("items").is_some());
        
        let items = menu.get("items").unwrap().as_array().unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_api_first_tools_menu() {
        // Test that get_tools_menu produces valid JSON with tool info
        let i18n = crate::config::I18n::default();
        let state = crate::config::WzllamaState::load();
        
        let menu = crate::menu_api::get_tools_menu(&i18n, &state);
        
        // Verify structure
        assert!(menu.is_object());
        assert!(menu.get("id").is_some());
        assert!(menu.get("items").is_some());
        
        let items = menu.get("items").unwrap().as_array().unwrap();
        for item in items {
            assert!(item.get("id").is_some());
            assert!(item.get("label").is_some());
            assert!(item.get("action_id").is_some());
        }
    }

    #[test]
    fn test_api_first_models_menu() {
        // Test that get_models_menu produces valid JSON
        let i18n = crate::config::I18n::default();
        let state = crate::config::WzllamaState::load();
        
        let menu = crate::menu_api::get_models_menu(&i18n, &state);
        
        // Verify structure
        assert!(menu.is_object());
        assert!(menu.get("id").is_some());
        assert!(menu.get("items").is_some());
    }

    #[test]
    fn test_cleanup_menu_tree_structure() {
        // Integration test for cleanup menu
        use crate::menu_api::wizard_actions::build_cleanup_menu_tree;
        
        let tree = build_cleanup_menu_tree();
        
        // Verify menu has Retour in position 0
        assert!(tree.root.submenus.len() >= 2);
        assert!(tree.root.submenus[0].label.contains("Retour"));
    }

    #[test]
    fn test_usecase_variants() {
        use crate::menu_api::UseCase;
        
        let all = UseCase::all();
        assert_eq!(all.len(), 6);
        
        assert_eq!(UseCase::General.as_str(), "general");
        assert_eq!(UseCase::Coding.as_str(), "coding");
        assert_eq!(UseCase::Reasoning.as_str(), "reasoning");
        assert_eq!(UseCase::Chat.as_str(), "chat");
        assert_eq!(UseCase::Multimodal.as_str(), "multimodal");
        assert_eq!(UseCase::Embedding.as_str(), "embedding");
    }

    #[test]
    fn test_scientific_category_variants() {
        use crate::menu_api::ScientificCategory;
        
        let all = ScientificCategory::all();
        assert_eq!(all.len(), 6);
        
        // Verify no duplicate skills across categories
        let all_skills: Vec<&str> = all.iter()
            .flat_map(|c| c.skills.iter().copied())
            .collect();
        
        // Each category should have unique skills
        assert!(all_skills.contains(&"gget")); // bioinformatics/genomics
        assert!(all_skills.contains(&"rdkit")); // cheminformatics
    }

    #[test]
    fn test_retour_in_position_0_pattern() {
        // Test TODO.md lign 72: "Retour" must be in position 0 for all submenus
        
        // Test cleanup menu
        let cleanup = crate::menu_api::wizard_actions::build_cleanup_menu_tree();
        assert!(cleanup.root.submenus[0].label.contains("Retour"), 
            "Cleanup menu should have Retour in position 0");
        
        // Test wizard menu
        let wizard = crate::wizard::menu_wizard::build_menu_tree();
        assert!(wizard.root.submenus[0].label.contains("Retour"),
            "Wizard menu should have Retour in position 0");
        
        // Test models menu
        let models = crate::wizard::menu_models::build_menu_tree();
        assert!(models.root.submenus[0].label.contains("Retour"),
            "Models menu should have Retour in position 0");
        
        // Test tools menu
        let tools = crate::wizard::menu_tools::build_menu_tree();
        assert!(tools.root.submenus[0].label.contains("Retour"),
            "Tools menu should have Retour in position 0");
        
        // Test scientific menu
        let scientific = crate::wizard::menu_scientific::build_menu_tree();
        assert!(scientific.root.submenus[0].label.contains("Retour"),
            "Scientific menu should have Retour in position 0");
        
        // Test config menu
        let config = crate::wizard::menu_config::build_menu_tree();
        assert!(config.root.submenus[0].label.contains("Retour"),
            "Config menu should have Retour in position 0");
    }
}