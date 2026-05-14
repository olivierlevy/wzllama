use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::{Duration, Instant};

/// Events from the terminal
#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    /// User pressed a key
    Key(KeyEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Tick for periodic updates
    Tick,
}

/// Event handler with timeout
pub struct EventHandler {
    /// Minimum time between tick events
    tick_rate: Duration,
    /// Track when last tick happened
    last_tick: Instant,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate),
            last_tick: Instant::now(),
        }
    }

    /// Wait for next event, handling tick timeout
    pub fn next(&mut self) -> AppEvent {
        let timeout = self.tick_rate.saturating_sub(self.last_tick.elapsed());
        
        if event::poll(timeout).ok().unwrap_or(false) {
            match event::read().ok() {
                Some(Event::Key(key)) => return AppEvent::Key(key),
                Some(Event::Resize(w, h)) => return AppEvent::Resize(w, h),
                _ => {}
            }
        }
        
        // Check for tick
        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
            return AppEvent::Tick;
        }
        
        // No event, wait more
        AppEvent::Tick
    }
}

/// Convert crossterm key to our navigation
pub fn key_to_nav(key: KeyEvent) -> Option<super::app::Navigation> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(super::app::Navigation::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(super::app::Navigation::Down),
        KeyCode::Left | KeyCode::Char('h') => Some(super::app::Navigation::Left),
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => Some(super::app::Navigation::Right),
        KeyCode::Tab => Some(super::app::Navigation::Search),
        // Esc is handled separately for quit behavior on Main
        KeyCode::Esc => Some(super::app::Navigation::Quit),
        _ => None,
    }
}