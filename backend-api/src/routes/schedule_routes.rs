use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

use crate::{
    auth::RequireAuthUser,
    error::ApiError,
    models::{BuilderProfile, HandshakeRecord, RoomState, RoomZone},
    services::ScheduleSolver,
    state::AppState,
    telemetry::current_trace_id,
};

#[derive(Deserialize)]
pub struct SearchTalksRequest {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub slot_time: Option<chrono::DateTime<Utc>>,
    pub exclude_full_rooms: Option<bool>,
}

#[derive(Deserialize)]
pub struct BookmarkRequest {
    pub talk_id: String,
}

#[tracing::instrument(skip_all, fields(http.route = "/api/state"))]
pub async fn get_state_handler(
    State(state): State<AppState>,
    user: RequireAuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let username = user.0.username;

    // 1. Fetch Builder Profile
    let profile_url = format!("{}/builders/_doc/{}", state.elastic().base_url, username);
    let profile_resp = state.http().get(&profile_url).send().await?;
    let profile: BuilderProfile = if profile_resp.status().is_success() {
        let body: serde_json::Value = profile_resp.json().await?;
        let src = body
            .get("_source")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        serde_json::from_value(src)
            .unwrap_or_else(|_| crate::routes::profile_routes::fallback_default_profile(&username))
    } else {
        crate::routes::profile_routes::fallback_default_profile(&username)
    };

    // 2. Fetch Rooms
    let rooms_url = format!("{}/rooms_state/_search?size=20", state.elastic().base_url);
    let rooms_resp = state.http().get(&rooms_url).send().await?;
    let mut rooms: Vec<RoomState> = Vec::new();
    if rooms_resp.status().is_success() {
        let body: serde_json::Value = rooms_resp.json().await?;
        if let Some(hits) = body["hits"]["hits"].as_array() {
            for h in hits {
                if let Ok(r) = serde_json::from_value::<RoomState>(h["_source"].clone()) {
                    rooms.push(r);
                }
            }
        }
    }

    // 3. 3-Way Hybrid RRF Search for User
    let (scored_talks, last_elastic_query_dsl) = state
        .elastic()
        .search_talks_hybrid_rrf(&profile, None, false)
        .await?;

    // 4. Build Conflict-Free Schedule DAG
    let schedule = ScheduleSolver::build_conflict_free_schedule(
        &scored_talks,
        &profile.bookmarked_talk_ids,
        profile.sensory_prefs.sensory_mode_enabled,
    );

    // 5. Sensory Havens
    let sensory_havens = state
        .elastic()
        .find_optimal_sensory_haven(&RoomZone::NorthWing)
        .await
        .unwrap_or_default();

    // 6. Hallway Matches
    let hallway_matches = state
        .elastic()
        .match_hallway_builders(
            &profile,
            Some(&profile.current_room_id),
            profile.bumped_from_talk_id.as_deref(),
        )
        .await
        .unwrap_or_default();

    // 7. Handshakes
    let handshakes_url = format!("{}/handshakes/_search", state.elastic().base_url);
    let mut handshakes: Vec<HandshakeRecord> = Vec::new();
    if let Ok(h_resp) = state.http().post(&handshakes_url).json(&json!({"size": 25, "sort": [{"created_at": "desc"}], "query": {"term": {"initiator_username": username}}})).send().await {
        if h_resp.status().is_success() {
            if let Ok(body) = h_resp.json::<serde_json::Value>().await {
                if let Some(hits) = body["hits"]["hits"].as_array() {
                    for h in hits {
                        if let Ok(rec) =
                            serde_json::from_value::<HandshakeRecord>(h["_source"].clone())
                        {
                            handshakes.push(rec);
                        }
                    }
                }
            }
        }
    }

    let trace_id = current_trace_id();

    Ok(Json(json!({
        "can_control_demo": state.config().is_admin(&username),
        "profile": profile,
        "schedule": schedule,
        "scored_talks": scored_talks,
        "rooms": rooms,
        "sensory_havens": sensory_havens,
        "hallway_matches": hallway_matches,
        "handshakes": handshakes,
        "last_elastic_query_dsl": last_elastic_query_dsl,
        "trace_id": trace_id,
        "timestamp_utc": Utc::now()
    })))
}

#[tracing::instrument(skip_all, fields(http.route = "/api/search/talks"))]
pub async fn search_talks_handler(
    State(state): State<AppState>,
    user: RequireAuthUser,
    Json(payload): Json<SearchTalksRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let username = user.0.username;

    let profile_url = format!("{}/builders/_doc/{}", state.elastic().base_url, username);
    let profile_resp = state.http().get(&profile_url).send().await?;
    let mut profile: BuilderProfile = if profile_resp.status().is_success() {
        let body: serde_json::Value = profile_resp.json().await?;
        let src = body
            .get("_source")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        serde_json::from_value(src)
            .unwrap_or_else(|_| crate::routes::profile_routes::fallback_default_profile(&username))
    } else {
        crate::routes::profile_routes::fallback_default_profile(&username)
    };

    if let Some(q) = payload.query {
        if !q.trim().is_empty() {
            profile.ideal_talk_query = q;
        }
    }
    if let Some(t) = payload.tags {
        if !t.is_empty() {
            profile.dna_tags = t;
        }
    }

    let exclude_full = payload.exclude_full_rooms.unwrap_or(false);
    let (scored, query_dsl) = state
        .elastic()
        .search_talks_hybrid_rrf(&profile, payload.slot_time, exclude_full)
        .await?;

    Ok(Json(json!({
        "scored_talks": scored,
        "elastic_query_dsl": query_dsl,
        "trace_id": current_trace_id(),
        "timestamp_utc": Utc::now()
    })))
}

#[tracing::instrument(skip_all, fields(http.route = "/api/schedule/bookmark"))]
pub async fn toggle_bookmark_handler(
    State(state): State<AppState>,
    user: RequireAuthUser,
    Json(payload): Json<BookmarkRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let username = user.0.username;

    let doc_url = format!("{}/builders/_doc/{}", state.elastic().base_url, username);
    let resp = state.http().get(&doc_url).send().await?;

    let mut profile: BuilderProfile = if resp.status().is_success() {
        let body: serde_json::Value = resp.json().await?;
        let src = body
            .get("_source")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        serde_json::from_value(src)
            .unwrap_or_else(|_| crate::routes::profile_routes::fallback_default_profile(&username))
    } else {
        crate::routes::profile_routes::fallback_default_profile(&username)
    };

    if let Some(pos) = profile
        .bookmarked_talk_ids
        .iter()
        .position(|id| id == &payload.talk_id)
    {
        profile.bookmarked_talk_ids.remove(pos);
    } else {
        profile.bookmarked_talk_ids.push(payload.talk_id.clone());
    }
    profile.updated_at = Utc::now();

    let _ = state.http().put(&doc_url).json(&profile).send().await;
    let _ = state.elastic().refresh_index("builders").await;

    let (scored, _) = state
        .elastic()
        .search_talks_hybrid_rrf(&profile, None, false)
        .await?;

    let schedule = ScheduleSolver::build_conflict_free_schedule(
        &scored,
        &profile.bookmarked_talk_ids,
        profile.sensory_prefs.sensory_mode_enabled,
    );

    Ok(Json(json!({
        "bookmarked_talk_ids": profile.bookmarked_talk_ids,
        "schedule": schedule,
        "trace_id": current_trace_id(),
        "timestamp_utc": Utc::now()
    })))
}
