use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::Terminal;
use crate::tui::App;
use crate::tui::event::{EventHandler, key_to_nav, AppEvent};
use crate::tui::screens::Screen;
use crate::core::system;
use crate::core::ollama_api;
use crate::core::shell;
use crate::config::{WzllamaState, I18n};
use crate::tools;
use std::io::stdout;
use crossterm::event::KeyCode;

pub fn run_tui(state: WzllamaState, hw: crate::core::hardware::HardwareInfo, i18n: I18n) -> anyhow::Result<()> {
    let mut app = App::new(state, hw, i18n);
    let mut event_handler = EventHandler::new(100);
    let mut ctrl_d_count = 0;
    
    let mut terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;
    
    crossterm::terminal::enable_raw_mode()?;
    terminal.hide_cursor()?;
    
    // Clear screen at startup to remove terminal artifacts
    let _ = crossterm::execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
    
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
                        
                        if let Some(nav) = key_to_nav(key) {
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
    
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    // Clear the terminal to remove any artifacts after exiting raw mode
    let _ = crossterm::execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
    // Print a newline to ensure clean prompt
    println!();
    
    // Execute the command if Exec screen was selected
    if let Some(cmd) = app.exec_command {
        // Use shell::exec to replace current process with the command
        shell::exec(&cmd);
    }
    
    Ok(())
}

fn render_app(frame: &mut Frame, app: &mut App) {
    let area = frame.size();
    
    // Check minimum terminal size
    if area.width < 60 || area.height < 20 {
        let msg = Paragraph::new("Terminal too small! Min 60x20 required.")
            .block(Block::default().borders(Borders::ALL).title("Error"));
        frame.render_widget(msg, area);
        return;
    }
    
    let constraints = if area.width > 100 {
        // Wide terminal: sidebar + main
        [Constraint::Percentage(25), Constraint::Percentage(75)]
    } else {
        // Narrow terminal: stacked
        [Constraint::Percentage(30), Constraint::Percentage(70)]
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Title
            Constraint::Length(12),  // Resources
            Constraint::Min(0),    // Menu
        ])
        .split(area);
    
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
    
    // Resources panel with colored bars
    let ram_total = app.hw.ram_gb;
    let ram_avail = system::get_available_ram_gb();
    let ram_used = ram_total - ram_avail;
    let ram_ratio = if ram_total > 0.0 { ram_used / ram_total } else { 0.0 };
    
    let vram_total = app.hw.total_vram_mb as f64 / 1024.0;
    let vram_avail = system::get_available_vram_gb().unwrap_or(0.0);
    let vram_used = (vram_total - vram_avail).max(0.0);
    let vram_ratio = if vram_total > 0.0 { vram_used / vram_total } else { 0.0 };
    
    let running = ollama_api::get_running_models();
    
    let resources_text = vec![
        Line::from(Span::styled("💾 RAM", Style::default().fg(Color::Cyan))),
        Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled("█".repeat(((ram_ratio * 20.0) as usize).min(20)), Style::default().fg(Color::Yellow)),
            Span::styled("░".repeat((20 - (ram_ratio * 20.0) as usize).max(0)), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(format!("   {:.1}/{:.1} Go", ram_avail, ram_total)),
        Line::from(Span::styled("🎮 GPU", Style::default().fg(Color::Cyan))),
        Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled("█".repeat(((vram_ratio * 20.0) as usize).min(20)), Style::default().fg(Color::Green)),
            Span::styled("░".repeat((20 - (vram_ratio * 20.0) as usize).max(0)), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(format!("   {:.1}/{:.1} Go", vram_avail, vram_total)),
        Line::from(Span::styled("⚡ Loaded", Style::default().fg(Color::Cyan))),
        Line::from(format!("   {}", if running.is_empty() { "none".to_string() } else { running.join(", ") })),
        Line::from(""),
        Line::from(Span::styled("🤖 Current Model", Style::default().fg(Color::Magenta))),
        Line::from(format!("   {}", app.state.lock().unwrap().last_model.clone().unwrap_or_else(|| "none".into()))),
    ];
    
    let resources = Paragraph::new(resources_text)
        .block(Block::default().borders(Borders::ALL).title("📊 Resources").border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(resources, chunks[1]);
    
    // Menu navigation with colors
    let menu_items: Vec<ListItem> = [
        ("🏠 Main", Screen::Main, Color::Cyan),
        ("🤖 Models", Screen::Models, Color::Magenta),
        ("🛠️ Tools", Screen::Tools, Color::Yellow),
        ("⚙️ Config", Screen::Config, Color::Green),
        ("🧹 Cleanup", Screen::Cleanup, Color::Red),
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
    frame.render_widget(menu, chunks[2]);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.current_screen {
        Screen::Main => render_main_screen(frame, app, area),
        Screen::Models => render_model_usage_screen(frame, app, area),
        Screen::ModelUsage => render_model_usage_screen(frame, app, area),
        Screen::Tools => render_tools_screen(frame, app, area),
        Screen::Cleanup => render_cleanup_menu_screen(frame, app, area),
        Screen::CleanupTools => render_cleanup_tools_screen(frame, app, area),
        Screen::CleanupFleets => render_cleanup_fleets_screen(frame, app, area),
        Screen::CleanupModels => render_cleanup_models_screen(frame, app, area),
        Screen::Config => render_config_menu_screen(frame, app, area),
        Screen::ConfigModels => render_config_models_screen(frame, app, area),
        Screen::ConfigPerformance => render_config_performance_screen(frame, app, area),
        Screen::ConfigShells => render_config_shells_screen(frame, app, area),
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
        " [CONTENT MODE: ↑↓ navigate tools, ← or Esc to return]"
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled("wzllama - Assistant IA Local", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("🎯 hjkl or ←→↑↓ to navigate, Tab to toggle focus, Enter to select"),
        Line::from("🚪 Ctrl-D twice or Ctrl-C to quit, Esc on Main to quit"),
    ])
    .block(Block::default().borders(Borders::ALL).title(format!("🏠 Main{}", focus_indicator)).border_style(Style::default().fg(Color::Cyan)));
    frame.render_widget(header, chunks[0]);
    
    // Tools list - shows available tools for quick launch
    let tools = tools::get_available_tools(&app.state.lock().unwrap().clone(), &app.i18n);
    let tool_items: Vec<ListItem> = tools.iter().enumerate().map(|(i, t)| {
        let icon = if t.installed { "✅" } else { "📦" };
        let selected = if Some(i) == app.selected_tool { "▶ " } else { "  " };
        let style = if Some(i) == app.selected_tool {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(Line::from(vec![
            Span::styled(selected, style),
            Span::styled(icon, style),
            Span::styled(&t.name, style),
            Span::styled(format!(" — {}", t.description), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();
    
    let tools_list = List::new(tool_items)
        .block(Block::default().borders(Borders::ALL).title("🛠️ Tools (Enter to launch)").border_style(Style::default().fg(Color::Yellow)));
    frame.render_widget(tools_list, chunks[1]);
}

fn render_models_screen(frame: &mut Frame, _app: &App, area: Rect) {
    let models = ollama_api::get_models();
    
    let model_items: Vec<ListItem> = models.iter().map(|m| {
        let status = if ollama_api::is_model_running(&m.name) {
            Span::styled("✅ ", Style::default().fg(Color::Green))
        } else {
            Span::styled("⬇️ ", Style::default().fg(Color::Yellow))
        };
        
        ListItem::new(Line::from(vec![
            status,
            Span::styled(&m.name, Style::default().fg(Color::White)),
            Span::styled(format!(" ({})", m.formatted_size()), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();
    
    // Add usage sections
    let usage_items = vec![
        ListItem::new(Line::from(Span::styled("Choisissez votre usage ::", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))),
        ListItem::new(Line::from(Span::styled("  ⚡ Agents rapides", Style::default().fg(Color::Yellow)))),
        ListItem::new(Line::from(Span::styled("  📚 Gros livre", Style::default().fg(Color::White)))),
        ListItem::new(Line::from(Span::styled("  💻 Grand codebase", Style::default().fg(Color::White)))),
        ListItem::new(Line::from(Span::styled("  🎯 Usage général", Style::default().fg(Color::White)))),
        ListItem::new(Line::from(Span::styled("  ↩️ Retour", Style::default().fg(Color::White).add_modifier(Modifier::DIM)))),
    ];
    
    let all_items: Vec<ListItem> = model_items.into_iter().chain(usage_items.into_iter()).collect();
    
    let model_list = List::new(all_items)
        .block(Block::default().borders(Borders::ALL).title("🤖 Choisir un modèle IA").border_style(Style::default().fg(Color::Magenta)));
    frame.render_widget(model_list, area);
}

fn render_tools_screen(frame: &mut Frame, app: &App, area: Rect) {
    // Match the existing format: tools with installed status and navigation
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
    
    // Add Back option at the end
    let back_item = ListItem::new(Line::from(Span::styled("  ↩️  Retour", Style::default().fg(Color::White).add_modifier(Modifier::DIM))));
    let mut items: Vec<ListItem> = tool_items;
    items.push(back_item);
    
    let tool_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("🛠️  Lancer un outil (Enter to launch, Esc to go back)").border_style(Style::default().fg(Color::Yellow)));
    frame.render_widget(tool_list, area);
}

fn render_config_screen(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.state.lock().unwrap();
    let code_model = state.last_model.clone().unwrap_or_else(|| "qwen2.5-coder:14b".into());
    
    let config_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled("🔧 127.0.0.1:11434", Style::default().fg(Color::White)))),
        ListItem::new(Line::from(Span::styled(format!("🤖 Code: {}", code_model), Style::default().fg(Color::White)))),
        ListItem::new(Line::from(Span::styled("🔄 Modèles par usage", Style::default().fg(Color::Cyan)))),
        ListItem::new(Line::from(Span::styled("⚡ Performance", Style::default().fg(Color::Yellow)))),
        ListItem::new(Line::from(Span::styled("📂 Shells", Style::default().fg(Color::Green)))),
        ListItem::new(Line::from(Span::styled("📄 Regénérer ~/.wzllama/env", Style::default().fg(Color::Magenta)))),
        ListItem::new(Line::from(Span::styled("🗑️ Désinstaller wzllama", Style::default().fg(Color::Red)))),
        ListItem::new(Line::from(Span::styled("↩️ Retour", Style::default().fg(Color::White).add_modifier(Modifier::DIM)))),
    ];
    
    let config_list = List::new(config_items)
        .block(Block::default().borders(Borders::ALL).title("⚙️ Configuration").border_style(Style::default().fg(Color::Green)));
    frame.render_widget(config_list, area);
}

fn render_cleanup_screen(frame: &mut Frame, _app: &App, area: Rect) {
    let cleanup_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("🗑️  Désinstaller des outils", Style::default().fg(Color::Red)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("📂 Supprimer des flottes", Style::default().fg(Color::Yellow)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🤖 Supprimer des modèles", Style::default().fg(Color::Magenta)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("↩️  Retour", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
        ])),
    ];
    
    let cleanup_list = List::new(cleanup_items)
        .block(Block::default().borders(Borders::ALL).title("🧹 Nettoyage").border_style(Style::default().fg(Color::Red)));
    frame.render_widget(cleanup_list, area);
}

fn render_placeholder(frame: &mut Frame, area: Rect) {
    let content = Paragraph::new("Screen not implemented yet")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(content, area);
}

fn render_language_screen(frame: &mut Frame, app: &App, area: Rect) {
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
        .block(Block::default().borders(Borders::ALL).title("🌍 Language").border_style(Style::default().fg(Color::Cyan)));
    frame.render_widget(lang_list, area);
}

fn render_quit_screen(frame: &mut Frame, area: Rect) {
    let content = Paragraph::new("Press Enter to quit wzllama")
        .block(Block::default().borders(Borders::ALL).title("🚪 Quit").border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(content, area);
}