use crate::config::{WzllamaState, I18n};
use crate::core::hardware::HardwareInfo;
use crate::core::ollama_api;
use crate::tui::screens::Screen;
use std::sync::{Arc, Mutex};

/// Workflow step for Models screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelWorkflowStep {
    #[default]
    UsageSelection,   // Selecting usage type
    ModelSelection,   // Selecting from installed models
}

pub struct App {
    #[allow(dead_code)]
    pub state: Arc<Mutex<WzllamaState>>,
    pub hw: HardwareInfo,
    #[allow(dead_code)]
    pub i18n: I18n,
    pub current_screen: Screen,
    pub should_quit: bool,
    pub models: Vec<ollama_api::OllamaModel>,
    #[allow(dead_code)]
    pub selected_model: Option<usize>,
    pub selected_tool: Option<usize>,
    #[allow(dead_code)]
    pub selected_cleanup: Option<usize>,
    #[allow(dead_code)]
    pub search_query: String,
    /// Command to execute when entering Exec screen
    pub exec_command: Option<String>,
    /// Focus mode: true = sidebar navigation, false = content navigation
    pub sidebar_focus: bool,
    /// Current workflow step for Models screen
    pub model_workflow_step: ModelWorkflowStep,
    /// Tick counter for periodic refresh (refresh every ~5 seconds at 10Hz)
    tick_count: u32,
    /// Terminal state for integrated PTY terminal
    pub terminal: crate::tui::TerminalState,
    /// Terminal input buffer for keyboard echo
    pub terminal_input: String,
}

impl App {
    /// Create a new App instance (used by the main TUI)
    pub fn new(state: WzllamaState, hw: HardwareInfo, i18n: I18n) -> Self {
        let models = ollama_api::get_models();
        Self {
            state: Arc::new(Mutex::new(state)),
            hw,
            i18n,
            current_screen: Screen::Information,
            should_quit: false,
            models,
            selected_model: None,
            selected_tool: None,
            selected_cleanup: None,
            search_query: String::new(),
            exec_command: None,
            sidebar_focus: true,
            model_workflow_step: ModelWorkflowStep::default(),
            tick_count: 0,
            terminal: crate::tui::TerminalState::new(),
            terminal_input: String::new(),
        }
    }

    /// Create a test App instance with defaults
    pub fn new_test() -> Self {
        use crate::config::WzllamaState;
        use crate::core::hardware::HardwareInfo;
        use crate::config::i18n::I18n;
        
        let state = WzllamaState::default();
        let hw = HardwareInfo::default_for_test();
        let i18n = I18n::default();
        let models = vec![];
        
        Self {
            state: Arc::new(Mutex::new(state)),
            hw,
            i18n,
            current_screen: Screen::Information,
            should_quit: false,
            models,
            selected_model: None,
            selected_tool: None,
            selected_cleanup: None,
            search_query: String::new(),
            exec_command: None,
            sidebar_focus: true,
            model_workflow_step: ModelWorkflowStep::default(),
            tick_count: 0,
            terminal: crate::tui::TerminalState::new(),
            terminal_input: String::new(),
        }
    }

    pub fn tick(&mut self) {
        // Refresh models every 50 ticks (~5 seconds at 10Hz)
        self.tick_count += 1;
        if self.tick_count >= 50 {
            self.tick_count = 0;
            self.models = ollama_api::get_models();
        }
    }

    pub fn navigate(&mut self, direction: Navigation) {
        match direction {
            Navigation::Up => self.move_up(),
            Navigation::Down => self.move_down(),
            Navigation::Left => self.go_back(),
            Navigation::Right => self.select(),
            Navigation::Quit => self.should_quit = true,
            Navigation::Search => {
                // Tab: toggle focus between sidebar and content (disabled on Information screen)
                if self.current_screen != Screen::Information {
                    self.sidebar_focus = !self.sidebar_focus;
                    if self.sidebar_focus {
                        self.selected_tool = None;
                    } else {
                        self.selected_tool = Some(0);
                    }
                }
            }
        }
    }

    /// Navigate up - depends on focus mode
    fn move_up(&mut self) {
        if self.sidebar_focus {
            // Navigate in the sidebar menu (left panel)
            self.current_screen = self.current_screen.prev_menu();
        } else {
            // Navigate in the content list
            if self.current_screen == Screen::Models && self.model_workflow_step == ModelWorkflowStep::ModelSelection {
                // In ModelSelection, use selected_model
                let count = self.models.len();
                if count > 0 {
                    self.selected_model = Some(match self.selected_model {
                        None => 0,
                        Some(i) if i == 0 => count - 1,
                        Some(i) => i.saturating_sub(1),
                    });
                }
            } else {
                let count = self.get_current_screen_item_count();
                self.selected_tool = Some(match self.selected_tool {
                    None => 0,
                    Some(i) if i == 0 => count - 1,
                    Some(i) => i - 1,
                });
            }
        }
    }

    /// Navigate down - depends on focus mode
    fn move_down(&mut self) {
        if self.sidebar_focus {
            // Navigate in the sidebar menu (left panel)
            self.current_screen = self.current_screen.next_menu();
        } else {
            // Navigate in the content list
            if self.current_screen == Screen::Models && self.model_workflow_step == ModelWorkflowStep::ModelSelection {
                // In ModelSelection, use selected_model
                let count = self.models.len();
                if count > 0 {
                    self.selected_model = Some(match self.selected_model {
                        None => 0,
                        Some(i) if i >= count - 1 => 0,
                        Some(i) => i + 1,
                    });
                }
            } else {
                let count = self.get_current_screen_item_count();
                self.selected_tool = Some(match self.selected_tool {
                    None => 0,
                    Some(i) if i >= count - 1 => 0,
                    Some(i) => i + 1,
                });
            }
        }
    }

    fn go_back(&mut self) {
        // Left/Esc: navigate up in hierarchy
        if self.sidebar_focus {
            // On sidebar - go to parent (Information) 
            // Left on Information stays on Information (no parent)
            if self.current_screen != Screen::Information {
                // Reset Models workflow if we're leaving it
                if self.current_screen == Screen::Models {
                    self.model_workflow_step = ModelWorkflowStep::UsageSelection;
                    self.selected_model = None;
                }
                self.current_screen = Screen::Information;
            }
        } else {
            // In content - handle based on current screen and workflow
            if self.current_screen == Screen::Models && self.model_workflow_step == ModelWorkflowStep::ModelSelection {
                // From ModelSelection, go back to UsageSelection
                self.model_workflow_step = ModelWorkflowStep::UsageSelection;
                self.selected_model = None;
                self.selected_tool = Some(0);
            } else {
                // In content - return to sidebar mode
                self.sidebar_focus = true;
                self.selected_tool = None;
            }
        }
    }

    fn select(&mut self) {
        // Right/Enter: navigate based on current context
        if self.sidebar_focus {
            // In sidebar navigation - select the screen and enter content mode
            match self.current_screen {
                Screen::Models => {
                    // Stay on Models, enter content mode to navigate usage selection
                }
                Screen::Cleanup => {
                    // Stay on Cleanup, enter content mode to navigate the menu
                }
                Screen::Config => {
                    // Stay on Config, enter content mode to navigate the menu
                }
                Screen::Tools => {
                    // Stay on Tools, enter content mode
                    self.sidebar_focus = false;
                    self.selected_tool = Some(0); // Select first tool
                }
                Screen::Terminal => {
                    // Entering Terminal screen - automatically enter content mode
                    self.sidebar_focus = false;
                }
                Screen::Language => {
                    // Language screen has its own list
                }
                Screen::Quit => {
                    self.should_quit = true;
                    return;
                }
                Screen::Information => {
                    // Information screen shows static content, no content selection needed
                    return;
                }
                _ => {}
            }
        } else {
            // In content - handle screen-specific selection
            match self.current_screen {
                Screen::Tools => {
                    let tools = crate::tools::get_available_tools(&self.state.lock().unwrap().clone(), &self.i18n);
                    if let Some(sel) = self.selected_tool {
                        if sel < tools.len() {
                            let tool = tools[sel].clone();
                            let model = self.state.lock().unwrap().last_model.clone();
                            
                            // Vérifier si l'outil est installé
                            if !tool.installed {
                                // Lancer l'installation - si c'est une commande avec sudo, quitter le TUI
                                if let Some(cmd) = crate::tools::get_install_command(&tool.id) {
                                    if cmd.contains("sudo") {
                                        // Les commandes avec sudo nécessitent un terminal interactif
                                        self.should_quit = true;
                                        self.exec_command = Some(cmd);
                                    } else {
                                        // Utiliser le terminal intégré
                                        self.current_screen = Screen::Terminal;
                                        self.sidebar_focus = false;
                                        self.exec_command = Some(cmd);
                                    }
                                }
                            } else {
                                // L'outil est installé
                                if let Some(cmd) = crate::tools::get_launch_command(&tool.id, model.as_deref()) {
                                    // Les commandes avec sudo quittent le TUI
                                    if cmd.contains("sudo") {
                                        self.should_quit = true;
                                        self.exec_command = Some(cmd);
                                    } else {
                                        self.current_screen = Screen::Terminal;
                                        self.sidebar_focus = false;
                                        self.exec_command = Some(cmd);
                                    }
                                }
                            }
                        } else {
                            // "Retour" selected
                            self.current_screen = Screen::Information;
                            self.sidebar_focus = true;
                            self.selected_tool = None;
                        }
                    }
                }
                Screen::Language => {
                    let langs = crate::config::i18n::get_available_languages();
                    if let Some(sel) = self.selected_tool {
                        if sel < langs.len() {
                            // Change language
                            let lang = langs[sel].code.clone();
                            self.state.lock().unwrap().language = Some(lang);
                        }
                        // Last item is Retour - handled below
                    }
                }
                Screen::Models => {
                    // Handle based on workflow step
                    match self.model_workflow_step {
                        ModelWorkflowStep::UsageSelection => {
                            // Selecting usage type - stay on Models, move to ModelSelection
                            if let Some(sel) = self.selected_tool {
                                if sel < 4 {
                                    let usage = match sel {
                                        0 => "agent",   // Agents rapides
                                        1 => "book",    // Gros livre
                                        2 => "code",    // Grand codebase
                                        3 => "chat",    // Usage général
                                        _ => "chat",
                                    };
                                    self.state.lock().unwrap().last_usage = Some(usage.to_string());
                                    // Transition to model selection step
                                    self.model_workflow_step = ModelWorkflowStep::ModelSelection;
                                    // Reset selection for model list (default to 0, even if empty)
                                    self.selected_model = Some(0);
                                    self.selected_tool = Some(0);
                                }
                            }
                        }
                        ModelWorkflowStep::ModelSelection => {
                            // Selecting model - set as current and return to Information
                            if self.models.is_empty() {
                                // No models installed - launch setup externally
                                self.exec_command = Some("wzllama setup".to_string());
                                self.should_quit = true;
                            } else if let Some(model_idx) = self.selected_model {
                                if model_idx < self.models.len() {
                                    let chosen_model = self.models[model_idx].name.clone();
                                    self.state.lock().unwrap().last_model = Some(chosen_model);
                                    // Return to Information screen
                                    self.current_screen = Screen::Information;
                                    self.sidebar_focus = true;
                                    self.selected_tool = None;
                                    self.selected_model = None;
                                    self.model_workflow_step = ModelWorkflowStep::UsageSelection;
                                }
                            }
                        }
                    }
                }
                Screen::Cleanup => {
                    // 3 items, pas de Retour
                    if let Some(sel) = self.selected_tool {
                        if sel < 3 {
                            // Launch cleanup action in subprocess
                            let cmd = match sel {
                                0 => "wzllama cleanup tools".to_string(),    // Désinstaller outils
                                1 => "wzllama cleanup fleets".to_string(),   // Supprimer flottes
                                2 => "wzllama cleanup models".to_string(),   // Supprimer modèles
                                _ => "wzllama cleanup".to_string(),
                            };
                            self.exec_command = Some(cmd);
                            self.should_quit = true;
                        }
                    }
                }
                Screen::Config => {
                    // 7 items, pas de Retour
                    if let Some(sel) = self.selected_tool {
                        if sel < 7 {
                            // Launch config action in subprocess
                            let cmd = match sel {
                                0 => "wzllama config models".to_string(),     // Modèles par usage
                                1 => "wzllama config performance".to_string(),  // Performance
                                2 => "wzllama config shells".to_string(),       // Shells
                                3 => "wzllama config env".to_string(),        // Régénérer env
                                4 => "wzllama uninstall".to_string(),         // Désinstaller wzllama
                                _ => "wzllama config".to_string(),
                            };
                            self.exec_command = Some(cmd);
                            self.should_quit = true;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn get_current_screen_item_count(&self) -> usize {
        match self.current_screen {
            Screen::Models => {
                // In ModelSelection step, return models count
                if self.model_workflow_step == ModelWorkflowStep::ModelSelection {
                    self.models.len().max(1) // At least 1 to avoid division by zero
                } else {
                    4 // 4 usages
                }
            }
            Screen::Tools => {
                let tools = crate::tools::get_available_tools(&self.state.lock().unwrap().clone(), &self.i18n);
                tools.len()
            }
            Screen::Language => {
                let langs = crate::config::i18n::get_available_languages();
                langs.len()
            }
            Screen::Cleanup => 3, // 3 items cleanup
            Screen::Config => 7, // 7 items config
            Screen::Information => 1, // Just a description panel
            _ => 1,
        }
    }

    #[allow(dead_code)]
    fn get_tools(&self) -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("open-webui", "Open WebUI", "Interface web moderne pour Ollama"),
            ("openclaw", "OpenClaw", "Assistant terminal avec agents IA"),
            ("claude-code", "Claude Code", "Assistant de code IA puissant"),
            ("opencode", "OpenCode", "Agent IA open source"),
            ("mcp", "MCP Tools", "Model Context Protocol"),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Navigation {
    Up,
    Down,
    Left,
    Right,
    Quit,
    Search,
}