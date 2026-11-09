use reqwest::Client;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::error::ApiError;
use crate::models::{BuilderProfile, ConferenceTalk, RoomState};

#[derive(Clone)]
pub struct ElasticClient {
    pub base_url: String,
    pub http: Client,
}

impl ElasticClient {
    pub fn new(base_url: &str, http: Client) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http,
        }
    }

    /// Generates a deterministic unit-normalized 1536-dimensional embedding vector
    /// based on input text tokens and SHA256 projection.
    pub fn generate_deterministic_embedding(text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; 1536];
        let words: Vec<&str> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            // Uniform fallback
            let val = (1.0 / (1536.0f32).sqrt()).min(1.0);
            return vec![val; 1536];
        }

        for word in &words {
            let lower = word.to_lowercase();
            let mut hasher = Sha256::new();
            hasher.update(lower.as_bytes());
            let hash = hasher.finalize();

            for chunk_idx in 0..8 {
                let offset = (chunk_idx * 4) as usize;
                let seed_u32 = u32::from_le_bytes([
                    hash[offset],
                    hash[offset + 1],
                    hash[offset + 2],
                    hash[offset + 3],
                ]);
                let target_dim = (seed_u32 as usize) % 1536;
                let sign = if (seed_u32 & 0x80000000) == 0 {
                    1.0f32
                } else {
                    -1.0f32
                };
                let magnitude = ((seed_u32 & 0x0000FFFF) as f32) / 65535.0f32;
                vector[target_dim] += sign * (0.5f32 + magnitude);
            }
        }

        // Compute L2 norm
        let norm_sq: f32 = vector.iter().map(|v| v * v).sum();
        let norm = norm_sq.sqrt();
        if norm > 1e-6 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        } else {
            let val = (1.0 / (1536.0f32).sqrt()).min(1.0);
            vector = vec![val; 1536];
        }

        vector
    }

    /// Verifies and creates the 4 Elasticsearch indices with dense_vector mappings,
    /// then seeds initial datasets from seed/*.json if indices are empty.
    pub async fn ensure_indices_and_seed(&self) -> Result<(), ApiError> {
        self.create_index_if_missing(
            "conference_talks",
            json!({
                "mappings": {
                    "properties": {
                        "talk_id": { "type": "keyword" },
                        "title": { "type": "text" },
                        "abstract_text": { "type": "text" },
                        "speaker": { "type": "text" },
                        "speaker_github": { "type": "keyword" },
                        "company": { "type": "keyword" },
                        "track": { "type": "keyword" },
                        "tags": { "type": "keyword" },
                        "room_id": { "type": "keyword" },
                        "room_name": { "type": "keyword" },
                        "floor": { "type": "integer" },
                        "zone": { "type": "keyword" },
                        "occupancy_ratio": { "type": "float" },
                        "decibel_level": { "type": "float" },
                        "start_time": { "type": "date" },
                        "end_time": { "type": "date" },
                        "embedding": {
                            "type": "dense_vector",
                            "dims": 1536,
                            "index": true,
                            "similarity": "cosine"
                        }
                    }
                }
            }),
        )
        .await?;

        self.create_index_if_missing(
            "builders",
            json!({
                "mappings": {
                    "properties": {
                        "username": { "type": "keyword" },
                        "github_id": { "type": "long" },
                        "display_name": { "type": "text" },
                        "persona_title": { "type": "text" },
                        "dna_tags": { "type": "keyword" },
                        "top_languages": { "type": "keyword" },
                        "current_room_id": { "type": "keyword" },
                        "bumped_from_talk_id": { "type": "keyword" },
                        "pin_code": { "type": "keyword" },
                        "updated_at": { "type": "date" },
                        "embedding": {
                            "type": "dense_vector",
                            "dims": 1536,
                            "index": true,
                            "similarity": "cosine"
                        }
                    }
                }
            }),
        )
        .await?;

        self.create_index_if_missing(
            "rooms_state",
            json!({
                "mappings": {
                    "properties": {
                        "room_id": { "type": "keyword" },
                        "room_name": { "type": "keyword" },
                        "floor": { "type": "integer" },
                        "zone": { "type": "keyword" },
                        "occupancy_ratio": { "type": "float" },
                        "decibel_level": { "type": "float" },
                        "active_talk_id": { "type": "keyword" },
                        "is_sensory_haven": { "type": "boolean" },
                        "updated_at": { "type": "date" }
                    }
                }
            }),
        )
        .await?;

        self.create_index_if_missing(
            "handshakes",
            json!({
                "mappings": {
                    "properties": {
                        "handshake_id": { "type": "keyword" },
                        "initiator_username": { "type": "keyword" },
                        "peer_username": { "type": "keyword" },
                        "created_at": { "type": "date" }
                    }
                }
            }),
        )
        .await?;

        // Seed data if empty
        self.seed_rooms_if_empty().await?;
        self.seed_talks_if_empty().await?;
        self.seed_builders_if_empty().await?;
        self.backfill_additional_demo_data().await?;

        Ok(())
    }

    /// Create missing demo documents in existing databases without replacing
    /// signed-in profiles, room check-ins, or other data changed during a demo.
    pub async fn backfill_additional_demo_data(&self) -> Result<(), ApiError> {
        let read_seed = |name: &str| -> Result<String, std::io::Error> {
            let local = Path::new("seed").join(name);
            std::fs::read_to_string(if local.exists() {
                local
            } else {
                Path::new("../seed").join(name)
            })
        };
        let rooms: Vec<RoomState> = serde_json::from_str(&read_seed("additional_rooms.json")?)?;
        let mut talks: Vec<ConferenceTalk> =
            serde_json::from_str(&read_seed("additional_talks.json")?)?;
        let mut builders: Vec<BuilderProfile> =
            serde_json::from_str(&read_seed("additional_builders.json")?)?;
        let mut body = String::new();
        let mut append = |index: &str, id: &str, document: Value| -> Result<(), ApiError> {
            body.push_str(&serde_json::to_string(
                &json!({"create": {"_index": index, "_id": id}}),
            )?);
            body.push('\n');
            body.push_str(&serde_json::to_string(&document)?);
            body.push('\n');
            Ok(())
        };
        for room in rooms {
            append("rooms_state", &room.room_id, serde_json::to_value(&room)?)?;
        }
        for talk in &mut talks {
            if talk.embedding.is_empty() {
                talk.embedding = Self::generate_deterministic_embedding(&format!(
                    "{} {} {}",
                    talk.title,
                    talk.abstract_text,
                    talk.tags.join(" ")
                ));
            }
            append(
                "conference_talks",
                &talk.talk_id,
                serde_json::to_value(&talk)?,
            )?;
        }
        for builder in &mut builders {
            if builder.embedding.is_empty() {
                builder.embedding = Self::generate_deterministic_embedding(&format!(
                    "{} {} {}",
                    builder.persona_title,
                    builder.dna_tags.join(" "),
                    builder.top_languages.join(" ")
                ));
            }
            append(
                "builders",
                &builder.username,
                serde_json::to_value(&builder)?,
            )?;
        }
        let response: Value = self
            .http
            .post(format!("{}/_bulk?refresh=wait_for", self.base_url))
            .header("content-type", "application/x-ndjson")
            .body(body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let items = response["items"]
            .as_array()
            .ok_or_else(|| ApiError::ElasticError("Missing demo backfill results".into()))?;
        let mut created = 0;
        for item in items {
            match item["create"]["status"].as_u64() {
                Some(201) => created += 1,
                Some(409) => {} // Existing document belongs to the running app.
                _ => {
                    return Err(ApiError::ElasticError(
                        "Unable to create additional demo data".into(),
                    ))
                }
            }
        }
        tracing::info!(
            created,
            existing = items.len() - created,
            "Additional demo seed backfill complete"
        );
        Ok(())
    }

    async fn create_index_if_missing(&self, index: &str, mapping: Value) -> Result<(), ApiError> {
        let exists_url = format!("{}/{}", self.base_url, index);
        let resp = self.http.head(&exists_url).send().await?;

        if resp.status().as_u16() == 404 {
            let create_url = format!("{}/{}", self.base_url, index);
            let create_resp = self.http.put(&create_url).json(&mapping).send().await?;
            if !create_resp.status().is_success() {
                let err_text = create_resp.text().await.unwrap_or_default();
                tracing::warn!(index = %index, error = %err_text, "Failed to create Elasticsearch index or already exists");
            } else {
                tracing::info!(index = %index, "Created Elasticsearch index with dense_vector mapping");
            }
        }
        Ok(())
    }

    async fn is_index_empty(&self, index: &str) -> Result<bool, ApiError> {
        let count_url = format!("{}/{}/_count", self.base_url, index);
        let resp = self.http.get(&count_url).send().await?;
        if !resp.status().is_success() {
            return Ok(true);
        }
        let body: Value = resp.json().await?;
        let count = body.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
        Ok(count == 0)
    }

    async fn seed_rooms_if_empty(&self) -> Result<(), ApiError> {
        if !self.is_index_empty("rooms_state").await? {
            return Ok(());
        }

        let seed_path = Path::new("seed/kubecon_rooms.json");
        let content = if seed_path.exists() {
            std::fs::read_to_string(seed_path)?
        } else {
            std::fs::read_to_string("../seed/kubecon_rooms.json")?
        };

        let rooms: Vec<RoomState> = serde_json::from_str(&content)?;
        for room in rooms {
            let doc_url = format!("{}/rooms_state/_doc/{}", self.base_url, room.room_id);
            self.http.put(&doc_url).json(&room).send().await?;
        }
        self.refresh_index("rooms_state").await?;
        tracing::info!("Successfully seeded rooms_state in Elasticsearch");
        Ok(())
    }

    async fn seed_talks_if_empty(&self) -> Result<(), ApiError> {
        if !self.is_index_empty("conference_talks").await? {
            return Ok(());
        }

        let seed_path = Path::new("seed/kubecon_talks.json");
        let content = if seed_path.exists() {
            std::fs::read_to_string(seed_path)?
        } else {
            std::fs::read_to_string("../seed/kubecon_talks.json")?
        };

        let mut talks: Vec<ConferenceTalk> = serde_json::from_str(&content)?;
        for talk in &mut talks {
            if talk.embedding.is_empty() {
                let semantic_text = format!(
                    "{} {} {} {}",
                    talk.title,
                    talk.abstract_text,
                    talk.track,
                    talk.tags.join(" ")
                );
                talk.embedding = Self::generate_deterministic_embedding(&semantic_text);
            }
            let doc_url = format!("{}/conference_talks/_doc/{}", self.base_url, talk.talk_id);
            self.http.put(&doc_url).json(&talk).send().await?;
        }
        self.refresh_index("conference_talks").await?;
        tracing::info!("Successfully seeded 25 conference_talks in Elasticsearch");
        Ok(())
    }

    async fn seed_builders_if_empty(&self) -> Result<(), ApiError> {
        if !self.is_index_empty("builders").await? {
            return Ok(());
        }

        let seed_path = Path::new("seed/judge_profiles.json");
        let content = if seed_path.exists() {
            std::fs::read_to_string(seed_path)?
        } else {
            std::fs::read_to_string("../seed/judge_profiles.json")?
        };

        let mut builders: Vec<BuilderProfile> = serde_json::from_str(&content)?;
        for builder in &mut builders {
            if builder.embedding.is_empty() {
                let semantic_text = format!(
                    "{} {} {} {}",
                    builder.display_name,
                    builder.persona_title,
                    builder.dna_tags.join(" "),
                    builder.top_languages.join(" ")
                );
                builder.embedding = Self::generate_deterministic_embedding(&semantic_text);
            }
            let doc_url = format!("{}/builders/_doc/{}", self.base_url, builder.username);
            self.http.put(&doc_url).json(&builder).send().await?;
        }
        self.refresh_index("builders").await?;
        tracing::info!("Successfully seeded 12 builder profiles in Elasticsearch");
        Ok(())
    }

    pub async fn refresh_index(&self, index: &str) -> Result<(), ApiError> {
        let refresh_url = format!("{}/{}/_refresh", self.base_url, index);
        let _ = self.http.post(&refresh_url).send().await;
        Ok(())
    }
}
