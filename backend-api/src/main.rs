use backend_api::{app_router, config::AppConfig, state::AppState, telemetry::init_telemetry};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = AppConfig::from_env();
    let _ = init_telemetry(&config);

    let state = AppState::new(config.clone());
    let app = app_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!(port = config.port, "VECTOR API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
