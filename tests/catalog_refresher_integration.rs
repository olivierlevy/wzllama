use wzllama::core;
use std::time::Duration;

#[tokio::test]
async fn refresher_writes_cache_via_task_manager() {
    // Initialize test helpers (creates dirs, i18n, TaskManager)
    core::init::main_init_for_tests().await;
    let mgr = core::init::get_global_task_manager().expect("task manager set").clone();

    // spawn a short periodic blocking task that writes the cache
    let key = "ollama_catalog";
    // Ensure cache cleared
    let _ = crate::core::cache::clear_cache();

    mgr.spawn_periodic_blocking(
        "test-catalog",
        Duration::from_secs(1),
        || {
            let _ = crate::core::cache::write_cache("ollama_catalog", "{\"tools\": []}");
        },
    )
    .await
    .expect("spawn periodic blocking");

    // Wait a moment for the periodic job to run
    tokio::time::sleep(Duration::from_secs(2)).await;

    let cache = crate::core::cache::read_cache(key, false).unwrap();
    assert!(cache.is_some(), "cache should be created by periodic task");
}
