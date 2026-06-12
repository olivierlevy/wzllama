use axum::response::IntoResponse;
use prometheus::{Registry, TextEncoder, Encoder, opts, Gauge, IntCounterVec};

/// Create and return a new Prometheus registry with basic application metrics registered.
pub fn register_registry() -> Registry {
    let registry = Registry::new();

    // Add example metrics (extend as needed)
    let _uptime = Gauge::with_opts(opts!("wzllama_uptime_seconds", "Process uptime seconds")).ok();
    if let Some(u) = _uptime { let _ = registry.register(Box::new(u)); }

    let _task_restarts = IntCounterVec::new(opts!("wzllama_task_restarts_total", "Task restart count"), &["task"]).ok();
    if let Some(tr) = _task_restarts { let _ = registry.register(Box::new(tr)); }

    registry
}

/// Axum handler to export metrics from the provided registry.
pub async fn metrics_handler(registry: Registry) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];
    if encoder.encode(&metric_families, &mut buffer).is_ok() {
        (axum::http::StatusCode::OK, String::from_utf8_lossy(&buffer).to_string())
    } else {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, String::from("failed to encode metrics"))
    }
}
