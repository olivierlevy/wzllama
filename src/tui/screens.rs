use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Screen {
    // Main navigation
    Main,
    Models,     // Choisir un modèle IA -> sous-écran Usage puis modèles
    Tools,      // Lancer un outil
    Cleanup,    // Nettoyage -> sous-menu
    Config,     // Configuration -> sous-menu
    Language,   // Changer de langue
    Quit,
    // Sub-screens
    ModelUsage, // Sélection de l'usage (Agents rapides, Gros livre, etc.)
    CleanupTools,
    CleanupFleets,
    CleanupModels,
    ConfigModels,
    ConfigPerformance,
    ConfigShells,
    Exec,
}

impl Screen {
    /// Get the main menu screens (for sidebar navigation)
    fn menu_screens() -> &'static [Screen] {
        static MENU_SCREENS: &[Screen] = &[
            Screen::Main,
            Screen::Models,
            Screen::Tools,
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
            Screen::Main => Screen::Models,
            Screen::Models => Screen::Tools,
            Screen::Tools => Screen::Cleanup,
            Screen::Cleanup => Screen::Config,
            Screen::Config => Screen::Language,
            Screen::Language => Screen::Quit,
            Screen::Quit => Screen::Main,
            Screen::ModelUsage => Screen::Cleanup,
            Screen::CleanupTools => Screen::CleanupFleets,
            Screen::CleanupFleets => Screen::CleanupModels,
            Screen::CleanupModels => Screen::Main,
            Screen::ConfigModels => Screen::ConfigPerformance,
            Screen::ConfigPerformance => Screen::ConfigShells,
            Screen::ConfigShells => Screen::Main,
            Screen::Exec => Screen::Exec,
        }
    }

    #[allow(dead_code)]
    pub fn previous(&self) -> Self {
        match self {
            Screen::Main => Screen::Quit,
            Screen::Quit => Screen::Language,
            Screen::Language => Screen::Config,
            Screen::Config => Screen::Cleanup,
            Screen::Cleanup => Screen::Tools,
            Screen::Tools => Screen::Models,
            Screen::Models => Screen::Main,
            Screen::ModelUsage => Screen::Main,
            Screen::CleanupTools => Screen::Main,
            Screen::CleanupFleets => Screen::CleanupTools,
            Screen::CleanupModels => Screen::CleanupFleets,
            Screen::ConfigModels => Screen::Main,
            Screen::ConfigPerformance => Screen::ConfigModels,
            Screen::ConfigShells => Screen::ConfigPerformance,
            Screen::Exec => Screen::Exec,
        }
    }

    #[allow(dead_code)]
    pub fn title(&self) -> &'static str {
        match self {
            Screen::Main => "🏠 Que voulez-vous faire ?",
            Screen::Models => "🤖 Choisir un modèle IA",
            Screen::ModelUsage => "🤖 Choisir votre usage",
            Screen::Tools => "🛠️ Lancer un outil",
            Screen::Cleanup => "🧹 Nettoyage",
            Screen::CleanupTools => "🧹 Désinstaller des outils",
            Screen::CleanupFleets => "🧹 Supprimer des flottes",
            Screen::CleanupModels => "🧹 Supprimer des modèles",
            Screen::Config => "⚙️ Configuration",
            Screen::ConfigModels => "⚙️ Modèles par usage",
            Screen::ConfigPerformance => "⚙️ Performance",
            Screen::ConfigShells => "⚙️ Shells",
            Screen::Language => "🌍 Changer de langue",
            Screen::Quit => "❌ Quitter",
            Screen::Exec => "💻 Shell",
        }
    }
}