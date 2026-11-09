use chrono::{DateTime, Utc};
use reqwest::Client;

use backend_api::{
    config::AppConfig,
    models::{BuilderProfile, GeoPoint, RoomZone, SensoryPreferences},
    services::{ElasticClient, ScheduleSolver},
};

#[tokio::test]
async fn test_elasticsearch_3way_hybrid_rrf_and_services() {
    let config = AppConfig::from_env();
    let http = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let client = ElasticClient::new(&config.elasticsearch_url, http);

    // 1. Verify index creation & seeding
    let seed_result = client.ensure_indices_and_seed().await;
    assert!(
        seed_result.is_ok(),
        "Failed to ensure indices and seed: {:?}",
        seed_result.err()
    );

    // 2. Build test profile (Kelsey Hightower)
    let profile = BuilderProfile {
        username: "kelseyhightower".to_string(),
        github_id: 110292,
        display_name: "Kelsey Hightower".to_string(),
        avatar_url: "https://avatars.githubusercontent.com/u/110292?v=4".to_string(),
        bio: "Minimalist. eBPF & Kubernetes pioneer.".to_string(),
        company: Some("Community".to_string()),
        github_url: "https://github.com/kelseyhightower".to_string(),
        linkedin_url: None,
        email: None,
        persona_title: "Cloud-Native Pioneer".to_string(),
        dna_tags: vec![
            "eBPF".to_string(),
            "Rust".to_string(),
            "Kubernetes".to_string(),
            "Service Mesh".to_string(),
        ],
        top_languages: vec!["Go".to_string(), "Rust".to_string()],
        hallway_icebreaker: "Let's talk eBPF".to_string(),
        ideal_talk_query: "zero-overhead service mesh ebpf rust".to_string(),
        top_repos: vec![],
        current_room_id: "ballroom-b".to_string(),
        coordinates: GeoPoint {
            lat: 40.76630,
            lon: -111.89670,
        },
        sensory_prefs: SensoryPreferences::default(),
        bookmarked_talk_ids: vec!["talk-ebpf-rust-mesh".to_string()],
        bumped_from_talk_id: None,
        pin_code: "1942".to_string(),
        embedding: ElasticClient::generate_deterministic_embedding(
            "zero-overhead service mesh ebpf rust",
        ),
        updated_at: Utc::now(),
    };

    // 3. Test Hybrid RRF Search
    let slot_2pm: DateTime<Utc> = "2026-11-12T20:00:00Z".parse().unwrap();
    let (scored_talks, query_dsl) = client
        .search_talks_hybrid_rrf(&profile, Some(slot_2pm), false)
        .await
        .expect("Hybrid RRF search failed");

    assert!(!scored_talks.is_empty(), "Expected non-empty scored talks");
    assert!(
        query_dsl.get("knn").is_some(),
        "Query DSL must contain knn vector search"
    );
    assert!(
        scored_talks[0].rrf_score > 0.0,
        "RRF score must be positive"
    );

    // 4. Test Schedule DAG Solver
    let schedule = ScheduleSolver::build_conflict_free_schedule(
        &scored_talks,
        &profile.bookmarked_talk_ids,
        false,
    );
    assert!(
        !schedule.slots.is_empty(),
        "Schedule must contain time slots"
    );

    // 5. Test Hallway Builder Matching
    let hallway_matches = client
        .match_hallway_builders(&profile, Some("ballroom-b"), None)
        .await
        .expect("Hallway matching failed");
    assert!(
        !hallway_matches.is_empty(),
        "Expected at least one hallway peer match"
    );
    assert_ne!(
        hallway_matches[0].username, profile.username,
        "Should not match oneself"
    );

    // 6. Test Sensory Haven Room Router
    let haven_rooms = client
        .find_optimal_sensory_haven(&RoomZone::NorthWing)
        .await
        .expect("Sensory haven query failed");
    assert!(
        !haven_rooms.is_empty(),
        "Expected at least one sensory haven room"
    );
    assert!(
        haven_rooms[0].decibel_level < 48.0,
        "Haven room must have decibel reading under 48 dB"
    );
    assert!(
        haven_rooms[0].occupancy_ratio < 0.25,
        "Haven room must have low occupancy"
    );
}
