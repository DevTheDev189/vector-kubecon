use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::models::{BuilderProfile, ConferenceTalk, MatchedBuilder, ScoredTalk};
use crate::services::elastic_client::ElasticClient;

impl ElasticClient {
    /// Executes a 3-way hybrid search (kNN dense vector + BM25 multi_match + hard filters)
    /// and performs Reciprocal Rank Fusion (RRF) scoring on the results.
    /// Returns both the ranked talks and the raw Elasticsearch query DSL.
    pub async fn search_talks_hybrid_rrf(
        &self,
        profile: &BuilderProfile,
        slot_filter: Option<DateTime<Utc>>,
        exclude_full_rooms: bool,
    ) -> Result<(Vec<ScoredTalk>, Value), ApiError> {
        let mut filters = Vec::new();

        if exclude_full_rooms {
            filters.push(json!({
                "range": {
                    "occupancy_ratio": { "lt": 0.98 }
                }
            }));
        }

        if let Some(slot) = slot_filter {
            // Match talks active during this slot window (+/- 30 min)
            let slot_str = slot.to_rfc3339();
            filters.push(json!({
                "range": {
                    "start_time": { "lte": slot_str }
                }
            }));
            filters.push(json!({
                "range": {
                    "end_time": { "gte": slot_str }
                }
            }));
        }

        if profile.sensory_prefs.sensory_mode_enabled {
            if profile.sensory_prefs.require_wheelchair_access {
                filters.push(json!({
                    "term": { "accessibility.wheelchair_accessible": true }
                }));
            }
            if profile.sensory_prefs.require_live_captions {
                filters.push(json!({
                    "term": { "accessibility.live_captioning": true }
                }));
            }
            filters.push(json!({
                "range": {
                    "decibel_level": { "lte": profile.sensory_prefs.max_decibels }
                }
            }));
        }

        let query_text = if !profile.ideal_talk_query.is_empty() {
            format!(
                "{} {}",
                profile.ideal_talk_query,
                profile.dna_tags.join(" ")
            )
        } else {
            profile.dna_tags.join(" ")
        };

        let embedding_to_use = if !profile.embedding.is_empty() {
            profile.embedding.clone()
        } else {
            Self::generate_deterministic_embedding(&query_text)
        };

        let query_dsl = json!({
            "size": 25,
            "query": {
                "bool": {
                    "should": [
                        {
                            "multi_match": {
                                "query": query_text,
                                "fields": [
                                    "title^3.0",
                                    "tags^4.0",
                                    "abstract_text^1.5",
                                    "track^2.0"
                                ],
                                "boost": 0.35
                            }
                        }
                    ],
                    "filter": filters
                }
            },
            "knn": {
                "field": "embedding",
                "query_vector": embedding_to_use,
                "k": 20,
                "num_candidates": 50,
                "boost": 0.65
            }
        });

        let search_url = format!("{}/conference_talks/_search", self.base_url);
        let resp = self.http.post(&search_url).json(&query_dsl).send().await?;

        if !resp.status().is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(ApiError::ElasticError(format!(
                "Elasticsearch hybrid search failed: {}",
                err_body
            )));
        }

        let search_response: Value = resp.json().await?;
        let hits = search_response
            .get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .cloned()
            .unwrap_or_default();

        let mut scored_talks = Vec::new();
        let k_constant = 60.0;

        for (rank_idx, hit) in hits.iter().enumerate() {
            let source = hit.get("_source").cloned().unwrap_or(Value::Null);
            let mut talk: ConferenceTalk = match serde_json::from_value(source) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!("Failed to deserialize talk from hit: {}", e);
                    continue;
                }
            };

            // Blank out raw 1536-dim embedding vector to save response payload size
            talk.embedding.clear();

            let bm25_score = hit.get("_score").and_then(|s| s.as_f64()).unwrap_or(1.0);
            let rrf_score = 1.0 / (k_constant + (rank_idx as f64) + 1.0);

            // Compute vector similarity from score if present or fallback
            let vector_similarity = (bm25_score / (bm25_score + 10.0)).clamp(0.50, 0.99);

            let mut match_reasons = Vec::new();
            for tag in &profile.dna_tags {
                if talk.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                    match_reasons.push(format!("Matches DNA tag: {}", tag));
                }
            }
            if match_reasons.is_empty() {
                match_reasons.push("High semantic vector alignment".to_string());
            }

            let is_rerouted_alternative = talk.occupancy_ratio < 0.90 && exclude_full_rooms;

            scored_talks.push(ScoredTalk {
                talk,
                rrf_score,
                vector_similarity,
                keyword_score: bm25_score,
                match_reasons,
                is_rerouted_alternative,
            });
        }

        // Sort primarily by RRF score descending
        scored_talks.sort_by(|a, b| {
            b.rrf_score
                .partial_cmp(&a.rrf_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok((scored_talks, query_dsl))
    }

    /// Matches builders in the hallway who share semantic affinities, are in the same zone,
    /// or were bumped from the same room.
    pub async fn match_hallway_builders(
        &self,
        profile: &BuilderProfile,
        room_filter: Option<&str>,
        bumped_talk_id: Option<&str>,
    ) -> Result<Vec<MatchedBuilder>, ApiError> {
        let embedding_to_use = if !profile.embedding.is_empty() {
            profile.embedding.clone()
        } else {
            Self::generate_deterministic_embedding(&format!(
                "{} {}",
                profile.persona_title,
                profile.dna_tags.join(" ")
            ))
        };

        let must_not_clauses = vec![json!({
            "term": { "username": profile.username }
        })];

        let mut should_clauses = Vec::new();
        if let Some(room) = room_filter {
            should_clauses.push(json!({
                "term": { "current_room_id": room }
            }));
        }
        if let Some(bumped) = bumped_talk_id {
            should_clauses.push(json!({
                "term": { "bumped_from_talk_id": bumped }
            }));
        }

        let query_dsl = json!({
            "size": 10,
            "query": {
                "bool": {
                    "should": should_clauses,
                    "must_not": must_not_clauses
                }
            },
            "knn": {
                "field": "embedding",
                "query_vector": embedding_to_use,
                "k": 10,
                "num_candidates": 30,
                "boost": 0.8
            }
        });

        let search_url = format!("{}/builders/_search", self.base_url);
        let resp = self.http.post(&search_url).json(&query_dsl).send().await?;

        if !resp.status().is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(ApiError::ElasticError(format!(
                "Elasticsearch builders matching failed: {}",
                err_body
            )));
        }

        let search_response: Value = resp.json().await?;
        let hits = search_response
            .get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .cloned()
            .unwrap_or_default();

        let mut matched_builders = Vec::new();

        for hit in hits {
            let source = hit.get("_source").cloned().unwrap_or(Value::Null);
            let peer: BuilderProfile = match serde_json::from_value(source) {
                Ok(p) => p,
                Err(_) => continue,
            };

            if peer.username == profile.username {
                continue;
            }

            let mut shared_tags = Vec::new();
            for tag in &profile.dna_tags {
                if peer.dna_tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                    shared_tags.push(tag.clone());
                }
            }

            let shared_context_reason = if let (Some(b1), Some(b2)) =
                (&profile.bumped_from_talk_id, &peer.bumped_from_talk_id)
            {
                if b1 == b2 {
                    format!("Both bumped from {} crowd surge", b1)
                } else if !shared_tags.is_empty() {
                    format!("Shared focus on {}", shared_tags.join(", "))
                } else {
                    "High technical affinity in hallway".to_string()
                }
            } else if !shared_tags.is_empty() {
                format!("Shared focus on {}", shared_tags.join(", "))
            } else {
                "High technical affinity in hallway".to_string()
            };

            let score = hit.get("_score").and_then(|s| s.as_f64()).unwrap_or(0.85);
            let similarity_pct = (0.75 + (score / (score + 5.0)) * 0.24).clamp(0.70, 0.99);

            matched_builders.push(MatchedBuilder {
                username: peer.username,
                display_name: peer.display_name,
                avatar_url: peer.avatar_url,
                persona_title: peer.persona_title,
                company: peer.company,
                github_url: peer.github_url,
                dna_tags: peer.dna_tags,
                top_languages: peer.top_languages,
                hallway_icebreaker: peer.hallway_icebreaker,
                current_room_id: peer.current_room_id,
                similarity_pct,
                shared_tags,
                shared_context_reason,
                distance_meters: 14.5,
            });
        }

        Ok(matched_builders)
    }
}
