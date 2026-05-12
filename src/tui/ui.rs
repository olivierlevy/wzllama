use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::Terminal;
use crate::tui::App;
use crate::tui::event::{EventHandler, key_to_nav, AppEvent};
use crate::tui::screens::Screen;
use crate::core::system;
use crate::core::ollama_api;
use crate::config::WzllamaState;

pub fn run_tui(state: WzllamaState, hw: crate::core::hardware::HardwareInfo) -> anyhow::Result<()> {
    let mut app = App::new(state, hw);
    let mut event_handler = EventHandler::new(100);
    
    let mut terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;
    
    crossterm::terminal::enable_raw_mode()?;
    terminal.hide_cursor()?;
    
    loop {
        terminal.draw(|frame| {
            render_app(frame, &mut app);
        })?;
        
        match event_handler.next() {
            AppEvent::Key(key) => {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    if let Some(nav) = key_to_nav(key) {
                        app.navigate(nav);
                    }
                }
            }
            AppEvent::Quit => break,
            AppEvent::Tick => app.tick(),
            _ => {}
        }
        
        if app.should_quit {
            break;
        }
    }
    
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
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
    ).block(title_block).centered();
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
        Screen::Models => render_models_screen(frame, app, area),
        Screen::Tools => render_tools_screen(frame, app, area),
        Screen::Config => render_config_screen(frame, app, area),
        Screen::Cleanup => render_cleanup_screen(frame, app, area),
        _ => render_placeholder(frame, area),
    }
}

fn render_main_screen(frame: &mut Frame, _app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);
    
    // Header
    let header = Paragraph::new(vec![
        Line::from(Span::styled("wzllama - Assistant IA Local", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("🎯 Navigate with ↑↓←→ or hjkl"),
        Line::from("⚡ Select with Enter or l"),
        Line::from("🚪 Quit with q or Esc"),
    ])
    .block(Block::default().borders(Borders::ALL).title("🏠 Main").border_style(Style::default().fg(Color::Cyan)));
    frame.render_widget(header, chunks[0]);
    
    // Model suggestions
    let models = vec![
        ("🤖 qwen2.5-coder:14b", "Code"),
        ("📚 qwen2.5:14b", "Books"),
        ("💬 qwen2.5:7b", "Chat"),
        ("🧠 qwen2.5:3b", "Agent"),
    ];
    
    let model_items: Vec<ListItem> = models.iter().map(|(name, desc)| {
        ListItem::new(Line::from(vec![
            Span::styled(*name, Style::default().fg(Color::Magenta)),
            Span::styled(format!(" → {}", desc), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();
    
    let model_list = List::new(model_items)
        .block(Block::default().borders(Borders::ALL).title("📦 Suggested Models").border_style(Style::default().fg(Color::Magenta)));
    frame.render_widget(model_list, chunks[1]);
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
    
    let model_list = List::new(model_items)
        .block(Block::default().borders(Borders::ALL).title("🤖 Models").border_style(Style::default().fg(Color::Magenta)));
    frame.render_widget(model_list, area);
}

fn render_tools_screen(frame: &mut Frame, _app: &App, area: Rect) {
    let tools = vec![
        ("claude-code", "Claude Code", "AI coding assistant"),
        ("opencode", "OpenCode", "Open source AI agent"),
        ("open-webui", "Open WebUI", "Web interface for Ollama"),
        ("openclaw", "OpenClaw", "Terminal-based agent"),
    ];
    
    let tool_items: Vec<ListItem> = tools.iter().map(|(id, name, desc)| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", id), Style::default().fg(Color::Yellow)),
            Span::styled(*name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" - {}", desc), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();
    
    let tool_list = List::new(tool_items)
        .block(Block::default().borders(Borders::ALL).title("🛠️ Tools").border_style(Style::default().fg(Color::Yellow)));
    frame.render_widget(tool_list, area);
}

fn render_config_screen(frame: &mut Frame, _app: &App, area: Rect) {
    let config_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("🔤 Language", Style::default().fg(Color::Cyan)),
            Span::styled(" → fr", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("📁 Templates", Style::default().fg(Color::Cyan)),
            Span::styled(" → configured", Style::default().fg(Color::Green)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("⚡ Default Model", Style::default().fg(Color::Cyan)),
            Span::styled(" → qwen2.5:7b", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🔄 Auto-pull Models", Style::default().fg(Color::Cyan)),
            Span::styled(" → on", Style::default().fg(Color::Green)),
        ])),
    ];
    
    let config_list = List::new(config_items)
        .block(Block::default().borders(Borders::ALL).title("⚙️ Config").border_style(Style::default().fg(Color::Green)));
    frame.render_widget(config_list, area);
}

fn render_cleanup_screen(frame: &mut Frame, _app: &App, area: Rect) {
    let cleanup_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("🗑️ Unused Models", Style::default().fg(Color::Red)),
            Span::styled(" → 0", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🧹 Temp Files", Style::default().fg(Color::Yellow)),
            Span::styled(" → 0 MB", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🔄 Reset Config", Style::default().fg(Color::Cyan)),
            Span::styled(" → reset to defaults", Style::default().fg(Color::White)),
        ])),
    ];
    
    let cleanup_list = List::new(cleanup_items)
        .block(Block::default().borders(Borders::ALL).title("🧹 Cleanup").border_style(Style::default().fg(Color::Red)));
    frame.render_widget(cleanup_list, area);
}

fn render_placeholder(frame: &mut Frame, area: Rect) {
    let content = Paragraph::new("Screen not implemented yet")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(content, area);
}