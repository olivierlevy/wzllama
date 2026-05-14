use std::sync::{Arc, Mutex};

/// Terminal state for integrated terminal (simplified version)
/// Note: True interactive PTY requires complex async handling.
/// For now, this provides a display area for command output.
pub struct TerminalState {
    pub output: Arc<Mutex<String>>,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            output: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::new()
    }
}