use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QrHandshakeToken {
    pub username: String,
    pub pin_code: String,
    pub current_room_id: String,
    pub active_talk_id: Option<String>,
    pub issued_at_utc: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandshakeRecord {
    pub handshake_id: String,
    pub initiator_username: String,
    pub peer_username: String,
    pub peer_display_name: String,
    pub peer_avatar_url: String,
    pub peer_persona_title: String,
    pub peer_github_url: String,
    pub peer_linkedin_url: Option<String>,
    pub met_room_id: String,
    pub met_room_name: String,
    pub met_during_talk_title: Option<String>,
    pub shared_dna_tags: Vec<String>,
    pub ai_context_summary: String,
    pub draft_linkedin_dm: String,
    pub draft_email_subject: String,
    pub draft_email_body: String,
    pub created_at: DateTime<Utc>,
}
