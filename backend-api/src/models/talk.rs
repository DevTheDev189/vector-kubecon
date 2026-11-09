use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SensoryLoad {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessibilityMeta {
    pub wheelchair_accessible: bool,
    pub asl_interpreter: bool,
    pub live_captioning: bool,
    pub sensory_load: SensoryLoad,
    pub quiet_zone_nearby: String,
}

impl Default for AccessibilityMeta {
    fn default() -> Self {
        Self {
            wheelchair_accessible: true,
            asl_interpreter: false,
            live_captioning: true,
            sensory_load: SensoryLoad::Medium,
            quiet_zone_nearby: "Room 204B (2nd Floor North)".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MapPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomZone {
    MainHall,
    NorthWing,
    SouthWing,
    Mezzanine,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConferenceTalk {
    pub talk_id: String,
    pub title: String,
    pub abstract_text: String,
    pub speaker: String,
    pub speaker_github: String,
    pub company: String,
    pub track: String,
    pub tags: Vec<String>,
    pub room_id: String,
    pub room_name: String,
    pub floor: i32,
    pub zone: RoomZone,
    pub coordinates: GeoPoint,
    pub map_point: MapPoint,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub capacity_max: i32,
    pub capacity_current: i32,
    pub occupancy_ratio: f64,
    pub decibel_level: f64,
    pub accessibility: AccessibilityMeta,
    #[serde(default)]
    pub embedding: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoredTalk {
    #[serde(flatten)]
    pub talk: ConferenceTalk,
    pub rrf_score: f64,
    pub vector_similarity: f64,
    pub keyword_score: f64,
    pub match_reasons: Vec<String>,
    pub is_rerouted_alternative: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomState {
    pub room_id: String,
    pub room_name: String,
    pub floor: i32,
    pub zone: RoomZone,
    pub coordinates: GeoPoint,
    pub map_point: MapPoint,
    pub capacity_max: i32,
    pub capacity_current: i32,
    pub occupancy_ratio: f64,
    pub decibel_level: f64,
    pub active_talk_id: Option<String>,
    pub active_talk_title: Option<String>,
    pub next_talk_start: Option<DateTime<Utc>>,
    pub is_sensory_haven: bool,
    pub accessibility: AccessibilityMeta,
    pub updated_at: DateTime<Utc>,
}
