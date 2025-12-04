//! Metrics API endpoint for Prometheus scraping

use axum::{extract::State, response::IntoResponse, routing::get, Router};
use std::sync::Arc;

use crate::services::message::MessageMetrics;

/// Application state for metrics endpoint
#[derive(Clone)]
pub struct MetricsState {
    pub metrics: Arc<MessageMetrics>,
}

/// Prometheus metrics endpoint
pub async fn prometheus_metrics(State(state): State<MetricsState>) -> impl IntoResponse {
    let snapshot = state.metrics.snapshot().await;
    (
        [("content-type", "text/plain; charset=utf-8")],
        snapshot.to_prometheus(),
    )
}

/// JSON metrics endpoint
pub async fn json_metrics(State(state): State<MetricsState>) -> impl IntoResponse {
    let snapshot = state.metrics.snapshot().await;
    axum::Json(snapshot)
}

/// Create metrics router
pub fn create_metrics_router(metrics: Arc<MessageMetrics>) -> Router {
    let state = MetricsState { metrics };
    
    Router::new()
        .route("/metrics", get(prometheus_metrics))
        .route("/metrics/json", get(json_metrics))
        .with_state(state)
}
