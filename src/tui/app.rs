use crate::config::{WzllamaState, I18n};
use crate::core::hardware::HardwareInfo;
use crate::core::ollama_api;
use crate::tui::screens::Screen;
use std::sync::{Arc, Mutex};

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
}

impl App {
    pub fn new(state: WzllamaState, hw: HardwareInfo, i18n: I18n) -> Self {
        let models = ollama_api::get_models();
        Self {
            state: Arc::new(Mutex::new(state)),
            hw,
            i18n,
            current_screen: Screen::Main,
            should_quit: false,
            models,
            selected_model: None,
            selected_tool: None,
            selected_cleanup: None,
            search_query: String::new(),
            exec_command: None,
            sidebar_focus: true,
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
                // Tab: toggle focus between sidebar and content
                self.sidebar_focus = !self.sidebar_focus;
                if self.sidebar_focus {
                    self.selected_tool = None;
                } else {
                    self.selected_tool = Some(0);
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
            // Navigate in the tools list (content)
            match self.current_screen {
                Screen::Tools => {
                    let tools = crate::tools::get_available_tools(&self.state.lock().unwrap().clone(), &self.i18n);
                    let count = tools.len() + 1; // +1 for "Retour" item
                    self.selected_tool = Some(match self.selected_tool {
                        None => 0,
                        Some(i) if i == 0 => count - 1,
                        Some(i) => i - 1,
                    });
                }
                _ => {
                    let tools = crate::tools::get_available_tools(&self.state.lock().unwrap().clone(), &self.i18n);
                    let count = tools.len().max(1);
                    self.selected_tool = Some(match self.selected_tool {
                        None => 0,
                        Some(i) if i == 0 => count - 1,
                        Some(i) => i - 1,
                    });
                }
            }
        }
    }

    /// Navigate down - depends on focus mode
    fn move_down(&mut self) {
        if self.sidebar_focus {
            // Navigate in the sidebar menu (left panel)
            self.current_screen = self.current_screen.next_menu();
        } else {
            // Navigate in the tools list (content)
            match self.current_screen {
                Screen::Tools => {
                    let tools = crate::tools::get_available_tools(&self.state.lock().unwrap().clone(), &self.i18n);
                    let count = tools.len() + 1; // +1 for "Retour" item
                    self.selected_tool = Some(match self.selected_tool {
                        None => 0,
                        Some(i) if i >= count - 1 => 0,
                        Some(i) => i + 1,
                    });
                }
                _ => {
                    let tools = crate::tools::get_available_tools(&self.state.lock().unwrap().clone(), &self.i18n);
                    let count = tools.len().max(1);
                    self.selected_tool = Some(match self.selected_tool {
                        None => 0,
                        Some(i) if i >= count - 1 => 0,
                        Some(i) => i + 1,
                    });
                }
            }
        }
    }

    fn go_back(&mut self) {
        // Left/Esc: return to sidebar navigation if in content mode
        // If on Main screen with sidebar focus, Esc quits
        if self.current_screen == Screen::Main && self.sidebar_focus {
            self.should_quit = true;
        } else {
            self.sidebar_focus = true;
            self.selected_tool = None;
        }
    }

    fn select(&mut self) {
        // Right/Enter: enter content mode or launch tool
        if self.current_screen == Screen::Main && self.sidebar_focus {
            // Entering content mode from Main
            self.sidebar_focus = false;
            self.selected_tool = Some(0);
        } else if !self.sidebar_focus {
            // In content mode - handle selection based on screen
            match self.current_screen {
                Screen::Tools => {
                    // Tools screen - check if Back is selected
                    let tools = crate::tools::get_available_tools(&self.state.lock().unwrap().clone(), &self.i18n);
                    if let Some(sel) = self.selected_tool {
                        if sel < tools.len() {
                            // Launch tool
                            let tool = tools[sel].clone();
                            match tool.id.as_str() {
                                "openclaw" => {
                                    let model = self.state.lock().unwrap().last_model.clone();
                                    let cmd = if let Some(m) = model {
                                        format!("ollama launch openclaw --model {}", m)
                                    } else {
                                        "ollama launch openclaw".to_string()
                                    };
                                    self.should_quit = true;
                                    self.exec_command = Some(cmd);
                                }
                                "open_webui" => {
                                    self.exec_command = Some("xdg-open http://localhost:3000".to_string());
                                    self.should_quit = true;
                                }
                                "claude_code" => {
                                    self.exec_command = Some("claude".to_string());
                                    self.should_quit = true;
                                }
                                "opencode" => {
                                    self.exec_command = Some("opencode".to_string());
                                    self.should_quit = true;
                                }
                                _ => {
                                    if tool.installed {
                                        if let Some(t) = crate::tools::get_tool(&tool.id) {
                                            let _ = t.launch(&self.i18n, &self.state.lock().unwrap().clone(), 
                                                self.state.lock().unwrap().last_model.as_deref());
                                        }
                                    }
                                }
                            }
                        } else {
                            // "Retour" selected - go back to Main
                            self.current_screen = Screen::Main;
                            self.sidebar_focus = true;
                            self.selected_tool = None;
                        }
                    }
                }
                _ => {}
            }
        } else {
            // Entering sidebar screens from navigation
            match self.current_screen {
                Screen::Models => self.current_screen = Screen::Models,
                Screen::Tools => self.current_screen = Screen::Tools,
                Screen::Config => self.current_screen = Screen::Config,
                Screen::Cleanup => self.current_screen = Screen::Cleanup,
                Screen::Language => self.sidebar_focus = false,
                Screen::Quit => {
                    self.should_quit = true;
                }
                _ => {}
            }
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
    #[allow(dead_code)]
    Search,
}