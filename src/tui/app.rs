use crate::config::WzllamaState;
use crate::core::hardware::HardwareInfo;
use crate::core::ollama_api;
use crate::tui::screens::Screen;
use std::sync::{Arc, Mutex};

pub struct App {
    pub state: Arc<Mutex<WzllamaState>>,
    pub hw: HardwareInfo,
    pub current_screen: Screen,
    pub should_quit: bool,
    pub models: Vec<ollama_api::OllamaModel>,
    pub selected_model: Option<usize>,
    pub selected_tool: Option<usize>,
    pub selected_cleanup: Option<usize>,
    pub search_query: String,
}

impl App {
    pub fn new(state: WzllamaState, hw: HardwareInfo) -> Self {
        let models = ollama_api::get_models();
        Self {
            state: Arc::new(Mutex::new(state)),
            hw,
            current_screen: Screen::Main,
            should_quit: false,
            models,
            selected_model: None,
            selected_tool: None,
            selected_cleanup: None,
            search_query: String::new(),
        }
    }

    pub fn tick(&mut self) {
        // Refresh models periodically
        self.models = ollama_api::get_models();
    }

    pub fn navigate(&mut self, direction: Navigation) {
        match direction {
            Navigation::Up => self.move_up(),
            Navigation::Down => self.move_down(),
            Navigation::Left => self.go_back(),
            Navigation::Right => self.select(),
            Navigation::Quit => self.should_quit = true,
            Navigation::Search => {
                // Enter search mode - could be expanded
            }
        }
    }

    fn move_up(&mut self) {
        match self.current_screen {
            Screen::Models | Screen::ModelSelect => {
                let count = self.models.len().max(1);
                self.selected_model = Some(match self.selected_model {
                    None => 0,
                    Some(i) if i == 0 => count - 1,
                    Some(i) => i - 1,
                });
            }
            Screen::Tools | Screen::ToolSelect => {
                let tools = self.get_tools();
                let count = tools.len().max(1);
                self.selected_tool = Some(match self.selected_tool {
                    None => 0,
                    Some(i) if i == 0 => count - 1,
                    Some(i) => i - 1,
                });
            }
            _ => {
                self.current_screen = self.current_screen.previous();
            }
        }
    }

    fn move_down(&mut self) {
        match self.current_screen {
            Screen::Models | Screen::ModelSelect => {
                let count = self.models.len().max(1);
                self.selected_model = Some(match self.selected_model {
                    None => 0,
                    Some(i) if i >= count - 1 => 0,
                    Some(i) => i + 1,
                });
            }
            Screen::Tools | Screen::ToolSelect => {
                let tools = self.get_tools();
                let count = tools.len().max(1);
                self.selected_tool = Some(match self.selected_tool {
                    None => 0,
                    Some(i) if i >= count - 1 => 0,
                    Some(i) => i + 1,
                });
            }
            _ => {
                self.current_screen = self.current_screen.next();
            }
        }
    }

    fn go_back(&mut self) {
        self.current_screen = Screen::Main;
    }

    fn select(&mut self) {
        match self.current_screen {
            Screen::Main => self.current_screen = Screen::Models,
            Screen::Models => self.current_screen = Screen::ModelSelect,
            Screen::Tools => self.current_screen = Screen::ToolSelect,
            Screen::Config => self.current_screen = Screen::ConfigEdit,
            Screen::Cleanup => self.current_screen = Screen::CleanupSelect,
            Screen::ModelSelect => {
                // Action on selected model
                if let Some(idx) = self.selected_model {
                    if idx < self.models.len() {
                        let model = self.models[idx].name.clone();
                        let _ = ollama_api::pull_model(&model);
                    }
                }
            }
            _ => {}
        }
    }

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