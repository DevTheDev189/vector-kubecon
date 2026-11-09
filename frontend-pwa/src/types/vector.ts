export type UtcDateTime = string;

export type RoomZone = "main_hall" | "north_wing" | "south_wing" | "mezzanine";

export type SensoryLoad = "low" | "medium" | "high";

export interface AccessibilityMeta {
  wheelchair_accessible: boolean;
  asl_interpreter: boolean;
  live_captioning: boolean;
  sensory_load: SensoryLoad;
  quiet_zone_nearby: string;
}

export interface GeoPoint {
  lat: number;
  lon: number;
}

export interface MapPoint {
  x: number;
  y: number;
}

export interface ConferenceTalk {
  talk_id: string;
  title: string;
  abstract_text: string;
  speaker: string;
  speaker_github: string;
  company: string;
  track: string;
  tags: string[];
  room_id: string;
  room_name: string;
  floor: number;
  zone: RoomZone;
  coordinates: GeoPoint;
  map_point: MapPoint;
  start_time: UtcDateTime;
  end_time: UtcDateTime;
  capacity_max: number;
  capacity_current: number;
  occupancy_ratio: number;
  decibel_level: number;
  accessibility: AccessibilityMeta;
  embedding: number[];
}

export interface RoomState {
  room_id: string;
  room_name: string;
  floor: number;
  zone: RoomZone;
  coordinates: GeoPoint;
  map_point: MapPoint;
  capacity_max: number;
  capacity_current: number;
  occupancy_ratio: number;
  decibel_level: number;
  active_talk_id: string | null;
  active_talk_title: string | null;
  next_talk_start: UtcDateTime | null;
  is_sensory_haven: boolean;
  accessibility: AccessibilityMeta;
  updated_at: UtcDateTime;
}

export interface SensoryPreferences {
  sensory_mode_enabled: boolean;
  max_crowd_ratio: number;
  max_decibels: number;
  require_wheelchair_access: boolean;
  require_live_captions: boolean;
  prefer_quiet_zones: boolean;
}

export interface RepoSummary {
  name: string;
  description: string;
  language: string;
  stars: number;
  topics: string[];
  readme_excerpt: string;
}

export interface BuilderProfile {
  username: string;
  github_id: number;
  display_name: string;
  avatar_url: string;
  bio: string;
  company: string | null;
  github_url: string;
  linkedin_url: string | null;
  email: string | null;
  persona_title: string;
  dna_tags: string[];
  top_languages: string[];
  hallway_icebreaker: string;
  ideal_talk_query: string;
  top_repos: RepoSummary[];
  current_room_id: string;
  coordinates: GeoPoint;
  sensory_prefs: SensoryPreferences;
  bookmarked_talk_ids: string[];
  bumped_from_talk_id: string | null;
  pin_code: string;
  embedding: number[];
  updated_at: UtcDateTime;
}

export interface MatchedBuilder {
  username: string;
  display_name: string;
  avatar_url: string;
  persona_title: string;
  company: string | null;
  github_url: string;
  dna_tags: string[];
  top_languages: string[];
  hallway_icebreaker: string;
  current_room_id: string;
  similarity_pct: number;
  shared_tags: string[];
  shared_context_reason: string;
  distance_meters: number;
}

export interface QrHandshakeToken {
  username: string;
  pin_code: string;
  current_room_id: string;
  active_talk_id: string | null;
  issued_at_utc: UtcDateTime;
  signature: string;
}

export interface HandshakeRecord {
  handshake_id: string;
  initiator_username: string;
  peer_username: string;
  peer_display_name: string;
  peer_avatar_url: string;
  peer_persona_title: string;
  peer_github_url: string;
  peer_linkedin_url: string | null;
  met_room_id: string;
  met_room_name: string;
  met_during_talk_title: string | null;
  shared_dna_tags: string[];
  ai_context_summary: string;
  draft_linkedin_dm: string;
  draft_email_subject: string;
  draft_email_body: string;
  created_at: UtcDateTime;
}

export interface ScoredTalk extends ConferenceTalk {
  rrf_score: number;
  vector_similarity: number;
  keyword_score: number;
  match_reasons: string[];
  is_rerouted_alternative: boolean;
}

export interface ScheduleSlot {
  slot_index: number;
  time_label: string;
  start_time: UtcDateTime;
  end_time: UtcDateTime;
  primary_talk: ScoredTalk | null;
  backup_talk: ScoredTalk | null;
  is_sensory_break: boolean;
  sensory_haven_room: string | null;
}

export interface CustomSchedule {
  slots: ScheduleSlot[];
  total_talks: number;
  sensory_decompression_count: number;
  generated_at_utc: UtcDateTime;
}

export interface SurgeAlert {
  jammed_room_id: string;
  jammed_room_name: string;
  jammed_talk_id: string;
  jammed_talk_title: string;
  occupancy_current: number;
  occupancy_max: number;
  decibel_level: number;
  recommended_alternative: ScoredTalk | null;
  bumped_hallway_peers: MatchedBuilder[];
  trace_id: string;
  timestamp_utc: UtcDateTime;
}
export interface DnaProgress {
  username: string;
  step: string;
  progress_pct: number;
  detail: string;
  timestamp_utc: UtcDateTime;
}
export type SseEvent =
  | {
      event_type: "room_telemetry_update";
      data: { room: RoomState; trace_id: string; timestamp_utc: UtcDateTime };
    }
  | { event_type: "crowd_surge_alert"; data: SurgeAlert }
  | { event_type: "dna_sync_progress"; data: DnaProgress }
  | {
      event_type: "handshake_completed";
      data: {
        handshake: HandshakeRecord;
        trace_id: string;
        timestamp_utc: UtcDateTime;
      };
    }
  | {
      event_type: "sensory_haven_update";
      data: {
        recommended_haven: RoomState;
        reason: string;
        timestamp_utc: UtcDateTime;
      };
    };
export interface StateResponse {
  can_control_demo: boolean;
  profile: BuilderProfile;
  schedule: CustomSchedule;
  scored_talks: ScoredTalk[];
  rooms: RoomState[];
  sensory_havens: RoomState[];
  hallway_matches: MatchedBuilder[];
  handshakes: HandshakeRecord[];
  last_elastic_query_dsl: unknown;
  trace_id: string;
  timestamp_utc: UtcDateTime;
}
export interface BadgeResponse {
  badge: QrHandshakeToken;
  pin_code: string;
  qr_data_uri: string;
}
