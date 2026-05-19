pub mod config;
pub mod core;
// TUI mode disabled - keeping module for future reference
// pub mod tui;
pub mod wizard;
pub mod tools;
pub mod cli;
pub mod display;
pub mod error;

// TUI re-exports commented out - CLI wizard is the primary interface
// pub use tui::App;
// pub use tui::Screen;
// pub use tui::Navigation;