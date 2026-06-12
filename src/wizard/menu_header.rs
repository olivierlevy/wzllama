//! Unifié header module for all wizard menus
//! Replaces duplicated header/header_with_resources patterns

use crate::config::I18n;
use crate::core::{ollama_api, system};
use crate::display;

/// Render a menu title header with optional system resources
///
/// # Arguments
/// * `i18n` - Internationalization handle
/// * `title_key` - Translation key for the title
/// * `show_resources` - Whether to show RAM/VRAM bars
/// * `last_model` - Optional default model to display
/// * `hw_ram_gb` - Total system RAM in GB
/// * `hw_vram_gb` - Total VRAM in GB
pub fn render(
    i18n: &I18n,
    title_key: &str,
    show_resources: bool,
    last_model: Option<&str>,
    hw_ram_gb: f64,
    hw_vram_gb: f64,
) {
    if show_resources {
        let ram_avail = system::get_available_ram_gb();
        let vram_avail = system::get_available_vram_gb();
        let running = ollama_api::get_running_models();
        display::clear_screen();
        display::header_with_resources(
            &i18n.t(title_key),
            hw_ram_gb,
            ram_avail,
            hw_vram_gb,
            vram_avail,
            &running,
            last_model,
        );
    } else {
        display::header(&i18n.t(title_key));
    }
}
