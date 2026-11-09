use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;

use crate::{
    auth::RequireAuthUser,
    error::ApiError,
    models::{BuilderProfile, HandshakeRecord, QrHandshakeToken, SseEvent},
    state::AppState,
    telemetry::current_trace_id,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub peer_username: String,
    pub pin_code: Option<String>,
    pub room_id: Option<String>,
    pub talk_title: Option<String>,
}

#[tracing::instrument(skip_all, fields(http.route = "/api/handshake/badge"))]
pub async fn get_badge_token_handler(
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
        serde_json::from_value(src)
            .unwrap_or_else(|_| crate::routes::profile_routes::fallback_default_profile(&username))
    } else {
        crate::routes::profile_routes::fallback_default_profile(&username)
    };

    let now_utc = Utc::now();
    let payload = format!(
        "{}:{}:{}",
        profile.username,
        profile.pin_code,
        now_utc.timestamp()
    );

    let mut mac = HmacSha256::new_from_slice(&state.config().session_secret).unwrap();
    mac.update(payload.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());

    let badge = QrHandshakeToken {
        username: profile.username.clone(),
        pin_code: profile.pin_code.clone(),
        current_room_id: profile.current_room_id.clone(),
        active_talk_id: Some("talk-ebpf-rust-mesh".to_string()),
        issued_at_utc: now_utc,
        signature: sig,
    };

    Ok(Json(json!({
        "badge": badge,
        "pin_code": profile.pin_code,
        "qr_data_uri": format!("vector:connect?u={}&pin={}&sig={}", profile.username, profile.pin_code, badge.signature),
        "trace_id": current_trace_id(),
        "timestamp_utc": now_utc
    })))
}

#[tracing::instrument(skip_all, fields(http.route = "/api/handshake/connect"))]
pub async fn connect_handshake_handler(
    State(state): State<AppState>,
    user: RequireAuthUser,
    Json(payload): Json<ConnectRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let initiator_username = user.0.username;

    let peer_username = payload.peer_username.trim().to_string();

    // Fetch initiator
    let init_url = format!(
        "{}/builders/_doc/{}",
        state.elastic().base_url,
        initiator_username
    );
    let init_resp = state.http().get(&init_url).send().await?;
    let initiator: BuilderProfile = if init_resp.status().is_success() {
        let body: serde_json::Value = init_resp.json().await?;
        serde_json::from_value(body["_source"].clone()).unwrap_or_else(|_| {
            crate::routes::profile_routes::fallback_default_profile(&initiator_username)
        })
    } else {
        crate::routes::profile_routes::fallback_default_profile(&initiator_username)
    };

    // Fetch peer
    let peer_url = format!(
        "{}/builders/_doc/{}",
        state.elastic().base_url,
        peer_username
    );
    let peer_resp = state.http().get(&peer_url).send().await?;
    let peer: BuilderProfile = if peer_resp.status().is_success() {
        let body: serde_json::Value = peer_resp.json().await?;
        serde_json::from_value(body["_source"].clone()).unwrap_or_else(|_| {
            crate::routes::profile_routes::fallback_default_profile(&peer_username)
        })
    } else {
        crate::routes::profile_routes::fallback_default_profile(&peer_username)
    };

    let room_id = payload
        .room_id
        .unwrap_or_else(|| initiator.current_room_id.clone());
    let room_name = if room_id == "ballroom-b" {
        "Grand Ballroom B"
    } else if room_id == "room-250a" {
        "Room 250A (Alpine Room)"
    } else {
        "Salt Palace North Concourse"
    };

    let talk_title = payload
        .talk_title
        .as_deref()
        .or(Some("Zero-Overhead Service Mesh with eBPF and Rust"));

    // Call OpenAI / OpenRouter engine to draft hyper-relevant followup messages
    let drafted = state
        .openai()
        .draft_handshake_followup(&initiator, &peer, room_name, talk_title)
        .await?;

    let mut shared_tags = Vec::new();
    for tag in &initiator.dna_tags {
        if peer.dna_tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
            shared_tags.push(tag.clone());
        }
    }

    let handshake_id = uuid::Uuid::new_v4().to_string();
    let record = HandshakeRecord {
        handshake_id: handshake_id.clone(),
        initiator_username: initiator.username.clone(),
        peer_username: peer.username.clone(),
        peer_display_name: peer.display_name.clone(),
        peer_avatar_url: peer.avatar_url.clone(),
        peer_persona_title: peer.persona_title.clone(),
        peer_github_url: peer.github_url.clone(),
        peer_linkedin_url: peer.linkedin_url.clone(),
        met_room_id: room_id,
        met_room_name: room_name.to_string(),
        met_during_talk_title: talk_title.map(|s| s.to_string()),
        shared_dna_tags: shared_tags,
        ai_context_summary: drafted.ai_context_summary,
        draft_linkedin_dm: drafted.draft_linkedin_dm,
        draft_email_subject: drafted.draft_email_subject,
        draft_email_body: drafted.draft_email_body,
        created_at: Utc::now(),
    };

    // Index into Elasticsearch handshakes
    let hs_url = format!(
        "{}/handshakes/_doc/{}",
        state.elastic().base_url,
        handshake_id
    );
    let _ = state.http().put(&hs_url).json(&record).send().await;
    let _ = state.elastic().refresh_index("handshakes").await;

    // Broadcast SSE
    let trace_id = current_trace_id();
    state.broadcast_sse(SseEvent::HandshakeCompleted {
        handshake: record.clone(),
        trace_id: trace_id.clone(),
        timestamp_utc: Utc::now(),
    });

    Ok(Json(json!({
        "handshake": record,
        "trace_id": trace_id,
        "timestamp_utc": Utc::now()
    })))
}

mod hex {
    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        data.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}
