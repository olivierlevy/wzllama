//! Helper modules - shared logic between TUI and normal modes

pub mod menu_cleanup;
pub mod menu_config;
pub mod menu_fleets;
pub mod menu_main;
pub mod menu_models;
pub mod menu_tools;
pub mod menu_usage;
pub mod cleanup_fleets;
pub mod cleanup_models;
pub mod cleanup_tools;
pub mod configurator;
pub mod estimator;
pub mod fleet_creator;
pub mod fleet_templates;
pub mod setup_models;

// Re-export for convenience
pub use menu_main::{run, select_language};