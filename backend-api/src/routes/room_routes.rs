use axum::{extract::State, Json};
use chrono::Utc;
use opentelemetry::KeyValue;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    auth::OptionalAuthUser,
    models::{BuilderProfile, ConferenceTalk, RoomState, SseEvent},
    telemetry::current_trace_id,
    ApiError, AppState,
};

#[derive(Deserialize)]
pub struct CheckinRequest {
    pub room_id: String,
    pub delta: i32,
    pub decibel_reading: Option<f64>,
}

#[derive(Deserialize)]
pub struct SurgeRequest {
    #[serde(default = "default_room")]
    pub room_id: String,
}

fn default_room() -> String {
    "ballroom-b".into()
}

fn validate_room_id(id: &str) -> Result<(), ApiError> {
    if id.is_empty() || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
        return Err(ApiError::BadRequest("Invalid room ID".into()));
    }
    Ok(())
}

async fn elastic_json(request: reqwest::RequestBuilder) -> Result<Value, ApiError> {
    let response = request.send().await?;
    if response.status().as_u16() == 404 {
        return Err(ApiError::NotFound("Room or active talk not found".into()));
    }
    if !response.status().is_success() {
        tracing::error!(status = %response.status(), "Room Elasticsearch operation failed");
        return Err(ApiError::ElasticError(
            "Room telemetry operation failed".into(),
        ));
    }
    Ok(response.json().await?)
}

#[tracing::instrument(skip_all, fields(http.route = "/api/rooms"))]
pub async fn get_rooms_handler(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let body = elastic_json(
        state
            .http()
            .post(format!("{}/rooms_state/_search", state.elastic().base_url))
            .json(&json!({"size": 100, "sort": [{"room_id": "asc"}]})),
    )
    .await?;
    let rooms: Vec<RoomState> = body["hits"]["hits"]
        .as_array()
        .ok_or_else(|| ApiError::ElasticError("Missing room hits".into()))?
        .iter()
        .map(|h| serde_json::from_value(h["_source"].clone()))
        .collect::<Result<_, _>>()?;
    Ok(Json(
        json!({"rooms": rooms, "trace_id": current_trace_id(), "timestamp_utc": Utc::now()}),
    ))
}

// Elasticsearch applies the delta atomically; the local lock also orders room/talk
// propagation and SSE events for concurrent simulator requests on this instance.
async fn update_room(
    state: &AppState,
    id: &str,
    delta: i32,
    absolute: Option<i32>,
    noise: Option<f64>,
) -> Result<RoomState, ApiError> {
    validate_room_id(id)?;
    if noise.is_some_and(|db| !db.is_finite() || !(0.0..=140.0).contains(&db)) {
        return Err(ApiError::BadRequest(
            "Decibel reading must be between 0 and 140".into(),
        ));
    }
    let body = elastic_json(state.http().post(format!("{}/rooms_state/_update/{}?refresh=true&retry_on_conflict=10", state.elastic().base_url, id))
        .json(&json!({"_source": true, "script": {"lang": "painless", "source": "if (ctx._source.capacity_max <= 0) { throw new IllegalArgumentException('Invalid room capacity'); } long count = params.absolute != null ? params.absolute : (long)ctx._source.capacity_current + params.delta; ctx._source.capacity_current = (int)Math.max(0L, Math.min((long)ctx._source.capacity_max, count)); ctx._source.occupancy_ratio = (double)ctx._source.capacity_current / ctx._source.capacity_max; if (params.noise != null) { ctx._source.decibel_level = params.noise; } ctx._source.updated_at = params.now;", "params": {"absolute": absolute, "delta": delta, "noise": noise, "now": Utc::now()}}}))).await?;
    let room: RoomState = serde_json::from_value(body["get"]["_source"].clone())?;
    if let Some(talk_id) = &room.active_talk_id {
        elastic_json(state.http().post(format!("{}/conference_talks/_update/{}?refresh=true", state.elastic().base_url, talk_id))
            .json(&json!({"doc": {"capacity_current": room.capacity_current, "occupancy_ratio": room.occupancy_ratio, "decibel_level": room.decibel_level}}))).await?;
    }
    let attributes = [KeyValue::new("room.id", room.room_id.clone())];
    state
        .metrics()
        .room_occupancy_ratio
        .record(room.occupancy_ratio, &attributes);
    state
        .metrics()
        .room_decibels
        .record(room.decibel_level, &attributes);
    state.broadcast_sse(SseEvent::RoomTelemetryUpdate {
        room: room.clone(),
        trace_id: current_trace_id(),
        timestamp_utc: Utc::now(),
    });
    Ok(room)
}

#[tracing::instrument(skip_all, fields(http.route = "/api/rooms/checkin"))]
pub async fn checkin_handler(
    State(state): State<AppState>,
    Json(payload): Json<CheckinRequest>,
) -> Result<Json<Value>, ApiError> {
    let _guard = state.inner.room_updates.lock().await;
    let room = update_room(
        &state,
        &payload.room_id,
        payload.delta,
        None,
        payload.decibel_reading,
    )
    .await?;
    Ok(Json(
        json!({"room": room, "trace_id": current_trace_id(), "timestamp_utc": Utc::now()}),
    ))
}

#[tracing::instrument(skip_all, fields(http.route = "/api/surge"))]
pub async fn surge_handler(
    State(state): State<AppState>,
    user: OptionalAuthUser,
    Json(payload): Json<SurgeRequest>,
) -> Result<Json<Value>, ApiError> {
    let _guard = state.inner.room_updates.lock().await;
    validate_room_id(&payload.room_id)?;
    let body = elastic_json(state.http().get(format!(
        "{}/rooms_state/_doc/{}",
        state.elastic().base_url,
        payload.room_id
    )))
    .await?;
    let before: RoomState = serde_json::from_value(body["_source"].clone())?;
    let talk_id = before
        .active_talk_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Room has no active talk".into()))?;
    let body = elastic_json(state.http().get(format!(
        "{}/conference_talks/_doc/{}",
        state.elastic().base_url,
        talk_id
    )))
    .await?;
    let talk: ConferenceTalk = serde_json::from_value(body["_source"].clone())?;
    let room = update_room(
        &state,
        &payload.room_id,
        0,
        Some(before.capacity_max),
        Some(84.5),
    )
    .await?;
    tracing::warn!(room.id = %room.room_id, room.occupancy_ratio = 1.0, "CROWD_SURGE_THRESHOLD_EXCEEDED: triggering 3-way RRF auto-reroute");
    let username = user
        .0
        .map(|t| t.username)
        .unwrap_or_else(|| "zircon-dev".into());
    let body = elastic_json(state.http().get(format!(
        "{}/builders/_doc/{}",
        state.elastic().base_url,
        username
    )))
    .await?;
    let profile: BuilderProfile = serde_json::from_value(body["_source"].clone())?;
    let (alternatives, dsl) = state
        .elastic()
        .search_talks_hybrid_rrf(&profile, Some(talk.start_time), true)
        .await?;
    let recommended_alternative = alternatives
        .into_iter()
        .find(|t| t.talk.room_id != room.room_id)
        .map(|mut t| {
            t.is_rerouted_alternative = true;
            t
        });
    // Mark actual occupants as displaced so hallway matching uses live context.
    let result = elastic_json(state.http().post(format!("{}/builders/_update_by_query?refresh=true", state.elastic().base_url))
        .json(&json!({"query": {"term": {"current_room_id": room.room_id}}, "script": {"source": "ctx._source.bumped_from_talk_id = params.talk; ctx._source.updated_at = params.now", "params": {"talk": talk_id, "now": Utc::now()}}}))).await?;
    if result["failures"].as_array().is_some_and(|f| !f.is_empty()) {
        return Err(ApiError::ElasticError(
            "Failed to update displaced attendees".into(),
        ));
    }
    let mut peers = state
        .elastic()
        .match_hallway_builders(&profile, Some(&room.room_id), Some(talk_id))
        .await?;
    peers.truncate(2);
    state
        .metrics()
        .surge_reroutes_total
        .add(1, &[KeyValue::new("room.id", room.room_id.clone())]);
    let event = SseEvent::CrowdSurgeAlert {
        jammed_room_id: room.room_id.clone(),
        jammed_room_name: room.room_name.clone(),
        jammed_talk_id: talk_id.clone(),
        jammed_talk_title: talk.title,
        occupancy_current: room.capacity_current,
        occupancy_max: room.capacity_max,
        decibel_level: room.decibel_level,
        recommended_alternative,
        bumped_hallway_peers: peers,
        trace_id: current_trace_id(),
        timestamp_utc: Utc::now(),
    };
    state.broadcast_sse(event.clone());
    Ok(Json(
        json!({"room": room, "surge": event, "elastic_query_dsl": dsl, "trace_id": current_trace_id(), "timestamp_utc": Utc::now()}),
    ))
}

#[tracing::instrument(skip_all, fields(http.route = "/api/surge/reset"))]
pub async fn reset_surge_handler(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let _guard = state.inner.room_updates.lock().await;
    let room = update_room(&state, "ballroom-b", 0, Some(142), Some(68.0)).await?;
    if let Some(talk_id) = &room.active_talk_id {
        let result = elastic_json(state.http().post(format!("{}/builders/_update_by_query?refresh=true", state.elastic().base_url))
            .json(&json!({"query": {"term": {"bumped_from_talk_id": talk_id}}, "script": {"source": "ctx._source.bumped_from_talk_id = null; ctx._source.updated_at = params.now", "params": {"now": Utc::now()}}}))).await?;
        if result["failures"].as_array().is_some_and(|f| !f.is_empty()) {
            return Err(ApiError::ElasticError(
                "Failed to reset displaced attendees".into(),
            ));
        }
    }
    Ok(Json(
        json!({"room": room, "trace_id": current_trace_id(), "timestamp_utc": Utc::now()}),
    ))
}

// Deliberate diagnostic baseline for the Phase 4 performance exercise.
// Replaced with one batched request in the immediately following fix commit.
pub async fn audit_rooms_handler(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let started = std::time::Instant::now();
    let rooms: Vec<RoomState> =
        serde_json::from_str(include_str!("../../../seed/kubecon_rooms.json"))?;
    let mut samples = Vec::with_capacity(15);
    for index in 0..15 {
        let body = elastic_json(state.http().get(format!(
            "{}/rooms_state/_doc/{}",
            state.elastic().base_url,
            rooms[index % rooms.len()].room_id
        )))
        .await?;
        samples.push(serde_json::from_value::<RoomState>(
            body["_source"].clone(),
        )?);
    }
    Ok(Json(
        json!({"rooms": samples, "elastic_requests": 15, "elapsed_ms": started.elapsed().as_secs_f64() * 1000.0, "trace_id": current_trace_id(), "timestamp_utc": Utc::now()}),
    ))
}
