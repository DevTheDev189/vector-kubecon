pub mod auth;
pub mod config;
pub mod error;
pub mod models;
pub mod routes;
pub mod services;
pub mod state;
pub mod telemetry;

pub use config::AppConfig;
pub use error::ApiError;
pub use state::AppState;

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::Utc;
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use crate::routes::*;

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
        // Health
        .route("/api/health", get(health_handler))
        // Auth
        .route("/api/auth/github", get(github_login_handler))
        .route("/api/auth/github/callback", get(github_callback_handler))
        .route("/api/auth/impersonate", post(impersonate_handler))
        .route("/api/auth/logout", post(logout_handler))
        // Profile & Sensory
        .route("/api/me", get(get_me_handler))
        .route("/api/sync-github", post(sync_github_handler))
        .route("/api/sensory", patch(toggle_sensory_handler))
        // Schedule & Search
        .route("/api/state", get(get_state_handler))
        .route("/api/search/talks", post(search_talks_handler))
        .route("/api/schedule/bookmark", post(toggle_bookmark_handler))
        // Handshake & Networking
        .route("/api/handshake/badge", get(get_badge_token_handler))
        .route("/api/handshake/connect", post(connect_handshake_handler))
        // Live room telemetry and crowd rerouting
        .route("/api/rooms", get(get_rooms_handler))
        .route("/api/rooms/audit-unbatched", post(audit_rooms_handler))
        .route("/api/rooms/checkin", post(checkin_handler))
        .route("/api/surge", post(surge_handler))
        .route("/api/surge/reset", post(reset_surge_handler))
        // Server-Sent Events (SSE)
        .route("/api/events/stream", get(sse_events_stream_handler))
        .layer(cors)
        .with_state(state)
}
