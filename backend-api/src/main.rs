use backend_api::{
    app_router, config::AppConfig, models::RoomState, state::AppState, telemetry::init_telemetry,
};
use opentelemetry::KeyValue;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = AppConfig::from_env();
    init_telemetry(&config)?;

    let state = AppState::new(config.clone());
    state.elastic().ensure_indices_and_seed().await?;
    let rooms: serde_json::Value = state
        .http()
        .get(format!(
            "{}/rooms_state/_search?size=100",
            state.elastic().base_url
        ))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    if let Some(hits) = rooms["hits"]["hits"].as_array() {
        for hit in hits {
            let room: RoomState = serde_json::from_value(hit["_source"].clone())?;
            let attributes = [KeyValue::new("room.id", room.room_id)];
            state
                .metrics()
                .room_occupancy_ratio
                .record(room.occupancy_ratio, &attributes);
            state
                .metrics()
                .room_decibels
                .record(room.decibel_level, &attributes);
        }
    }
    let app = app_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!(port = config.port, "VECTOR API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
