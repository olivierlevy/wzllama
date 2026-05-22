//! ToolAction trait - Bridge between menu actions and tool execution
//!
//! This trait defines how menu items trigger tool actions with dynamic parameters.

use anyhow::Result;
use std::collections::HashMap;

/// Type alias for action closures
type ActionFn = Box<dyn Fn(&ActionContext) -> Result<ActionResult> + Send + Sync>;

/// Context provided to tool actions execution
#[derive(Debug, Clone, Default)]
pub struct ActionContext {
    /// Parameters passed to the action
    pub params: HashMap<String, String>,
    /// Current state from wzllama
    pub state: Option<serde_json::Value>,
}

impl ActionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
}

/// Action result with status and optional message
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success: bool,
    pub message: Option<String>,
}

impl ActionResult {
    pub fn success() -> Self {
        Self { success: true, message: None }
    }

    pub fn success_with(message: &str) -> Self {
        Self { success: true, message: Some(message.to_string()) }
    }

    pub fn failure(message: &str) -> Self {
        Self { success: false, message: Some(message.to_string()) }
    }
}

/// Trait for executing menu actions
///
/// This trait bridges the gap between menu selection and tool execution.
/// Each menu item can have an associated Action that handles:
/// - Installation commands
/// - Launch commands  
/// - Configuration actions
/// - Navigation actions
pub trait ToolAction: Send + Sync {
    /// Returns the unique identifier for this action
    fn id(&self) -> &str;

    /// Returns the display name for this action
    fn name(&self) -> &str;

    /// Execute the action with the given context
    fn execute(&self, ctx: &ActionContext) -> Result<ActionResult>;

    /// Validate input parameters before execution
    fn validate(&self, _ctx: &ActionContext) -> Result<()> {
        Ok(())
    }

    /// Returns true if this action requires confirmation
    fn requires_confirmation(&self) -> bool {
        false
    }

    /// Returns confirmation message if needed
    fn confirmation_message(&self) -> Option<&str> {
        None
    }
}

/// Wrapper for closures as ToolAction
pub struct ClosureAction {
    id: String,
    name: String,
    action: ActionFn,
}

impl ClosureAction {
    pub fn new<F>(id: &str, name: &str, action: F) -> Self
    where
        F: Fn(&ActionContext) -> Result<ActionResult> + Send + Sync + 'static,
    {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            action: Box::new(action),
        }
    }
}

impl ToolAction for ClosureAction {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, ctx: &ActionContext) -> Result<ActionResult> {
        (self.action)(ctx)
    }
}

/// Action dispatcher for managing available actions
pub struct ActionDispatcher {
    actions: HashMap<String, Box<dyn ToolAction>>,
}

impl Default for ActionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionDispatcher {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn register(&mut self, action: Box<dyn ToolAction>) {
        self.actions.insert(action.id().to_string(), action);
    }

    pub fn get(&self, id: &str) -> Option<&dyn ToolAction> {
        self.actions.get(id).map(|a| a.as_ref())
    }

    pub fn execute(&self, id: &str, ctx: &ActionContext) -> Result<ActionResult> {
        let action = self.actions.get(id)
            .ok_or_else(|| anyhow::anyhow!("Action '{}' not found", id))?;
        
        action.validate(ctx)?;
        action.execute(ctx)
    }

    pub fn list_ids(&self) -> Vec<&str> {
        self.actions.keys().map(|s| s.as_str()).collect()
    }
}