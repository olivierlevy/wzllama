//! Config menu adapter for menu_api
//!
//! Bridges menu_api with the config wizard function.

use anyhow::Result;
use crate::config::I18n;
use crate::core::HardwareInfo;
use crate::config::WzllamaState;

/// Config menu runner
pub struct ConfigMenuRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> ConfigMenuRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }
    
    /// Run the config menu
    pub fn run(&mut self) -> Result<()> {
        crate::wizard::menu_config::run(self.i18n, self.state, self.hw)
    }
}