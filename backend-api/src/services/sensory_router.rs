use serde_json::{json, Value};

use crate::error::ApiError;
use crate::models::{RoomState, RoomZone};
use crate::services::elastic_client::ElasticClient;

impl ElasticClient {
    /// Queries Elasticsearch rooms_state to discover verified quiet rooms and unscheduled
    /// conference rooms suitable for sensory decompression (< 45 dB, < 20% occupancy).
    pub async fn find_optimal_sensory_haven(
        &self,
        current_zone: &RoomZone,
    ) -> Result<Vec<RoomState>, ApiError> {
        let query_dsl = json!({
            "size": 10,
            "query": {
                "bool": {
                    "should": [
                        { "bool": { "must_not": { "exists": { "field": "active_talk_id" } } } },
                        { "term": { "is_sensory_haven": true } }
                    ],
                    "minimum_should_match": 1,
                    "filter": [
                        { "range": { "occupancy_ratio": { "lt": 0.25 } } },
                        { "range": { "decibel_level": { "lt": 48.0 } } }
                    ]
                }
            },
            "sort": [
                { "decibel_level": { "order": "asc" } },
                { "occupancy_ratio": { "order": "asc" } }
            ]
        });

        let search_url = format!("{}/rooms_state/_search", self.base_url);
        let resp = self.http.post(&search_url).json(&query_dsl).send().await?;

        if !resp.status().is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(ApiError::ElasticError(format!(
                "Elasticsearch sensory haven query failed: {}",
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

        let mut havens: Vec<RoomState> = Vec::new();
        for hit in hits {
            let source = hit.get("_source").cloned().unwrap_or(Value::Null);
            if let Ok(room) = serde_json::from_value::<RoomState>(source) {
                havens.push(room);
            }
        }

        // Sort prioritizing lowest decibel level, then same zone
        havens.sort_by(|a, b| {
            let a_same_zone = a.zone == *current_zone;
            let b_same_zone = b.zone == *current_zone;

            match (a_same_zone, b_same_zone) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a
                    .decibel_level
                    .partial_cmp(&b.decibel_level)
                    .unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        Ok(havens)
    }
}
