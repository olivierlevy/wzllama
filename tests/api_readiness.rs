use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::test]
async fn readiness_returns_ok_when_task_manager_present() {
    // initialize global test helpers (creates GLOBAL_TASK_MANAGER)
    crate::core::init::main_init_for_tests().await;

    // Start server on ephemeral port
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let bound = crate::api_server::start_test_server(addr).await;

    // Blocking client is fine for this integration test
    let url = format!("http://{}/ready", bound);
    let resp = reqwest::blocking::get(&url).expect("request failed");
    assert!(resp.status().is_success(), "expected 200 OK, got {}", resp.status());
}
