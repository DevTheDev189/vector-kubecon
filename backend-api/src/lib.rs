pub mod config;
pub mod error;
pub mod state;
pub mod telemetry;

pub use config::AppConfig;
pub use error::ApiError;
pub use state::AppState;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use chrono::Utc;
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

#[tracing::instrument(skip_all, fields(http.route = "/api/health"))]
pub async fn health_handler(State(_state): State<AppState>) -> impl IntoResponse {
    let trace_id = telemetry::current_trace_id();
    let timestamp_utc = Utc::now().to_rfc3339();

    Json(json!({
        "status": "ok",
        "service": "vector-api",
        "trace_id": trace_id,
        "timestamp_utc": timestamp_utc
    }))
}

pub fn app_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_handler))
        .layer(cors)
        .with_state(state)
}
