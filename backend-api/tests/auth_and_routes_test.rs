mod common;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use backend_api::{
    app_router,
    auth::{sign_session_cookie, verify_session_cookie, SessionToken, COOKIE_NAME},
    state::AppState,
};

#[tokio::test]
async fn test_hmac_session_cookie_signing_and_verification() {
    let secret = b"my-ultra-secret-test-key-32bytes";
    let token = SessionToken::new("kelseyhightower".to_string(), 110292);

    let cookie = sign_session_cookie(&token, secret, false);
    assert_eq!(cookie.name(), COOKIE_NAME);

    let verified = verify_session_cookie(cookie.value(), secret);
    assert!(verified.is_some());
    let verified_token = verified.unwrap();
    assert_eq!(verified_token.username, "kelseyhightower");
    assert_eq!(verified_token.github_id, 110292);

    // Tampered cookie verification must fail
    let tampered_val = format!("{}tampered", cookie.value());
    assert!(verify_session_cookie(&tampered_val, secret).is_none());
}

#[tokio::test]
async fn test_full_api_auth_and_routes_flow() {
    let mut config = common::config();
    config.port = 8091;
    let state = AppState::new(config.clone());

    // Ensure Elasticsearch indices are ready
    let _ = state.elastic().ensure_indices_and_seed().await;

    let app = app_router(state);

    // 1. GET /api/health
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Authenticated fixture; production sessions are issued only by OAuth.
    let cookie_header_val = common::cookie(&config, "kelseyhightower", 110292);

    // 3. GET /api/state with cookie
    let state_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/state")
                .header(header::COOKIE, &cookie_header_val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let state_status = state_resp.status();
    let state_bytes = axum::body::to_bytes(state_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let state_text = String::from_utf8_lossy(&state_bytes);
    assert_eq!(
        state_status,
        StatusCode::OK,
        "State response failed with: {}",
        state_text
    );

    // 4. PATCH /api/sensory (Toggle Sensory Mode)
    let sensory_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/sensory")
                .header(header::COOKIE, &cookie_header_val)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "sensory_mode_enabled": true, "max_decibels": 45.0 }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sensory_resp.status(), StatusCode::OK);

    // 5. GET /api/handshake/badge
    let badge_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/handshake/badge")
                .header(header::COOKIE, &cookie_header_val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(badge_resp.status(), StatusCode::OK);

    // 6. POST /api/handshake/connect
    let conn_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/handshake/connect")
                .header(header::COOKIE, &cookie_header_val)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "peer_username": "zircon-dev",
                        "room_id": "ballroom-b",
                        "talk_title": "Zero-Overhead Service Mesh with eBPF and Rust"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conn_resp.status(), StatusCode::OK);

    let conn_body_bytes = axum::body::to_bytes(conn_resp.into_body(), 1024 * 64)
        .await
        .unwrap();
    let conn_json: Value = serde_json::from_slice(&conn_body_bytes).unwrap();
    assert!(
        conn_json["handshake"]["draft_linkedin_dm"].is_string(),
        "Must generate drafted LinkedIn DM"
    );
}
