# Menu API Architecture

## Overview

The `menu_api` module provides a flexible, data-driven menu system that separates menu structure from business logic.

## Core Components

### MenuTree
Hierarchical menu structure with metadata.

```rust
let tree = MenuTree::new("myapp")
    .with_root(MenuItem::branch("main")
        .add_submenu(MenuItem::leaf("↩️ Retour"))
        .add_submenu(MenuItem::leaf("Action 1").with_action("action_1")));
```

### MenuItem
Represents a single menu item - either a leaf (with action) or branch (has submenus).

- `leaf(label)` - Create a leaf node
- `with_action(id)` - Associate an action ID
- `add_submenu(item)` - Add child menu items

### MenuHandler
Interactive menu navigation and execution.

```rust
let mut handler = MenuHandler::new(tree, dispatcher);
handler.run()?;
```

### ToolAction & ActionDispatcher
Action dispatch system. Actions are registered with the dispatcher and executed by ID.

## Migration Status

### Completed ✓
- [x] MenuTree/MenuItem data structures
- [x] MenuHandler with dialoguer Select
- [x] ToolAction trait and ClosureAction implementation
- [x] ActionDispatcher for action registration
- [x] "↩️ Retour" in position 0 pattern
- [x] build_menu_tree() in all wizard files

### In Progress
- [ ] Full MenuHandler integration in MainMenuRunner
- [ ] External configuration (TOML/JSON)

## Usage Example

```rust
// 1. Define menu structure (in wizard files)
pub fn build_menu_tree() -> MenuTree {
    MenuItem::branch("tools")
        .add_submenu(MenuItem::leaf("↩️ Retour"))
        .add_submenu(MenuItem::leaf("Tool A").with_action("tool_a"))
        .add_submenu(MenuItem::leaf("Tool B").with_action("tool_b"))
}

// 2. Run menu (uses dialoguer under the hood)
let mut runner = ToolsMenuRunner::new(i18n, state, hw);
runner.run()?;
```

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    menu_api module                       │
├─────────────────────────────────────────────────────────┤
│  MenuTree ────> MenuItem ────> submenus (Vec<MenuItem>)  │
│       │            │                                        │
│       │            └──> action_id: Option<String>          │
│       │                                                        │
│       └──> MenuMetadata (title, description, icon)          │
├─────────────────────────────────────────────────────────┤
│  MenuHandler ────> dialoguer::Select                      │
│        │                                                    │
│        ├──> navigate_to() ────> execute action            │
│        └──> navigate_back() ────> parent menu               │
├─────────────────────────────────────────────────────────┤
│  ToolAction trait                                          │
│  ├──> ClosureAction (for simple actions)                   │
│  └──> wizard functions (via adapters)                      │
└─────────────────────────────────────────────────────────┘
```