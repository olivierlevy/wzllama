use wzllama::core;
use std::time::Duration;

#[tokio::test]
async fn updater_marks_timestamp() {
    core::init::main_init_for_tests().await;
    let mgr = core::init::get_global_task_manager().expect("task manager set").clone();

    // Spawn a named task that marks an update timestamp once (via spawn_named)
    let _ = mgr.spawn_named(
        "test-updater",
        || async move {
            let _ = tokio::task::spawn_blocking(move || {
                // Directly mark updated to simulate a successful background update
                wzllama::core::tool_updater::ToolUpdater::mark_updated();
            })
            .await;
        },
        wzllama::core::task_manager::RestartPolicy::Never,
    ).await.expect("spawned");

    tokio::time::sleep(Duration::from_secs(1)).await;

    let ts = dirs::home_dir().unwrap().join(".wzllama").join("last_update.txt");
    assert!(ts.exists(), "timestamp should exist");
}
