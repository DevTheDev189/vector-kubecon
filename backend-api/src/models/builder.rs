use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::talk::GeoPoint;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensoryPreferences {
    pub sensory_mode_enabled: bool,
    pub max_crowd_ratio: f64,
    pub max_decibels: f64,
    pub require_wheelchair_access: bool,
    pub require_live_captions: bool,
    pub prefer_quiet_zones: bool,
}

impl Default for SensoryPreferences {
    fn default() -> Self {
        Self {
            sensory_mode_enabled: false,
            max_crowd_ratio: 0.85,
            max_decibels: 75.0,
            require_wheelchair_access: false,
            require_live_captions: false,
            prefer_quiet_zones: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepoSummary {
    pub name: String,
    pub description: String,
    pub language: String,
    pub stars: u32,
    pub topics: Vec<String>,
    pub readme_excerpt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuilderProfile {
    pub username: String,
    pub github_id: u64,
    pub display_name: String,
    pub avatar_url: String,
    pub bio: String,
    pub company: Option<String>,
    pub github_url: String,
    pub linkedin_url: Option<String>,
    pub email: Option<String>,
    pub persona_title: String,
    pub dna_tags: Vec<String>,
    pub top_languages: Vec<String>,
    pub hallway_icebreaker: String,
    pub ideal_talk_query: String,
    pub top_repos: Vec<RepoSummary>,
    pub current_room_id: String,
    pub coordinates: GeoPoint,
    pub sensory_prefs: SensoryPreferences,
    pub bookmarked_talk_ids: Vec<String>,
    pub bumped_from_talk_id: Option<String>,
    pub pin_code: String,
    #[serde(default)]
    pub embedding: Vec<f32>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchedBuilder {
    pub username: String,
    pub display_name: String,
    pub avatar_url: String,
    pub persona_title: String,
    pub company: Option<String>,
    pub github_url: String,
    pub dna_tags: Vec<String>,
    pub top_languages: Vec<String>,
    pub hallway_icebreaker: String,
    pub current_room_id: String,
    pub similarity_pct: f64,
    pub shared_tags: Vec<String>,
    pub shared_context_reason: String,
    pub distance_meters: f64,
}
