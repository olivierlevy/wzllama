# Mode TUI (Terminal User Interface)

> **Note**: Le mode TUI est actuellement désactivé. Le code est conservé pour référence future, mais l'interface CLI wizard reste l'interface principale.

## État actuel

Le mode TUI (`--tui`) a été désactivé au profit de l'interface CLI wizard interactif. 

### Pourquoi le désactiver ?

- L'interface CLI utilise `dialoguer` et est plus légère
- Moins de dépendances (pas de `ratatui`/`crossterm` requis)
- Compatible avec plus de terminaux
- Plus simple à maintenir

## Code conservé

Le code source TUI est conservé dans `src/tui/` pour référence future:

```
src/tui/
├── mod.rs          # run_tui() entry point
├── app.rs          # Application state machine  
├── ui.rs           # Rendering functions
├── event.rs        # Event handling
├── screens.rs      # Screen enum and navigation
├── widgets.rs      # Custom widgets
└── terminal.rs     # Terminal setup/cleanup
```

## Réactivation future

Pour réactiver le TUI, il suffit de:

1. Décommenter `mod tui;` dans `src/main.rs` et `src/lib.rs`
2. Décommenter le flag `--tui` dans `src/cli.rs`
3. Rebrancher sur la logique TUI dans `cli.rs`

```rust
// src/main.rs
mod tui; // Décommenter

// src/cli.rs  
#[arg(long, global = true)]
pub tui: bool, // Décommenter

// Et décommenter la logique dans execute()
```