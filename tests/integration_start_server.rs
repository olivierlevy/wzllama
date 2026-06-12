use wzllama::core;

#[tokio::test]
async fn main_starts_taskmanager() {
    core::init::main_init_for_tests().await;
    assert!(core::init::get_global_task_manager().is_some());
}
