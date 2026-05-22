//! Scientific menu adapter for menu_api
//!
//! Bridges menu_api with the scientific wizard function.

use anyhow::Result;
use crate::config::I18n;
use crate::core::HardwareInfo;
use crate::config::WzllamaState;

/// Scientific menu runner
pub struct ScientificMenuRunner<'a> {
    i18n: &'a I18n,
    state: &'a mut WzllamaState,
    hw: &'a HardwareInfo,
}

impl<'a> ScientificMenuRunner<'a> {
    pub fn new(i18n: &'a I18n, state: &'a mut WzllamaState, hw: &'a HardwareInfo) -> Self {
        Self { i18n, state, hw }
    }
    
    /// Run the scientific menu
    pub fn run(&mut self) -> Result<()> {
        crate::wizard::menu_scientific::run(self.i18n, self.state, self.hw)
    }
}