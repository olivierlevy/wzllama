use std::time::Duration;
use tokio::runtime::Runtime;

#[test]
fn taskmanager_start_stop() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mgr = wzllama::core::task_manager::TaskManager::new();
        // spawn a simple task that sleeps for a while
        mgr.spawn_named("t1", || async move { tokio::time::sleep(Duration::from_millis(500)).await }, wzllama::core::task_manager::RestartPolicy::Never).await.unwrap();
        let status = mgr.status("t1").await.unwrap();
        assert!(status.running);
        mgr.stop("t1").await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        let status = mgr.status("t1").await.unwrap();
        assert!(!status.running);
    });
}
