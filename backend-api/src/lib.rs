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
    routing::{any, get, patch, post},
    Json, Router,
};
use chrono::Utc;
use serde_json::json;

use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

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

#[tracing::instrument(skip_all, fields(http.route = "/api/*path"))]
async fn api_not_found() -> ApiError {
    ApiError::NotFound("API route not found".into())
}

pub fn app_router(state: AppState) -> Router {
    let static_dir = std::path::PathBuf::from(&state.config().static_dir);
    let admin_routes = Router::new()
        .route("/api/rooms/audit-unbatched", post(audit_rooms_handler))
        .route("/api/rooms/checkin", post(checkin_handler))
        .route("/api/surge", post(surge_handler))
        .route("/api/surge/reset", post(reset_surge_handler))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::require_admin,
        ));
    let protected = Router::new()
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
        .merge(admin_routes)
        .route("/api/auth/logout", post(logout_handler))
        // Server-Sent Events (SSE)
        .route("/api/events/stream", get(sse_events_stream_handler))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::require_session,
        ));

    Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/auth/status", get(auth_status_handler))
        .route("/api/auth/github", get(github_login_handler))
        .route("/api/auth/github/callback", get(github_callback_handler))
        .merge(protected)
        // Unknown API routes must remain JSON errors rather than SPA HTML.
        .route("/api", any(api_not_found))
        .route("/api/*path", any(api_not_found))
        // Missing hashed assets must return 404, never index.html.
        .nest_service("/assets", ServeDir::new(static_dir.join("assets")))
        .fallback_service(
            ServeDir::new(&static_dir).fallback(ServeFile::new(static_dir.join("index.html"))),
        )
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
