use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    builder::MatchedBuilder,
    handshake::HandshakeRecord,
    talk::{RoomState, ScoredTalk},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum SseEvent {
    #[serde(rename = "room_telemetry_update")]
    RoomTelemetryUpdate {
        room: RoomState,
        trace_id: String,
        timestamp_utc: DateTime<Utc>,
    },
    #[serde(rename = "crowd_surge_alert")]
    CrowdSurgeAlert {
        jammed_room_id: String,
        jammed_room_name: String,
        jammed_talk_id: String,
        jammed_talk_title: String,
        occupancy_current: i32,
        occupancy_max: i32,
        decibel_level: f64,
        recommended_alternative: Option<ScoredTalk>,
        bumped_hallway_peers: Vec<MatchedBuilder>,
        trace_id: String,
        timestamp_utc: DateTime<Utc>,
    },
    #[serde(rename = "dna_sync_progress")]
    DnaSyncProgress {
        username: String,
        step: String,
        progress_pct: u8,
        detail: String,
        timestamp_utc: DateTime<Utc>,
    },
    #[serde(rename = "handshake_completed")]
    HandshakeCompleted {
        handshake: HandshakeRecord,
        trace_id: String,
        timestamp_utc: DateTime<Utc>,
    },
    #[serde(rename = "sensory_haven_update")]
    SensoryHavenUpdate {
        recommended_haven: RoomState,
        reason: String,
        timestamp_utc: DateTime<Utc>,
    },
}
