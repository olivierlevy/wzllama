//! Wizard action wrapper using Arc for clone support
//!
//! This allows menu actions to be cloned and shared across the menu system.

use anyhow::Result;
use std::sync::{Arc, Mutex};
use crate::config::{I18n, WzllamaState};
use crate::core::HardwareInfo;
use crate::menu_api::{ActionContext, ActionResult, ToolAction};

/// Cloneable wrapper for wizard actions that holds Arc references
pub struct ArcAction {
    id: String,
    name: String,
    i18n: Arc<I18n>,
    state: Arc<Mutex<WzllamaState>>,
    hw: Arc<HardwareInfo>,
    action_fn: Arc<dyn Fn() -> Result<()> + Send + Sync>,
}

impl Clone for ArcAction {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            i18n: Arc::clone(&self.i18n),
            state: Arc::clone(&self.state),
            hw: Arc::clone(&self.hw),
            action_fn: Arc::clone(&self.action_fn),
        }
    }
}

impl ArcAction {
    /// Create a new ArcAction with captured context
    pub fn new(
        id: &str,
        name: &str,
        i18n: Arc<I18n>,
        state: Arc<Mutex<WzllamaState>>,
        hw: Arc<HardwareInfo>,
        action_fn: Arc<dyn Fn() -> Result<()> + Send + Sync>,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            i18n,
            state,
            hw,
            action_fn,
        }
    }
}

impl ToolAction for ArcAction {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, _ctx: &ActionContext) -> Result<ActionResult> {
        (self.action_fn)()
            .map(|_| ActionResult::success())
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

/// Runner that holds the context for creating ArcAction instances
pub struct ArcActionRunner {
    i18n: Arc<I18n>,
    state: Arc<Mutex<WzllamaState>>,
    hw: Arc<HardwareInfo>,
}

impl ArcActionRunner {
    pub fn new(i18n: I18n, state: WzllamaState, hw: HardwareInfo) -> Self {
        Self {
            i18n: Arc::new(i18n),
            state: Arc::new(Mutex::new(state)),
            hw: Arc::new(hw),
        }
    }

    /// Create an action that executes a wizard function with the captured context
    pub fn create_action<F>(&self, id: &str, name: &str, func: F) -> ArcAction
    where
        F: Fn(&I18n, &mut WzllamaState, &HardwareInfo) -> Result<()> + Send + Sync + 'static,
    {
        // Clone for the action_fn closure
        let i18n_for_fn = Arc::clone(&self.i18n);
        let state_for_fn = Arc::clone(&self.state);
        let hw_for_fn = Arc::clone(&self.hw);
        
        let action_fn = Arc::new(move || {
            let mut state_guard = state_for_fn.lock().unwrap();
            func(&i18n_for_fn, &mut state_guard, &hw_for_fn)
        });
        
        // Clone again for ArcAction (these will be cloned again inside ArcAction)
        ArcAction::new(
            id,
            name,
            Arc::clone(&self.i18n),
            Arc::clone(&self.state),
            Arc::clone(&self.hw),
            action_fn,
        )
    }
}