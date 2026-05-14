# Mode CLI (Wizard)

## Vue d'ensemble

Le mode Wizard est l'interface par défaut de wzllama. Il utilise `dialoguer` pour créer des menus en ligne de commande interactifs.

## Architecture du wizard

### Point d'entrée

```
src/main.rs
└── Cli::execute() -> wizard::run()

src/wizard/menu_main.rs
└── run(i18n, state, hw) -> boucle principale
```

### Menus du wizard

1. **Menu principal** (`menu_main.rs`)
   - Affichage ressources système (RAM/VRAM)
   - Navigation entre les sous-menus
   
2. **Modèles** (`menu_models.rs`)
   - Lister modèles locaux
   - Télécharger de nouveaux modèles
   - Supprimer des modèles
   
3. **Outils** (`menu_tools.rs`)
   - Lancer un outil installé
   - Installer un nouvel outil
   - Voir l'état d'installation
   
4. **Flottes** (`menu_fleets.rs`)
   - Lister les flottes OpenClaw
   - Créer une nouvelle flotte
   - Lancer une flotte existante
   
5. **Nettoyage** (`menu_cleanup.rs`)
   - Supprimer des outils
   - Supprimer des modèles
   - Supprimer des flottes
   
6. **Configuration** (`menu_config.rs`)
   - Modifier les modèles par usage
   - Ajuster les paramètres Ollama
   - Générer le fichier env

## Alternate Screen Buffer

Le menu principal utilise le buffer d'écran alternatif pour un effet "interface fixe":

```rust
// src/wizard/menu_main.rs

/// Enter alternate screen buffer (keeps content fixed)
fn enter_alternate_screen() {
    print!("\x1b[?1049h");
    use std::io::Write;
    std::io::stdout().flush().ok();
}

/// Exit alternate screen buffer
fn exit_alternate_screen() {
    print!("\x1b[?1049l");
    use std::io::Write;
    std::io::stdout().flush().ok();
}
```

Cela permet d'afficher un header fixe qui ne défile pas avec le contenu.

## Navigation et contrôles

### Contrôles clavier

| Touche | Action |
|--------|--------|
| ↑/↓ | Naviguer dans le menu |
| Enter | Valider la sélection |
| Escape | Retour au menu précédent |
| Ctrl-C | Quitter immédiatement |

### Gestion de l'Escape

Tous les menus utilisent `interact_opt()` au lieu de `interact()`:

```rust
// Pattern standard utilisé dans tout le wizard
let choice = match Select::new()
    .with_prompt(i18n.t("menu.main.choose"))
    .items(&items)
    .interact_opt()? {
    Some(c) => c,
    None => return Ok(()), // Escape/Ctrl-C pressed - retour au menu parent
};
```

### Calcul dynamique de la hauteur de menu

```rust
// src/display.rs
pub fn menu_max_items(items_count: usize, reserved_lines: usize) -> usize {
    let (_, term_height) = get_terminal_size();
    let max = (term_height as usize).saturating_sub(reserved_lines);
    std::cmp::min(items_count, std::cmp::max(3, max))
}
```

Adapte automatiquement la hauteur du menu au terminal.

## Exemples de code wizard

### Structure d'un menu wizard

```rust
pub fn run(i18n: &I18n, state: &mut WzllamaState, hw: &HardwareInfo) -> Result<()> {
    loop {
        // Affichage du header
        display::header(&i18n.t("menu.title"));
        
        // Construction des items
        let items = vec![item1, item2, back_item];
        
        // Sélection avec Escape handling
        let sel = match Select::new()
            .with_prompt(i18n.t("menu.choose"))
            .items(&items)
            .interact_opt()? {
            Some(s) => s,
            None => return Ok(()),
        };
        
        // Gestion du choix
        match sel {
            n if n == items.len() - 1 => return Ok(()), // Retour
            0 => action1(i18n, state)?,
            1 => action2(i18n, state)?,
            _ => {}
        }
    }
}
```

### Menu avec Input

```rust
let new_value: String = Input::new()
    .with_prompt(i18n.t_with_vars("config.edit", &[("field", label)]))
    .default(old_value.clone())
    .interact()?;
```

### Menu avec Confirm

```rust
if Confirm::new()
    .with_prompt(i18n.t("confirm"))
    .default(false)
    .interact()? 
{
    // Action confirmée
}
```

## État persistant

### Structure WzllamaState

```rust
pub struct WzllamaState {
    pub language: Option<String>,      // Langue choisie
    pub last_model: Option<String>,    // Dernier modèle utilisé
    pub installed: InstalledTools,     // Outils installés
    pub fleets: Vec<String>,           // Noms des flottes
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstalledTools {
    pub ollama: bool,
    pub open_webui: bool,
    pub openclaw: bool,
    pub claude_code: bool,
    pub opencode: bool,
    pub codex: bool,
    pub copilot_cli: bool,
    pub droid: bool,
    pub hermes_agent: bool,
    pub pi: bool,
    pub pool: bool,
}
```

### Sauvegarde/Chargement

```rust
// Sauvegarde dans ~/.wzllama/state.json
impl WzllamaState {
    pub fn load() -> Self { /* ... */ }
    pub fn save(&self) -> Result<()> { /* ... */ }
}
```

## Internationalisation dans le wizard

### Utilisation des clés de traduction

```rust
// Dans le code wizard
display::header(&i18n.t("menu.main.title"));

let sel = Select::new()
    .with_prompt(i18n.t("menu.main.choose"))
    .items(&items)
    .interact_opt()?;
```

### Fichier de traduction (excerpt)

```json
{
  "menu.main.title": "Menu Principal",
  "menu.main.choose": "Que voulez-vous faire ?",
  "menu.main.models": "🤖 Choisir un modèle IA",
  "menu.main.tools": "🛠 Lancer un outil",
  "menu.main.quit": "❌ Quitter",
  "menu.back": "↩ Retour"
}
```

## Compatibilité terminal

Le wizard fonctionne avec:
- Terminal minimum: 40x10 caractères (mode compact)
- Terminal recommandé: 70x25 caractères (mode complet)
- Toute taille: adaptation automatique de la hauteur de menu