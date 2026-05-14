use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::Terminal;
use crate::tui::App;
use crate::tui::event::{EventHandler, key_to_nav, AppEvent};
use crate::tui::screens::Screen;
use crate::tui::app::ModelWorkflowStep;
use crate::core::system;
use crate::core::shell;
use crate::config::{WzllamaState, I18n};
use std::io::stdout;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::sync::{Arc, Mutex};
use std::thread;
use crossterm::event::KeyCode;
extern crate libc;

pub fn run_tui(state: WzllamaState, hw: crate::core::hardware::HardwareInfo, i18n: I18n) -> anyhow::Result<()> {
    let mut app = App::new(state, hw, i18n);
    let mut event_handler = EventHandler::new(100);
    let mut ctrl_d_count = 0;
    let mut terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;
    
    'tui_loop: loop {
        if !app.should_quit {
            crossterm::terminal::enable_raw_mode()?;
            terminal.hide_cursor()?;
        }
        
        // Clear screen at startup to remove terminal artifacts
        let _ = crossterm::execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
        
        // Inner loop for TUI interaction
        loop {
            // Always draw on each iteration
            terminal.draw(|frame| {
                render_app(frame, &mut app);
            })?;
            
            match event_handler.next() {
                AppEvent::Key(key) => {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        // Check for Ctrl-C: typically 'c' with CONTROL modifier
                        let is_ctrl_c = matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C')) 
                            && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL);
                        
                        // Check for Ctrl-D
                        let is_ctrl_d = matches!(key.code, KeyCode::Char('d') | KeyCode::Char('D')) 
                            && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL);
                        
                        if is_ctrl_c {
                            app.should_quit = true;
                        } else if is_ctrl_d {
                            ctrl_d_count += 1;
                            if ctrl_d_count >= 2 {
                                app.should_quit = true;
                            }
                        } else {
                            ctrl_d_count = 0; // Reset counter on other keys
                            
                            // Handle terminal screen input
                            if app.current_screen == Screen::Terminal && !app.sidebar_focus {
                                match key.code {
                                    KeyCode::Esc => {
                                        // Return to sidebar mode
                                        app.sidebar_focus = true;
                                        app.exec_command = None; // Clear any pending command
                                        app.terminal_input.clear();
                                    }
                                    KeyCode::Enter => {
                                        // If there's a pending exec_command, copy it to input first
                                        if let Some(ref cmd) = app.exec_command {
                                            app.terminal_input = cmd.clone();
                                            app.exec_command = None;
                                        }
                                        
                                        // Execute the command in the buffer
                                        let cmd = app.terminal_input.clone();
                                        if !cmd.is_empty() {
                                            // Afficher la commande immédiatement
                                            {
                                                let mut out = app.terminal.output.lock().unwrap();
                                                out.push_str(&format!("$ {}\n", cmd));
                                            }
                                            
                                            // Exécution SYNCHRONÈQUE avec capture d'erreur
                                            // Utiliser sh directement, pas le shell utilisateur
                                            let result = std::process::Command::new("/bin/sh")
                                                .args(["-c", &cmd])
                                                .current_dir(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")))
                                                .output();
                                            
                                            match result {
                                                Ok(output) => {
                                                    let stdout = String::from_utf8_lossy(&output.stdout);
                                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                                    let mut out = app.terminal.output.lock().unwrap();
                                                    let pwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
                                                    out.push_str(&format!("[pwd: {}]\n", pwd.display()));
                                                    if !stdout.is_empty() {
                                                        out.push_str(&stdout);
                                                        out.push('\n');
                                                    } else {
                                                        out.push_str("(no stdout)\n");
                                                    }
                                                    if !stderr.is_empty() {
                                                        out.push_str("stderr: ");
                                                        out.push_str(&stderr);
                                                        out.push('\n');
                                                    }
                                                    if output.status.code() != Some(0) {
                                                        out.push_str(&format!("[exit code: {}]\n", output.status.code().unwrap_or(-1)));
                                                    }
                                                }
                                                Err(e) => {
                                                    let mut out = app.terminal.output.lock().unwrap();
                                                    out.push_str(&format!("Error executing command: {}\n", e));
                                                }
                                            }
                                            app.terminal_input.clear();
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        app.terminal_input.pop();
                                    }
                                    KeyCode::Char(c) => {
                                        app.terminal_input.push(c);
                                    }
                                    _ => {}
                                }
                            } else if let Some(nav) = key_to_nav(key) {
                                app.navigate(nav);
                            }
                        }
                    }
                }
                AppEvent::Resize(_w, _h) => {
                    // Resize is handled by the next draw cycle automatically
                }
                AppEvent::Tick => app.tick(),
            }
            
            if app.should_quit {
                break;
            }
        }
        
        // Execute the command if Terminal screen was selected
        if let Some(cmd) = &app.exec_command {
            terminal.show_cursor()?;
            crossterm::terminal::disable_raw_mode()?;
            let _ = crossterm::execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
            println!();
            
            if cmd == "bash" {
                // Exec bash directly (remplace le processus TUI)
                // Le terminal a été remis en mode normal par disable_raw_mode()
                let _ = Command::new("/bin/bash").exec();
            } else {
                // Other commands - replace process
                shell::spawn_and_exit(cmd);
            }
            
            // After shell returns, normal exit
            break;
        }
        
        // Normal exit (Quit screen)
        break;
    }
    
    terminal.show_cursor()?;
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
    println!();
    
    Ok(())
}

fn render_app(frame: &mut Frame, app: &mut App) {
    let area = frame.size();
    
    // Check minimum terminal size - use compact layout for small terminals
    if area.width < 40 || area.height < 10 {
        let msg = Paragraph::new("Terminal too small! Min 40x10 required.\nResize for better experience.")
            .block(Block::default().borders(Borders::ALL).title("Error"));
        frame.render_widget(msg, area);
        return;
    }
    
    let constraints = if area.width > 100 {
        // Wide terminal: sidebar + main
        [Constraint::Percentage(25), Constraint::Percentage(75)]
    } else if area.width > 60 {
        // Medium terminal: balanced sidebar + main
        [Constraint::Percentage(35), Constraint::Percentage(65)]
    } else {
        // Small terminal: minimal sidebar, larger main
        [Constraint::Length(15), Constraint::Min(20)]
    };
    
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);
    
    // Sidebar with resources and navigation
    render_sidebar(frame, app, layout[0]);
    
    // Main content based on screen
    render_main(frame, app, layout[1]);
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = if area.width > 60 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Length(6),   // Resources (compact)
                Constraint::Min(0),        // Menu
            ])
            .split(area)
    } else {
        // Very small sidebar - minimal
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),   // Title (compact)
                Constraint::Min(0),      // Menu only
            ])
            .split(area)
    };
    
    // Title with color bar
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title_alignment(Alignment::Center);
    
    let title = Paragraph::new(
        Line::from(vec![
            Span::styled("🚀 ", Style::default().fg(Color::Magenta)),
            Span::styled("wzllama", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])
    ).block(title_block).alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);
    
    // Resources panel only for width > 60
    if area.width > 60 {
        let ram_total = app.hw.ram_gb;
        let ram_avail = system::get_available_ram_gb();
        let ram_used = ram_total - ram_avail;
        let ram_ratio = if ram_total > 0.0 { ram_used / ram_total } else { 0.0 };
        
        let vram_total = app.hw.total_vram_mb as f64 / 1024.0;
        let vram_avail = system::get_available_vram_gb().unwrap_or(0.0);
        let vram_used = (vram_total - vram_avail).max(0.0);
        let vram_ratio = if vram_total > 0.0 { vram_used / vram_total } else { 0.0 };
        
        let resources_text = vec![
            Line::from(vec![
                Span::styled("💾 RAM ", Style::default().fg(Color::Cyan)),
                Span::styled("█".repeat(((ram_ratio * 10.0) as usize).min(10)), Style::default().fg(Color::Yellow)),
                Span::styled("░".repeat((10 - (ram_ratio * 10.0) as usize).max(0)), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("  {:.1}/{:.1} Go", ram_avail, ram_total), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("🎮 GPU ", Style::default().fg(Color::Cyan)),
                Span::styled("█".repeat(((vram_ratio * 10.0) as usize).min(10)), Style::default().fg(Color::Green)),
                Span::styled("░".repeat((10 - (vram_ratio * 10.0) as usize).max(0)), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("  {:.1}/{:.1} Go", vram_avail, vram_total), Style::default().fg(Color::White)),
            ]),
            Line::from(Span::styled("🤖 Current Model", Style::default().fg(Color::Magenta))),
            Line::from(format!("   {}", app.state.lock().unwrap().last_model.clone().unwrap_or_else(|| "none".into()))),
        ];
        
        let resources = Paragraph::new(resources_text)
            .block(Block::default().borders(Borders::ALL).title("📊 Resources").border_style(Style::default().fg(Color::DarkGray)));
        frame.render_widget(resources, chunks[1]);
    }
    
    // Menu navigation - use appropriate chunk index
    let menu_chunk = if area.width > 60 { chunks[2] } else { chunks[1] };
    
    let menu_items: Vec<ListItem> = [
        ("📖 Information", Screen::Information, Color::Cyan),
        ("🤖 Models", Screen::Models, Color::Magenta),
        ("🛠️ Tools", Screen::Tools, Color::Yellow),
        ("💻 Terminal", Screen::Terminal, Color::Green),
        ("🧹 Cleanup", Screen::Cleanup, Color::Red),
        ("⚙️ Config", Screen::Config, Color::Green),
        ("🌍 Language", Screen::Language, Color::Cyan),
        ("🚪 Quit", Screen::Quit, Color::DarkGray),
    ].iter().map(|(text, screen, color)| {
        let style = if app.current_screen == *screen {
            Style::default().fg(*color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(Line::from(Span::styled(*text, style)))
    }).collect();
    
    let menu = List::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title("Navigation").border_style(Style::default().fg(Color::Blue)));
    frame.render_widget(menu, menu_chunk);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.current_screen {
        Screen::Information => render_main_screen(frame, app, area),
        Screen::Models => {
            // Render based on workflow step
            if app.model_workflow_step == ModelWorkflowStep::ModelSelection {
                render_model_select_screen(frame, app, area)
            } else {
                render_model_usage_screen(frame, app, area)
            }
        }
        Screen::Tools => render_tools_screen(frame, app, area),
        Screen::Terminal => render_terminal_screen(frame, app, area),
        Screen::Cleanup => render_cleanup_menu_screen(frame, app, area),
        Screen::Config => render_config_menu_screen(frame, app, area),
        Screen::Language => render_language_screen(frame, app, area),
        Screen::Quit => render_quit_screen(frame, area),
        _ => render_placeholder(frame, area),
    }
}

fn render_main_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);
    
    // Header
    let focus_indicator = if app.sidebar_focus {
        " [NAVIGATE MENU: ↑↓ change selection]"
    } else {
        " [CONTENT MODE]"
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled("wzllama - Assistant IA Local", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("🎯 ↑↓ to navigate menu, Enter selects, Esc quits"),
        Line::from("🚪 Ctrl-D twice or Ctrl-C to quit"),
    ])
    .block(Block::default().borders(Borders::ALL).title(format!("📖 Information{}", focus_indicator)).border_style(Style::default().fg(Color::Cyan)));
    frame.render_widget(header, chunks[0]);
    
    // wzllama description panel
    let desc_text = vec![
        Line::from(Span::styled("🚀 wzllama", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Assistant IA local multi-modèles basé sur Ollama."),
        Line::from(""),
        Line::from(Span::styled("Fonctionnalités :", Style::default().fg(Color::Cyan))),
        Line::from("  • Chat avec vos modèles IA locaux"),
        Line::from("  • Outils d'assistance développement"),
        Line::from("  • Gestion des modèles et ressources"),
        Line::from("  • Interface unifiée et intuitive"),
        Line::from(""),
        Line::from(Span::styled("Navigation :", Style::default().fg(Color::Cyan))),
        Line::from("  ←→↑↓ : Naviguer dans le menu"),
        Line::from("  Tab   : Basculer focus sidebar/content"),
        Line::from("  Enter : Sélectionner"),
        Line::from("  Esc   : Retour/Quitter"),
    ];
    
    let desc = Paragraph::new(desc_text)
        .block(Block::default().borders(Borders::ALL).title("📖 À propos").border_style(Style::default().fg(Color::Magenta)));
    frame.render_widget(desc, chunks[1]);
}

fn render_tools_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),     // Content
        ])
        .split(area);
    
    // Header with navigation instructions
    let focus_indicator = if app.sidebar_focus {
        " [NAVIGATE MENU: ↑↓ change selection]"
    } else {
        " [CONTENT MODE: ↑↓ navigate, ← or Esc to return]"
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled("🛠️ Lancer un outil", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("🎯 ←→↑↓ to navigate, Tab to toggle focus"),
        Line::from("🚪 Esc to return to sidebar"),
    ])
    .block(Block::default().borders(Borders::ALL).title(format!("🛠️ Tools{}", focus_indicator)).border_style(Style::default().fg(Color::Yellow)));
    frame.render_widget(header, chunks[0]);
    
    // Tools list - shows available tools for quick launch
    let tools = crate::tools::get_available_tools(&app.state.lock().unwrap().clone(), &app.i18n);
    
    let tool_items: Vec<ListItem> = tools.iter().enumerate().map(|(i, t)| {
        let icon = if t.installed { "✅" } else { "📦" };
        let prefix = if Some(i) == app.selected_tool { "> " } else { "  " };
        let style = if Some(i) == app.selected_tool {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(icon, style),
            Span::styled(&t.name, style),
            Span::styled(format!(" — {}", t.description), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();
    
    let tool_list = List::new(tool_items)
        .block(Block::default().borders(Borders::ALL).title("📋 Sélectionnez un outil").border_style(Style::default().fg(Color::Yellow)));
    frame.render_widget(tool_list, chunks[1]);
}

fn render_placeholder(frame: &mut Frame, area: Rect) {
    let content = Paragraph::new("Screen not implemented yet")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(content, area);
}

fn render_language_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),     // Content
        ])
        .split(area);
    
    // Header with navigation instructions
    let focus_indicator = if app.sidebar_focus {
        " [NAVIGATE MENU: ↑↓ change selection]"
    } else {
        " [CONTENT MODE: ↑↓ navigate, ← or Esc to return]"
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled("🌍 Changer de langue", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("🎯 ←→↑↓ to navigate, Tab to toggle focus"),
        Line::from("🚪 Esc to return to sidebar"),
    ])
    .block(Block::default().borders(Borders::ALL).title(format!("🌍 Language{}", focus_indicator)).border_style(Style::default().fg(Color::Cyan)));
    frame.render_widget(header, chunks[0]);
    
    let current_lang = app.state.lock().unwrap().language.clone().unwrap_or_else(|| "fr".into());
    let languages = crate::config::i18n::get_available_languages();
    
    let lang_items: Vec<ListItem> = languages.iter().map(|lang| {
        let marker = if lang.code == current_lang { "✓ " } else { "  " };
        ListItem::new(Line::from(vec![
            Span::styled(marker, Style::default().fg(Color::Green)),
            Span::styled(&lang.name, Style::default().fg(Color::White)),
            Span::styled(format!(" ({})", lang.code), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();
    
    let lang_list = List::new(lang_items)
        .block(Block::default().borders(Borders::ALL).title("📋 Sélectionnez une langue").border_style(Style::default().fg(Color::Cyan)));
    frame.render_widget(lang_list, chunks[1]);
}

fn render_quit_screen(frame: &mut Frame, area: Rect) {
    let content = Paragraph::new("Press Enter to quit wzllama")
        .block(Block::default().borders(Borders::ALL).title("🚪 Quit").border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(content, area);
}

// Sub-screen render functions
fn render_model_usage_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),     // Content
        ])
        .split(area);
    
    // Header with navigation instructions
    let focus_indicator = if app.sidebar_focus {
        " [NAVIGATE MENU: ↑↓ change selection]"
    } else {
        " [CONTENT MODE: ↑↓ navigate, ← or Esc to return]"
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled("🤖 Choisissez votre usage", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("🎯 ←→↑↓ to navigate, Tab to toggle focus"),
        Line::from("🚪 Esc to return to sidebar"),
    ])
    .block(Block::default().borders(Borders::ALL).title(format!("🤖 Models{}", focus_indicator)).border_style(Style::default().fg(Color::Magenta)));
    frame.render_widget(header, chunks[0]);
    
    // Usage items
    let items: Vec<ListItem> = vec![
        ("  ⚡ Agents rapides", Color::Yellow),
        ("  📚 Gros livre", Color::White),
        ("  💻 Grand codebase", Color::White),
        ("  🎯 Usage général", Color::White),
    ].into_iter().enumerate().map(|(i, (text, base_color))| {
        let style = if Some(i) == app.selected_tool {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(base_color)
        };
        let prefix = if Some(i) == app.selected_tool { "> " } else { "  " };
        ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, text), style)))
    }).collect();
    
    let usage_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("📋 Sélectionnez un usage").border_style(Style::default().fg(Color::Magenta)));
    frame.render_widget(usage_list, chunks[1]);
}

fn render_model_select_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),     // Content
        ])
        .split(area);
    
    // Header with navigation instructions
    let focus_indicator = if app.sidebar_focus {
        " [NAVIGATE MENU: ↑↓ change selection]"
    } else {
        " [CONTENT MODE: ↑↓ navigate, ← or Esc to return]"
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled("🤖 Choisir un modèle IA", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("🎯 ←→↑↓ to navigate, Tab to toggle focus"),
        Line::from("🚪 Esc to return to sidebar"),
    ])
    .block(Block::default().borders(Borders::ALL).title(format!("🤖 Model Select{}", focus_indicator)).border_style(Style::default().fg(Color::Magenta)));
    frame.render_widget(header, chunks[0]);
    
    // Model items
    let model_items: Vec<ListItem> = app.models.iter().enumerate().map(|(i, m)| {
        let style = if Some(i) == app.selected_model {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if Some(i) == app.selected_model { "> " } else { "  " };
        let size_str = format!(" ({})", m.formatted_size());
        ListItem::new(Line::from(Span::styled(format!("{}{}{}", prefix, m.name, size_str), style)))
    }).collect();
    
    if model_items.is_empty() {
        // Proposer d'exécuter le setup pour installer des modèles
        let empty_items = vec![
            ListItem::new(Line::from(Span::styled("> 🚀 Installer des modèles (wzllama setup)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))),
        ];
        
        let empty = List::new(empty_items)
            .block(Block::default().borders(Borders::ALL).title("📋 Aucun modèle installé").border_style(Style::default().fg(Color::DarkGray)));
        frame.render_widget(empty, chunks[1]);
    } else {
        let model_list = List::new(model_items)
            .block(Block::default().borders(Borders::ALL).title("📋 Modèles installés").border_style(Style::default().fg(Color::Magenta)));
        frame.render_widget(model_list, chunks[1]);
    }
}

fn render_cleanup_menu_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),     // Content
        ])
        .split(area);
    
    // Header with navigation instructions
    let focus_indicator = if app.sidebar_focus {
        " [NAVIGATE MENU: ↑↓ change selection]"
    } else {
        " [CONTENT MODE: ↑↓ navigate, ← or Esc to return]"
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled("🧹 Nettoyage", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("🎯 ←→↑↓ to navigate, Tab to toggle focus"),
        Line::from("🚪 Esc to return to sidebar"),
    ])
    .block(Block::default().borders(Borders::ALL).title(format!("🧹 Cleanup{}", focus_indicator)).border_style(Style::default().fg(Color::Red)));
    frame.render_widget(header, chunks[0]);
    
    // Cleanup items
    let items: Vec<ListItem> = vec![
        ("  🗑️  Désinstaller des outils", Color::Red),
        ("  📂 Supprimer des flottes", Color::Yellow),
        ("  🤖 Supprimer des modèles", Color::Magenta),
    ].into_iter().enumerate().map(|(i, (text, base_color))| {
        let style = if Some(i) == app.selected_tool {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(base_color)
        };
        let prefix = if Some(i) == app.selected_tool { "> " } else { "  " };
        ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, text), style)))
    }).collect();
    
    let cleanup_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("📋 Options de nettoyage").border_style(Style::default().fg(Color::Red)));
    frame.render_widget(cleanup_list, chunks[1]);
}

fn render_config_menu_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),     // Content
        ])
        .split(area);
    
    // Header with navigation instructions
    let focus_indicator = if app.sidebar_focus {
        " [NAVIGATE MENU: ↑↓ change selection]"
    } else {
        " [CONTENT MODE: ↑↓ navigate, ← or Esc to return]"
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled("⚙️ Configuration", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("🎯 ←→↑↓ to navigate, Tab to toggle focus"),
        Line::from("🚪 Esc to return to sidebar"),
    ])
    .block(Block::default().borders(Borders::ALL).title(format!("⚙️ Config{}", focus_indicator)).border_style(Style::default().fg(Color::Green)));
    frame.render_widget(header, chunks[0]);
    
    let state = app.state.lock().unwrap();
    let code_model = state.last_model.clone().unwrap_or_else(|| "qwen2.5-coder:14b".into());
    
    let items: Vec<ListItem> = vec![
        format!("  🔧 Host: 127.0.0.1:11434",),
        format!("  🤖 Code: {}", code_model),
        "  🔄 Modèles par usage".to_string(),
        "  ⚡ Performance".to_string(),
        "  📂 Shells".to_string(),
        "  📄 Régénérer ~/.wzllama/env".to_string(),
        "  🗑️ Désinstaller wzllama".to_string(),
    ].into_iter().enumerate().map(|(i, text)| {
        let base_color = match i {
            0..=1 => Color::White,
            2 => Color::Cyan,
            3 => Color::Yellow,
            4 => Color::Green,
            5 => Color::Magenta,
            6 => Color::Red,
            _ => Color::White,
        };
        let style = if Some(i) == app.selected_tool {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(base_color)
        };
        let prefix = if Some(i) == app.selected_tool { "> " } else { "  " };
        ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, text), style)))
    }).collect();
    
    let config_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("📋 Options de configuration").border_style(Style::default().fg(Color::Green)));
    frame.render_widget(config_list, chunks[1]);
}

/// Render the integrated terminal screen
fn render_terminal_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    
    // Command input - show exec_command if set (from tools menu), otherwise show terminal_input
    let display_input = if let Some(ref cmd) = app.exec_command {
        cmd.clone()
    } else {
        app.terminal_input.clone()
    };
    
    // Clone the output for rendering (avoid holding lock during render)
    let output_content: String = {
        let output = app.terminal.output.lock().unwrap();
        let content = output.clone();
        // Keep only last 100 lines
        let lines: Vec<&str> = content.lines().rev().take(100).collect();
        if lines.is_empty() {
            // Show current directory and examples
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
            format!("Terminal intégré\nRépertoire: {}\n\nExemples:\n  ls          - lister les fichiers\n  pwd         - afficher le répertoire\n  whoami      - qui êtes-vous\n\nTapez une commande et appuyez sur Entrée", cwd.display())
        } else {
            lines.into_iter().rev().collect::<Vec<_>>().join("\n")
        }
    };
    let output_text = Paragraph::new(output_content)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("💻 Terminal").border_style(Style::default().fg(Color::Green)));
    frame.render_widget(output_text, chunks[0]);
    
    let input_text = Paragraph::new(format!("> {}", display_input))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Command (Enter to run, Esc to exit)").border_style(Style::default().fg(Color::Blue)));
    frame.render_widget(input_text, chunks[1]);
}