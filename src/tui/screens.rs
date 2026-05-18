#![allow(dead_code)]

use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Screen {
    // Main navigation screens (sidebar)
    Information,  // Écran d'accueil avec description
    Models,       // Choisir un modèle IA (usage selection)
    ModelSelect,  // Sélection du modèle installé
    Tools,        // Lancer un outil
    Terminal,     // Terminal intégré PTY
    Cleanup,      // Nettoyage
    Config,       // Configuration
    Language,     // Changer de langue
    Quit,
    Exec,
}

impl Screen {
    /// Get the main menu screens (for sidebar navigation)
    pub fn menu_screens() -> &'static [Screen] {
        static MENU_SCREENS: &[Screen] = &[
            Screen::Information,
            Screen::Models,
            Screen::Tools,
            Screen::Terminal,
            Screen::Cleanup,
            Screen::Config,
            Screen::Language,
            Screen::Quit,
        ];
        MENU_SCREENS
    }

    /// Navigate to next main menu screen (for sidebar navigation with Up/Down)
    pub fn next_menu(&self) -> Self {
        let screens = Self::menu_screens();
        let idx = screens.iter().position(|s| *s == *self).unwrap_or(0);
        screens[(idx + 1) % screens.len()]
    }

    /// Navigate to previous main menu screen (for sidebar navigation with Up/Down)
    pub fn prev_menu(&self) -> Self {
        let screens = Self::menu_screens();
        let idx = screens.iter().position(|s| *s == *self).unwrap_or(0);
        screens[(idx + screens.len() - 1) % screens.len()]
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        match self {
            Screen::Information => Screen::Models,
            Screen::Models => Screen::Tools,
            Screen::ModelSelect => Screen::Tools,
            Screen::Tools => Screen::Terminal,
            Screen::Terminal => Screen::Cleanup,
            Screen::Cleanup => Screen::Config,
            Screen::Config => Screen::Language,
            Screen::Language => Screen::Quit,
            Screen::Quit => Screen::Information,
            Screen::Exec => Screen::Exec,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Screen::Information => Screen::Quit,
            Screen::Quit => Screen::Language,
            Screen::Language => Screen::Config,
            Screen::Config => Screen::Cleanup,
            Screen::Cleanup => Screen::Terminal,
            Screen::Terminal => Screen::Tools,
            Screen::Tools => Screen::ModelSelect,
            Screen::ModelSelect => Screen::Models,
            Screen::Models => Screen::Information,
            Screen::Exec => Screen::Exec,
        }
    }

    #[allow(dead_code)]
    pub fn title(&self) -> &'static str {
        match self {
            Screen::Information => "📖 À propos de wzllama",
            Screen::Models => "🤖 Choisir un modèle IA",
            Screen::ModelSelect => "📋 Sélectionner un modèle",
            Screen::Tools => "🛠️ Lancer un outil",
            Screen::Terminal => "💻 Terminal",
            Screen::Cleanup => "🧹 Nettoyage",
            Screen::Config => "⚙️ Configuration",
            Screen::Language => "🌍 Changer de langue",
            Screen::Quit => "❌ Quitter",
            Screen::Exec => "💻 Shell",
        }
    }
}