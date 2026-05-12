use ratatui::widgets::{Widget, Block, Borders, Paragraph, List, ListItem};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Layout, Constraint, Direction};
use ratatui::style::{Style, Color, Modifier};
use ratatui::text::{Span, Line, Text};

/// Barre de ressources avec affichage graphique moderne
pub struct ResourceBar {
    pub label: String,
    pub used: f64,
    pub total: f64,
    pub bar_width: usize,
    pub color: Color,
}

impl ResourceBar {
    pub fn new(label: &str, used: f64, total: f64) -> Self {
        Self {
            label: label.to_string(),
            used,
            total,
            bar_width: 20,
            color: Color::Cyan,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Widget for ResourceBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let ratio = if self.total > 0.0 { 
            (self.used / self.total).min(1.0) 
        } else { 
            0.0 
        };
        let filled = ((ratio * self.bar_width as f64) as usize).min(self.bar_width);
        
        let bar_filled = "█".repeat(filled);
        let bar_empty = "░".repeat(self.bar_width - filled);
        let percent = format!("{:.0}%", ratio * 100.0);
        
        let line = Line::from(vec![
            Span::styled(format!("{} ", self.label), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(bar_filled, Style::default().fg(self.color)),
            Span::styled(bar_empty, Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" {}", percent), Style::default().fg(self.color).add_modifier(Modifier::BOLD)),
        ]);
        
        buf.set_line(area.x, area.y, &line, area.width);
    }
}

/// Indicateur de statut moderne avec icônes
pub struct StatusIndicator {
    pub status: Status,
    pub text: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    Warning,
    Error,
    Info,
    None,
    Running,
    Stopped,
}

impl Status {
    pub fn symbol(&self) -> &'static str {
        match self {
            Status::Ok => "✓",
            Status::Warning => "⚠",
            Status::Error => "✗",
            Status::Info => "ⓘ",
            Status::None => " ",
            Status::Running => "●",
            Status::Stopped => "○",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Status::Ok => Color::Green,
            Status::Warning => Color::Yellow,
            Status::Error => Color::Red,
            Status::Info => Color::Cyan,
            Status::None => Color::White,
            Status::Running => Color::Green,
            Status::Stopped => Color::DarkGray,
        }
    }
}

impl Widget for StatusIndicator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let symbol = self.status.symbol();
        let color = self.status.color();
        
        let mut spans = vec![
            Span::styled(symbol, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {}", self.text), Style::default().fg(Color::White)),
        ];
        
        if let Some(detail) = &self.detail {
            spans.push(Span::styled(format!(" [{}]", detail), Style::default().fg(Color::DarkGray)));
        }
        
        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}

/// Carte de modèle moderne avec design épuré
pub struct ModelCard {
    pub name: String,
    pub size: String,
    pub family: String,
    pub installed: bool,
    pub running: bool,
    pub selected: bool,
    pub tags: Vec<&'static str>,
}

impl ModelCard {
    pub fn new(name: &str, size: &str, family: &str) -> Self {
        Self {
            name: name.to_string(),
            size: size.to_string(),
            family: family.to_string(),
            installed: false,
            running: false,
            selected: false,
            tags: Vec::new(),
        }
    }
}

impl Widget for ModelCard {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let icon = if self.running {
            Span::styled("● ", Style::default().fg(Color::Green))
        } else if self.installed {
            Span::styled("○ ", Style::default().fg(Color::Cyan))
        } else {
            Span::styled("○ ", Style::default().fg(Color::DarkGray))
        };
        
        let bg = if self.selected { 
            Color::Rgb(50, 50, 50) 
        } else { 
            Color::Reset 
        };
        
        let style = Style::default().bg(bg);
        
        let mut spans = vec![icon];
        spans.push(Span::styled(format!(" {} ", self.name), style.fg(Color::White).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!("[{}] ", self.size), style.fg(Color::DarkGray)));
        
        for tag in self.tags {
            spans.push(Span::styled(format!("{} ", tag), style.fg(Color::Cyan)));
        }
        
        let line = Line::from(spans);
        
        // Dessiner un fond coloré pour la sélection
        for y in area.y..area.y + 1 {
            for x in area.x..area.x + area.width {
                if x < buf.area.width && y < buf.area.height {
                    buf.get_mut(x, y).set_bg(bg);
                }
            }
        }
        
        buf.set_line(area.x, area.y, &line, area.width);
    }
}

/// En-tête moderne avec gradient de couleurs
pub struct Header {
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: &'static str,
}

impl Header {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            subtitle: None,
            icon: "",
        }
    }
    
    pub fn with_subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }
    
    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.icon = icon;
        self
    }
}

impl Widget for Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = vec![Line::from(vec![
            Span::styled(self.icon, Style::default().fg(Color::Magenta)),
            Span::styled(format!(" {} ", self.title), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])];
        
        if let Some(sub) = &self.subtitle {
            lines.push(Line::from(Span::styled(sub, Style::default().fg(Color::DarkGray))));
        }
        
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Cyan));
        
        Paragraph::new(Text::from(lines)).block(block).render(area, buf);
    }
}

/// Carte d'information moderne
pub struct InfoCard {
    pub title: String,
    pub content: String,
    pub status: Status,
    pub accent_color: Color,
}

impl InfoCard {
    pub fn new(title: &str, content: &str) -> Self {
        Self {
            title: title.to_string(),
            content: content.to_string(),
            status: Status::Info,
            accent_color: Color::Cyan,
        }
    }
}

impl Widget for InfoCard {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_area = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        
        let title_line = Line::from(Span::styled(
            format!(" {} ", self.title),
            Style::default().fg(self.accent_color).add_modifier(Modifier::BOLD),
        ));
        
        let content_line = Line::from(Span::styled(
            format!(" {}", self.content),
            Style::default().fg(Color::White),
        ));
        
        let text = Text::from(vec![title_line, content_line]);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.accent_color));
        
        Paragraph::new(text).block(block).render(area, buf);
    }
}

/// Barre de progression moderne
pub struct ProgressBar {
    pub label: String,
    pub percent: f64,
    pub show_percent: bool,
    pub color: Color,
}

impl ProgressBar {
    pub fn new(label: &str, percent: f64) -> Self {
        Self {
            label: label.to_string(),
            percent,
            show_percent: true,
            color: Color::Cyan,
        }
    }
}

impl Widget for ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let ratio = self.percent.clamp(0.0, 100.0);
        let bar_width = (area.width as f64 * 0.7) as usize;
        let filled = ((ratio / 100.0) * bar_width as f64) as usize;
        
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_width - filled));
        
        let mut spans = vec![Span::styled(format!("{} ", self.label), Style::default().fg(Color::White))];
        spans.push(Span::styled(bar, Style::default().fg(self.color)));
        
        if self.show_percent {
            spans.push(Span::styled(format!(" {:.0}%", ratio), Style::default().fg(self.color)));
        }
        
        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}

/// Ligne de commande interactive
pub struct CommandRow {
    pub prompt: String,
    pub command: String,
    pub cursor: bool,
}

impl CommandRow {
    pub fn new(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            command: String::new(),
            cursor: false,
        }
    }
}

impl Widget for CommandRow {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let cursor_char = if self.cursor { "█" } else { " " };
        let line = Line::from(vec![
            Span::styled(format!("{} ", self.prompt), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&self.command, Style::default().fg(Color::White)),
            Span::styled(cursor_char, Style::default().fg(Color::Green)),
        ]);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}