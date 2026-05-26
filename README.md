# wzllama - Menu API Documentation

> **Documentation de migration disponible**: Voir [`docs/migration/`](docs/migration/README.md) pour une documentation complète permettant de reconstruire ce projet dans un autre langage.

## Overview

The `menu_api` module provides a hierarchical, configurable menu system for the wzllama CLI application. It separates menu structure from business logic, enabling dynamic menu generation and flexible action dispatch.

## Architecture

```
menu_api/
├── menu_tree.rs        # MenuTree - root container for hierarchical menus
├── menu_item.rs        # MenuItem - leaf or branch with optional action
├── menu_handler.rs     # MenuHandler - interactive navigation & execution
├── tool_action.rs      # ToolAction trait & ActionDispatcher for command execution
├── wizard_adapter.rs   # WizardAdapter - bridges wizard:: functions to menu_api
├── wizard_menu_handler.rs # WizardMenuRunner - migrated wizard logic
├── dynamic_generators.rs  # Dynamic submenu generators
├── models_engine.rs    # ModelsEngineRunner for model workflows
├── tools_engine.rs     # ToolsEngineRunner for tool workflows
├── scientific_menu_adapter.rs # ScientificMenuRunner
├── config_menu_adapter.rs     # ConfigMenuRunner
├── cleanup_menu_adapter.rs    # CleanupMenuRunner
├── main_menu_adapter.rs       # MainMenuRunner
└── api_service.rs      # HTTP API service layer
```

## Core Components

### MenuTree

```rust
pub struct MenuTree {
    pub root: MenuItem,
    pub metadata: MenuMetadata,
}

impl MenuTree {
    pub fn new(root_label: &str) -> Self;
    pub fn with_title(root_label: &str, title: &str) -> Self;
    pub fn with_root(mut self, root: MenuItem) -> Self;
    pub fn find_by_path(&self, path: &str) -> Option<&MenuItem>;
    pub fn get_leaf_items(&self) -> Vec<&MenuItem>;
}
```

### MenuItem

```rust
pub struct MenuItem {
    pub label: String,
    pub action_id: Option<String>,
    pub submenus: Vec<MenuItem>,
    pub label_vars: HashMap<String, String>,
}

impl MenuItem {
    pub fn leaf(label: &str) -> Self;
    pub fn branch(label: &str) -> Self;
    pub fn with_action(self, action_id: &str) -> Self;
    pub fn add_submenu(self, item: MenuItem) -> Self;
    pub fn is_leaf(&self) -> bool;
    pub fn has_action(&self) -> bool;
}
```

### MenuHandler

```rust
pub struct MenuHandler<'a> {
    // Navigation state, references to i18n, state, hardware
}

impl<'a> MenuHandler<'a> {
    pub fn new(
        tree: MenuTree,
        dispatcher: ActionDispatcher,
        i18n: &'a I18n,
        state: &'a mut WzllamaState,
        hw: &'a HardwareInfo,
    ) -> Self;
    
    pub fn run(&mut self) -> Result<()>;  // Interactive loop
    pub fn register_action(&mut self, action: Box<dyn ToolAction>);
}
```

### ActionDispatcher & ToolAction

```rust
pub trait ToolAction: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, ctx: &ActionContext) -> Result<ActionResult>;
}

pub struct ActionDispatcher {
    // Registered actions map
}

impl ActionDispatcher {
    pub fn register(&mut self, action: Box<dyn ToolAction>);
    pub fn execute(&self, action_id: &str, ctx: &ActionContext) -> Result<ActionResult>;
}
```

## Navigation Pattern

The MenuHandler implements the "Retour en position 0" pattern as specified in TODO.md line 72:

- "Retour" is always displayed in position 0 for sub-menus
- "Quitter" is always displayed in the last position
- The handler automatically detects and handles "Retour" navigation

```
Main Menu
├── ↩️ Retour (only shown in sub-menus)
├── Menu Item 1
├── Menu Item 2
├── Menu Item 3
└── ✖ Quitter
```

## Usage Example

```rust
use menu_api::{MenuTree, MenuItem, MenuHandler, ActionDispatcher, ToolAction, ActionResult};

// Build menu tree
let tree = MenuTree::new("main")
    .with_root(
        MenuItem::branch("main")
            .add_submenu(MenuItem::leaf("Option 1").with_action("action1"))
            .add_submenu(MenuItem::leaf("Option 2").with_action("action2"))
    );

// Create dispatcher with actions
let mut dispatcher = ActionDispatcher::new();
dispatcher.register(Box::new(ClosureAction::new(
    "action1",
    "Action 1",
    |_| Ok(ActionResult::success())
)));

// Run interactive handler
let mut handler = MenuHandler::new(tree, dispatcher, i18n, state, hw);
handler.run()?;
```

## Dynamic Menu Generation

Menus can be generated dynamically from:

1. **TOML/JSON configuration files** - see `config_loader.rs`
2. **Runtime functions** - see `dynamic_generators.rs`
3. **API data** - models from llmfit/localmaxxing APIs

## Integration Points

| Wizard File | Menu API Equivalent | Status |
|-------------|-------------------|--------|
| `menu_wizard.rs` | `WizardMenuRunner` | ✅ Migrated |
| `menu_models.rs` | `ModelsEngineRunner` | ✅ Wrapper |
| `menu_tools.rs` | `ToolsEngineRunner` | ✅ Wrapper |
| `menu_scientific.rs` | `ScientificMenuRunner` | ✅ Wrapper |
| `menu_cleanup.rs` | `CleanupMenuRunner` | ✅ Wrapper |
| `menu_config.rs` | `ConfigMenuRunner` | ✅ Wrapper |

## API Service Layer

The `api_service.rs` provides HTTP endpoints for:
- `GET /api/menu/state` - Current state
- `GET /api/menu/structure` - Menu tree structure
- `POST /api/menu/action` - Execute action
- `GET /api/menu/i18n` - Internationalization strings

## Testing

Run tests with:
```bash
cargo test --lib
```

All 27 tests pass covering:
- MenuTree creation and navigation
- MenuItem structure and actions
- ActionDispatcher registration and execution
- MenuHandler navigation patterns
- UseCase and ScientificCategory enums
- Dynamic menu generators