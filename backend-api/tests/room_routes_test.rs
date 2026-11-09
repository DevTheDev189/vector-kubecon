use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use backend_api::{app_router, models::SseEvent, AppConfig, AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn call(app: &Router, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn checkins_surge_and_reset_preserve_live_state() {
    let mut config = AppConfig::from_env();
    config.openai_api_key = None;
    let state = AppState::new(config);
    state.elastic().ensure_indices_and_seed().await.unwrap();
    let mut events = state.sse_sender().subscribe();
    let app = app_router(state.clone());
    let (status, reset) = call(&app, "POST", "/api/surge/reset", json!({})).await;
    assert_eq!(status, StatusCode::OK, "{reset}");
    let (status, rooms) = call(&app, "GET", "/api/rooms", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rooms["rooms"].as_array().unwrap().len(), 12);
    for payload in [
        json!({"room_id":"../bad", "delta":1}),
        json!({"room_id":"ballroom-b", "delta":1, "decibel_reading":-1}),
    ] {
        assert_eq!(
            call(&app, "POST", "/api/rooms/checkin", payload).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/rooms/checkin",
            json!({"room_id":"absent", "delta":1})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    // Concurrent arrivals cannot overwrite each other's increments.
    let arrivals = (0..4).map(|_| {
        call(
            &app,
            "POST",
            "/api/rooms/checkin",
            json!({"room_id":"ballroom-b", "delta":1}),
        )
    });
    for (status, _) in futures::future::join_all(arrivals).await {
        assert_eq!(status, StatusCode::OK);
    }
    let (_, checked) = call(
        &app,
        "POST",
        "/api/rooms/checkin",
        json!({"room_id":"ballroom-b", "delta":0}),
    )
    .await;
    assert_eq!(checked["room"]["capacity_current"], 146);
    let (status, surge) = call(&app, "POST", "/api/surge", json!({})).await;
    assert_eq!(status, StatusCode::OK, "{surge}");
    assert_eq!(surge["room"]["capacity_current"], 150);
    assert_eq!(surge["room"]["occupancy_ratio"], 1.0);
    let alert = &surge["surge"]["data"];
    let alternative = &alert["recommended_alternative"];
    assert!(alternative["room_id"].is_string(), "{surge}");
    assert_ne!(alternative["room_id"], "ballroom-b");
    assert!(alternative["occupancy_ratio"].as_f64().unwrap() < 0.98);
    assert_eq!(alternative["is_rerouted_alternative"], true);
    assert_eq!(alert["bumped_hallway_peers"].as_array().unwrap().len(), 2);
    let talk: Value = state
        .http()
        .get(format!(
            "{}/conference_talks/_doc/talk-ebpf-rust-mesh",
            state.elastic().base_url
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(talk["_source"]["occupancy_ratio"], 1.0);
    assert!(talk["_source"]["accessibility"].is_object());
    let mut saw_surge = false;
    while let Ok(event) = events.try_recv() {
        if matches!(event, SseEvent::CrowdSurgeAlert { .. }) {
            saw_surge = true;
        }
    }
    assert!(saw_surge);
    let (status, reset) = call(&app, "POST", "/api/surge/reset", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reset["room"]["capacity_current"], 142);
    assert_eq!(reset["room"]["decibel_level"], 68.0);
    // Large departures are clamped at zero, including i32 bounds.
    let (_, empty) = call(
        &app,
        "POST",
        "/api/rooms/checkin",
        json!({"room_id":"ballroom-b", "delta":i32::MIN}),
    )
    .await;
    assert_eq!(empty["room"]["capacity_current"], 0);
    assert_eq!(
        call(&app, "POST", "/api/surge/reset", json!({})).await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn room_audit_retains_all_samples_and_reports_latency() {
    let mut config = AppConfig::from_env();
    config.openai_api_key = None;
    let state = AppState::new(config);
    state.elastic().ensure_indices_and_seed().await.unwrap();
    let app = app_router(state);
    let mut elapsed = Vec::new();
    for _ in 0..7 {
        let (status, audit) = call(&app, "POST", "/api/rooms/audit-unbatched", json!({})).await;
        assert_eq!(status, StatusCode::OK, "{audit}");
        assert_eq!(audit["rooms"].as_array().unwrap().len(), 15);
        assert!(audit["rooms"][0]["accessibility"].is_object());
        elapsed.push(audit["elapsed_ms"].as_f64().unwrap());
    }
    elapsed.sort_by(f64::total_cmp);
    println!(
        "room_audit elapsed_ms: samples={elapsed:?}, median={}",
        elapsed[3]
    );
}
