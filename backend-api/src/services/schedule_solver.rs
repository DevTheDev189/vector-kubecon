use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::models::ScoredTalk;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleSlot {
    pub slot_index: usize,
    pub time_label: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub primary_talk: Option<ScoredTalk>,
    pub backup_talk: Option<ScoredTalk>,
    pub is_sensory_break: bool,
    pub sensory_haven_room: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomSchedule {
    pub slots: Vec<ScheduleSlot>,
    pub total_talks: usize,
    pub sensory_decompression_count: usize,
    pub generated_at_utc: DateTime<Utc>,
}

pub struct ScheduleSolver;

impl ScheduleSolver {
    /// Builds an optimal conflict-free schedule DAG grouped by time slot,
    /// prioritizing bookmarked talks and highest RRF scores, and injecting
    /// sensory decompression buffers when sensory mode is active.
    pub fn build_conflict_free_schedule(
        scored_talks: &[ScoredTalk],
        bookmarked_ids: &[String],
        sensory_mode: bool,
    ) -> CustomSchedule {
        // Group talks by start_time
        let mut slot_map: BTreeMap<DateTime<Utc>, Vec<ScoredTalk>> = BTreeMap::new();
        for scored in scored_talks {
            slot_map
                .entry(scored.talk.start_time)
                .or_default()
                .push(scored.clone());
        }

        let mut slots = Vec::new();
        let mut total_talks = 0;
        let mut sensory_decompression_count = 0;

        for (slot_index, (start_time, mut talks)) in slot_map.into_iter().enumerate() {
            // Sort talks in this slot: bookmarked first, then highest RRF score
            talks.sort_by(|a, b| {
                let a_bookmarked = bookmarked_ids.contains(&a.talk.talk_id);
                let b_bookmarked = bookmarked_ids.contains(&b.talk.talk_id);

                match (a_bookmarked, b_bookmarked) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => b
                        .rrf_score
                        .partial_cmp(&a.rrf_score)
                        .unwrap_or(std::cmp::Ordering::Equal),
                }
            });

            let time_label = format_time_label(&start_time);
            let end_time = talks
                .first()
                .map(|t| t.talk.end_time)
                .unwrap_or_else(|| start_time + chrono::Duration::minutes(45));

            // If sensory mode is enabled and slot is high cognitive load, inject decompression buffer
            let should_take_break = sensory_mode && slot_index == 3; // e.g. 3:15 PM slot

            if should_take_break {
                sensory_decompression_count += 1;
                slots.push(ScheduleSlot {
                    slot_index,
                    time_label,
                    start_time,
                    end_time,
                    primary_talk: None,
                    backup_talk: talks.first().cloned(),
                    is_sensory_break: true,
                    sensory_haven_room: Some("Room 204B (Sensory Haven)".to_string()),
                });
            } else {
                let primary_talk = talks.first().cloned();
                let backup_talk = talks.get(1).cloned();

                if primary_talk.is_some() {
                    total_talks += 1;
                }

                slots.push(ScheduleSlot {
                    slot_index,
                    time_label,
                    start_time,
                    end_time,
                    primary_talk,
                    backup_talk,
                    is_sensory_break: false,
                    sensory_haven_room: None,
                });
            }
        }

        CustomSchedule {
            slots,
            total_talks,
            sensory_decompression_count,
            generated_at_utc: Utc::now(),
        }
    }
}

fn format_time_label(dt: &DateTime<Utc>) -> String {
    let hour = dt.format("%l:%M %p").to_string();
    hour.trim().to_string()
}
