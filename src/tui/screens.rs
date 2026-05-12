use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Screen {
    Main,
    Models,
    ModelSelect,
    Tools,
    ToolSelect,
    Fleets,
    FleetSelect,
    Cleanup,
    CleanupSelect,
    Config,
    ConfigEdit,
}

impl Screen {
    pub fn next(&self) -> Self {
        match self {
            Screen::Main => Screen::Models,
            Screen::Models => Screen::ModelSelect,
            Screen::ModelSelect => Screen::Tools,
            Screen::Tools => Screen::ToolSelect,
            Screen::ToolSelect => Screen::Fleets,
            Screen::Fleets => Screen::FleetSelect,
            Screen::FleetSelect => Screen::Cleanup,
            Screen::Cleanup => Screen::CleanupSelect,
            Screen::CleanupSelect => Screen::Config,
            Screen::Config => Screen::ConfigEdit,
            Screen::ConfigEdit => Screen::Main,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Screen::Main => Screen::ConfigEdit,
            Screen::ConfigEdit => Screen::Config,
            Screen::Config => Screen::CleanupSelect,
            Screen::CleanupSelect => Screen::Cleanup,
            Screen::Cleanup => Screen::FleetSelect,
            Screen::FleetSelect => Screen::Fleets,
            Screen::Fleets => Screen::ToolSelect,
            Screen::ToolSelect => Screen::Tools,
            Screen::Tools => Screen::ModelSelect,
            Screen::ModelSelect => Screen::Models,
            Screen::Models => Screen::Main,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Screen::Main => "🏠 Menu Principal",
            Screen::Models => "🤖 Modèles IA",
            Screen::ModelSelect => "🔍 Sélection Modèle",
            Screen::Tools => "🛠️  Outils",
            Screen::ToolSelect => "🔧 Sélection Outil",
            Screen::Fleets => "🚀 Flottes OpenClaw",
            Screen::FleetSelect => "⚓ Sélection Flotte",
            Screen::Cleanup => "🧹 Nettoyage",
            Screen::CleanupSelect => "🗑️  Sélection Suppression",
            Screen::Config => "⚙️  Configuration",
            Screen::ConfigEdit => "🔧 Édition Config",
        }
    }
}