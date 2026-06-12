use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::test]
async fn readiness_returns_ok_when_task_manager_present() {
    // initialize global test helpers (creates GLOBAL_TASK_MANAGER)
    // initialize global test helpers (creates GLOBAL_TASK_MANAGER)
    wzllama::core::init::main_init_for_tests().await;

    // Start server on ephemeral port
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let bound = wzllama::api_server::start_test_server(addr).await;

    // Async client to avoid blocking the test runtime
    let url = format!("http://{}/ready", bound);
    let resp = reqwest::get(&url).await.expect("request failed");
    assert!(resp.status().is_success(), "expected 200 OK, got {}", resp.status());
}
