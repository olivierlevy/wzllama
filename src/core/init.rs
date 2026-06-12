use crate::config;
use crate::core::task_manager::TaskManager;
use std::sync::OnceLock;

pub static GLOBAL_TASK_MANAGER: OnceLock<TaskManager> = OnceLock::new();

/// Initialize core runtime helpers for tests: create TaskManager and init i18n
pub async fn main_init_for_tests() {
    // Ensure paths and logging/i18n initialization similar to main
    let _ = config::paths::ensure_dirs();
    config::i18n::init_global();
    // Create and register TaskManager
    let mgr = TaskManager::new();
    let _ = GLOBAL_TASK_MANAGER.set(mgr);
}

pub fn get_global_task_manager() -> Option<&'static TaskManager> {
    GLOBAL_TASK_MANAGER.get()
}
