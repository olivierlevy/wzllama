# Mode TUI (Terminal User Interface)

## Vue d'ensemble

Le mode TUI (Terminal User Interface) utilise `ratatui` et `crossterm` pour une interface plus riche avec widgets temps réel.

## Activation

```bash
# Activer le mode TUI
wzllama --tui

# Le mode TUI nécessite un terminal suffisamment grand
# (recommendé: 80x25 minimum)
```

## Architecture TUI

### Point d'entrée

```
src/tui/mod.rs
└── pub fn run_tui(state, hardware, i18n)
```

### Structure du code

```
src/tui/
├── mod.rs          # run_tui() entry point
├── app.rs          # Application state machine
├── ui.rs           # Rendering functions
├── event.rs        # Event handling (keyboard, mouse)
├── screens.rs      # Screen enum and navigation
├── widgets.rs      # Custom widgets (ResourceBar, etc.)
└── terminal.rs     # Terminal setup/cleanup
```

## Application State (src/tui/app.rs)

```rust
pub struct App {
    pub state: WzllamaState,
    pub hardware: HardwareInfo,
    pub i18n: I18n,
    pub screen: Screen,
    pub should_exit: bool,
    // ... autres champs
}

pub enum Screen {
    Main,
    Models,
    Tools,
    Fleets,
    Cleanup,
    Config,
    Language,
    Quit,
}
```

## Événements et navigation

### Gestion des événements

```rust
// src/tui/event.rs
pub fn handle_event(app: &mut App, event: Event) -> Result<()> {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Up => app.previous(),
            KeyCode::Down => app.next(),
            KeyCode::Enter => app.select(),
            KeyCode::Esc => app.go_back(),
            _ => {}
        },
        // ... mouse events
    }
}
```

### Navigation entre écrans

```rust
impl App {
    pub fn go_to(&mut self, screen: Screen) {
        self.screen = screen;
    }
    
    pub fn go_back(&mut self) {
        self.screen = match self.screen {
            Screen::Main => Screen::Quit,
            _ => Screen::Main,
        };
    }
}
```

## Rendu TUI (src/tui/ui.rs)

```rust
pub fn render(app: &mut App, frame: &mut Frame) {
    match app.screen {
        Screen::Main => render_main(app, frame),
        Screen::Models => render_models(app, frame),
        Screen::Tools => render_tools(app, frame),
        // ...
    }
}
```

### Layout principal

```
┌─────────────────────────────────────────────────────┐
│ Header: Titre + Status bar                           │
├─────────────────────────────────────────────────────┤
│                                                       │
│  [   Menu Principal / Sous-menu   ]                  │
│                                                       │
│  -> Choix 1                                      │
│  -> Choix 2                                      │
│  -> Quitter                                       │
│                                                       │
├─────────────────────────────────────────────────────┤
│ Footer: Infos système                                │
└─────────────────────────────────────────────────────┘
```

## Différences TUI vs Wizard

| Aspect | Wizard (CLI) | TUI |
|--------|-------------|-----|
| Bibliothèque | dialoguer | ratatui/crossterm |
| Affichage | Ligne par ligne | Widgets temps réel |
| Navigation | ←→ Enter | ↑↓ Enter |
| Escape | Retour menu | Retour/Quitter |
| Terminal min | 40x10 | 60x20 recommandé |
| State | Menue par menu | State machine unifiée |

## Widgets personnalisés (src/tui/widgets.rs)

### ResourceBar

Affiche les ressources système avec barres de progression:

```rust
pub struct ResourceBar {
    pub label: String,
    pub current: f64,
    pub total: f64,
    pub color: Color,
}
```

## Performance et optimisation

Le TUI est plus gourmand en ressources mais offre:
- Mise à jour temps réel des ressources
- Affichage fixe sans clignotement
- Navigation fluide entre écrans

## Limitations actuelles

- Pas de mode compact si terminal trop petit
- Certaines fonctions du wizard pas encore dans TUI
- Le TUI est en développement actif