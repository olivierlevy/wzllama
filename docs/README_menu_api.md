# Menu API - Dynamic Menu Tree Management System

## Overview

The `menu_api` module provides a flexible, data-driven menu system that completely separates menu structure from business logic in wzllama.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        menu_api module                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐   │
│  │   MenuTree   │─────▶│   MenuItem   │─────▶│  MenuHandler │   │
│  │ (structure)  │      │ (node)       │      │ (runtime)    │   │
│  └──────────────┘      └──────────────┘      └──────────────┘   │
│         │                       │                     │          │
│         ▼                       ▼                     ▼          │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐   │
│  │ MenuMetadata │      │ action_id    │      │ ToolAction   │   │
│  │ (title, etc) │      │ submenus     │      │ dispatcher   │   │
│  └──────────────┘      └──────────────┘      └──────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Option A: Integration with MenuHandler (Current Approach)

### Current State
The menu_api system provides both:
1. **MenuTree structure** - via `build_menu_tree()` in each wizard module
2. **Interactive menu_runner** - via `*MenuRunner` structs that delegate to existing wizard functions

This hybrid approach provides:
- **API compatibility**: `MenuTree` can be consumed by external systems (JSON API at port 1133)
- **Backward compatibility**: Existing `dialoguer::Select` calls continue to work
- **Gradual migration**: Easy to switch to full `MenuHandler` when ready

### Migration Path
```rust
// Phase 1: Structure exposed ✓
let tree = MainMenuRunner::new(i18n, state, hw).menu_tree();

// Phase 2: MenuHandler integration ✓
let mut runner = MainMenuRunner::new(i18n, state, hw);
runner.run_with_menu_handler()?;  // Uses ArcAction + MenuHandler

// Phase 3: Complete Migration ✓
runner.run()?;  // Now uses MenuHandler by default
```

### Dynamic Menu Structure
```rust
// MainMenuRunner.menu_tree() shows dynamic content:
// - Resume option appears only if last_tool && last_model exist
// - Uses I18n for labels
// - Reflects current state
```

## Core Components

### MenuTree
```rust
let tree = MenuTree::new("myapp")
    .with_root(MenuItem::branch("main")
        .add_submenu(MenuItem::leaf("↩️ Retour"))  // Position 0 = Back
        .add_submenu(MenuItem::leaf("Action").with_action("action_id")));
```

### MenuItem
- `leaf(label)` - Terminal menu item
- `branch(label)` - Container with submenus
- `with_action(id)` - Associate an action ID
- `add_submenu(item)` - Add child item

### MenuHandler
```rust
let handler = MenuHandler::new(tree, dispatcher);
handler.run()?;  // Interactive loop with dialoguer
```

### ToolAction & ActionDispatcher
```rust
let action = ClosureAction::new("id", "Name", |ctx| {
    Ok(ActionResult::success())
});
dispatcher.register(Box::new(action));
```

## Pattern: "Retour" in Position 0

Per TODO.md ligne 72, all submenus must have "↩️ Retour" in position 0:

```rust
MenuItem::branch("menu")
    .add_submenu(MenuItem::leaf("↩️ Retour"))  // Always first
    .add_submenu(MenuItem::leaf("Option 1"))
    .add_submenu(MenuItem::leaf("Option 2"));
```

The `MenuHandler` detects this pattern and handles back navigation automatically.

## External Configuration

Menus can be defined in TOML or JSON:

**menus/main.toml**
```toml
version = "1.0"
title = "Main Menu"

[[items]]
label = "Install"
children = [
    { label = "Ollama", action_id = "install_ollama" },
    { label = "Open WebUI", action_id = "install_open_webui" },
]

[[items]]
label = "Launch"
children = [
    { label = "Chat", action_id = "launch_chat" },
]
```

```rust
let tree = load_from_toml("menus/main.toml")?;
```

## Migration Status

### Completed ✓
- [x] MenuTree/MenuItem data structures
- [x] MenuHandler with dialoguer Select
- [x] ToolAction trait and implementations
- [x] ActionDispatcher
- [x] "Retour" position 0 pattern
- [x] build_menu_tree() in all wizard files
- [x] Configuration loading (TOML/JSON)
- [x] API endpoints for menu structure
- [x] ArcAction for cloneable actions
- [x] **Phase 3: run() uses MenuHandler by default**

### Files Structure
```
src/menu_api/
├── mod.rs              # Public exports
├── menu_tree.rs        # MenuTree, MenuMetadata, MenuConfig
├── menu_item.rs        # MenuItem structure
├── menu_handler.rs     # Interactive handler
├── tool_action.rs      # ToolAction trait
├── arc_action.rs       # Cloneable actions
├── config_loader.rs    # TOML/JSON loaders
├── wizard_adapter.rs   # Migration utilities
├── wizard_actions.rs   # Wizard context
├── wizard_helpers.rs   # Shared helpers
├── dynamic_builder.rs  # Menu builders
├── api_first.rs        # API endpoints
└── *_adapter.rs        # Menu runners
```

## Usage Example

```rust
use wzllama::menu_api::{MenuTree, MenuItem, MenuHandler, ActionDispatcher, ClosureAction, MenuItem};

// 1. Define menu structure
fn build_menu() -> MenuTree {
    MenuItem::branch("main")
        .add_submenu(MenuItem::leaf("↩️ Retour"))
        .add_submenu(MenuItem::leaf("Wizard").with_action("wizard"))
        .add_submenu(MenuItem::leaf("Tools").with_action("tools"))
}

// 2. Create dispatcher with actions
let mut dispatcher = ActionDispatcher::new();
dispatcher.register(Box::new(ClosureAction::new("wizard", "Wizard", |_ctx| {
    Ok(ActionResult::success_with("Wizard menu"))
})));

// 3. Run menu
let tree = build_menu();
let mut handler = MenuHandler::new(tree, dispatcher);
handler.run()?;
```

## Test Coverage

```
cargo test --lib
# 92+ tests passing
# - 13 tool_trait_tests
# - 26 menu_api tests (including test_retour_in_position_0_pattern)
# - 10 i18n tests
# - 8 state tests
# - 9 templates tests
# - 3 wizard tests
```

### Key Test: Retour in Position 0

```rust
#[test]
fn test_retour_in_position_0_pattern() {
    // Verifies TODO.md ligne 72: "Retour" must be in position 0 for all submenus
    let wizard = crate::wizard::menu_wizard::build_menu_tree();
    assert!(wizard.root.submenus[0].label.contains("Retour"));
    
    let models = crate::wizard::menu_models::build_menu_tree();
    assert!(models.root.submenus[0].label.contains("Retour"));
    
    // ... similar checks for tools, scientific, config menus
}
```