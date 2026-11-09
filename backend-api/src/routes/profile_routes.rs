use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

use crate::{
    auth::RequireAuthUser,
    error::ApiError,
    models::{BuilderProfile, RoomZone, SseEvent},
    routes::auth_routes::sync_and_index_profile,
    services::ScheduleSolver,
    state::AppState,
    telemetry::current_trace_id,
};

#[derive(Deserialize)]
pub struct SyncGithubRequest {
    pub username: String,
    pub linkedin_url: Option<String>,
}

#[derive(Deserialize)]
pub struct ToggleSensoryRequest {
    pub sensory_mode_enabled: Option<bool>,
    pub max_decibels: Option<f64>,
    pub require_wheelchair_access: Option<bool>,
    pub require_live_captions: Option<bool>,
}

#[tracing::instrument(skip_all, fields(http.route = "/api/me"))]
pub async fn get_me_handler(
    State(state): State<AppState>,
    user: RequireAuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let username = user.0.username;

    let doc_url = format!("{}/builders/_doc/{}", state.elastic().base_url, username);
    let resp = state.http().get(&doc_url).send().await?;

    let profile: BuilderProfile = if resp.status().is_success() {
        let body: serde_json::Value = resp.json().await?;
        let src = body
            .get("_source")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        serde_json::from_value(src).unwrap_or_else(|_| fallback_default_profile(&username))
    } else {
        fallback_default_profile(&username)
    };

    let trace_id = current_trace_id();

    Ok(Json(json!({
        "can_control_demo": state.config().is_admin(&username),
        "profile": profile,
        "trace_id": trace_id,
        "timestamp_utc": Utc::now()
    })))
}

#[tracing::instrument(skip_all, fields(http.route = "/api/sync-github"))]
pub async fn sync_github_handler(
    State(state): State<AppState>,
    RequireAuthUser(user): RequireAuthUser,
    Json(payload): Json<SyncGithubRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !payload.username.trim().eq_ignore_ascii_case(&user.username) {
        return Err(ApiError::Forbidden(
            "You may only sync your signed-in GitHub account".into(),
        ));
    }
    let profile =
        sync_and_index_profile(&state, &user.username, payload.linkedin_url, None).await?;
    if profile.github_id != user.github_id {
        return Err(ApiError::Unauthorized(
            "GitHub account identity could not be verified. Please sign in again.".into(),
        ));
    }

    // Generate updated schedule
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
        "profile": profile,
        "schedule": schedule,
        "trace_id": current_trace_id(),
        "timestamp_utc": Utc::now()
    })))
}

#[tracing::instrument(skip_all, fields(http.route = "/api/sensory"))]
pub async fn toggle_sensory_handler(
    State(state): State<AppState>,
    user: RequireAuthUser,
    Json(payload): Json<ToggleSensoryRequest>,
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
        serde_json::from_value(src).unwrap_or_else(|_| fallback_default_profile(&username))
    } else {
        fallback_default_profile(&username)
    };

    if let Some(enabled) = payload.sensory_mode_enabled {
        profile.sensory_prefs.sensory_mode_enabled = enabled;
    }
    if let Some(max_db) = payload.max_decibels {
        profile.sensory_prefs.max_decibels = max_db;
    }
    if let Some(wc) = payload.require_wheelchair_access {
        profile.sensory_prefs.require_wheelchair_access = wc;
    }
    if let Some(captions) = payload.require_live_captions {
        profile.sensory_prefs.require_live_captions = captions;
    }
    profile.updated_at = Utc::now();

    // Save back to Elasticsearch
    let _ = state.http().put(&doc_url).json(&profile).send().await;
    let _ = state.elastic().refresh_index("builders").await;

    // Discover top sensory haven
    let havens = state
        .elastic()
        .find_optimal_sensory_haven(&RoomZone::NorthWing)
        .await?;
    let recommended_haven = havens.first().cloned();

    if let Some(haven) = &recommended_haven {
        state.broadcast_sse(SseEvent::SensoryHavenUpdate {
            recommended_haven: haven.clone(),
            reason: "Sensory Haven Mode Activated: Low sound & empty space guaranteed.".to_string(),
            timestamp_utc: Utc::now(),
        });
    }

    // Recalculate schedule
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
        "sensory_mode_enabled": profile.sensory_prefs.sensory_mode_enabled,
        "recommended_haven": recommended_haven,
        "schedule": schedule,
        "trace_id": current_trace_id(),
        "timestamp_utc": Utc::now()
    })))
}

pub fn fallback_default_profile(username: &str) -> BuilderProfile {
    BuilderProfile {
        username: username.to_string(),
        github_id: 981245,
        display_name: if username == "zircon-dev" {
            "Deven Winslow".to_string()
        } else {
            username.to_string()
        },
        avatar_url: format!("https://github.com/{}.png", username),
        bio: "Autonomous Cloud-Native Systems Engineer".to_string(),
        company: Some("Winslow Systems".to_string()),
        github_url: format!("https://github.com/{}", username),
        linkedin_url: None,
        email: None,
        persona_title: "VECTOR Lead Architect".to_string(),
        dna_tags: vec![
            "Rust".to_string(),
            "Axum".to_string(),
            "OpenTelemetry".to_string(),
            "Elasticsearch".to_string(),
            "Kubernetes".to_string(),
        ],
        top_languages: vec!["Rust".to_string(), "TypeScript".to_string()],
        hallway_icebreaker: "Creator of VECTOR: Autonomous KubeCon conference OS.".to_string(),
        ideal_talk_query: "autonomous cloud-native telemetry open telemetry rust".to_string(),
        top_repos: vec![],
        current_room_id: "ballroom-b".to_string(),
        coordinates: crate::models::GeoPoint {
            lat: 40.76630,
            lon: -111.89670,
        },
        sensory_prefs: crate::models::SensoryPreferences::default(),
        bookmarked_talk_ids: vec!["talk-ebpf-rust-mesh".to_string()],
        bumped_from_talk_id: None,
        pin_code: "4829".to_string(),
        embedding: vec![],
        updated_at: Utc::now(),
    }
}
