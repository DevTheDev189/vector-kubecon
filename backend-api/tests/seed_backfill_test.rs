mod common;
use backend_api::AppState;
use serde_json::{json, Value};

#[tokio::test]
async fn expanded_demo_backfill_adds_missing_docs_and_preserves_existing_changes() {
    let state = AppState::new(common::config());
    state.elastic().ensure_indices_and_seed().await.unwrap();
    let base = &state.elastic().base_url;
    for (index, min_count) in [
        ("rooms_state", 24),
        ("conference_talks", 49),
        ("builders", 36),
    ] {
        let count: Value = state
            .http()
            .get(format!("{base}/{index}/_count"))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(
            count["count"].as_u64().unwrap() >= min_count,
            "{index}: {count}"
        );
    }
    let url = format!("{base}/builders/_doc/demo-aria-chen");
    let original: Value = state
        .http()
        .get(&url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mut changed = original["_source"].clone();
    changed["persona_title"] = json!("Preserved local demo edit");
    state
        .http()
        .put(&url)
        .json(&changed)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    state
        .elastic()
        .backfill_additional_demo_data()
        .await
        .unwrap();
    let stored: Value = state
        .http()
        .get(&url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    state
        .http()
        .put(format!("{url}?refresh=true"))
        .json(&original["_source"])
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    assert_eq!(
        stored["_source"], changed,
        "Backfill must never replace existing documents"
    );
}
