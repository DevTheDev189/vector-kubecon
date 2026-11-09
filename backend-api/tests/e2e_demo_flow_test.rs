mod common;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use backend_api::{app_router, AppConfig, AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn serves_spa_assets_and_preserves_api_errors() {
    let dir = std::env::temp_dir().join(format!("vector-spa-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(
        dir.join("index.html"),
        "<!doctype html><title>VECTOR test</title>",
    )
    .unwrap();
    std::fs::write(dir.join("assets/app.js"), "console.log('VECTOR');").unwrap();
    let mut config = AppConfig::from_env();
    config.static_dir = dir.to_string_lossy().into_owned();
    let app = app_router(AppState::new(config));
    for (path, status, content_type, expected) in [
        ("/", StatusCode::OK, "text/html", "VECTOR test"),
        (
            "/schedule/day-one",
            StatusCode::OK,
            "text/html",
            "VECTOR test",
        ),
        (
            "/assets/app.js",
            StatusCode::OK,
            "javascript",
            "console.log",
        ),
        (
            "/api/missing",
            StatusCode::NOT_FOUND,
            "application/json",
            "API route not found",
        ),
        (
            "/api",
            StatusCode::NOT_FOUND,
            "application/json",
            "API route not found",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status, "{path}");
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .contains(content_type),
            "{path}"
        );
        let bytes = to_bytes(response.into_body(), 65536).await.unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains(expected), "{path}");
    }
    let missing = app
        .oneshot(
            Request::builder()
                .uri("/assets/missing.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    std::fs::remove_dir_all(dir).unwrap();
}

async fn json_response(response: reqwest::Response) -> Value {
    let status = response.status();
    let body: Value = response.json().await.unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[tokio::test]
async fn full_demo_flow_over_http() {
    let mut config = common::config();
    let (provider_url, provider_task, _) = common::mock_github().await;
    config.github_client_id = Some("test-client".into());
    config.github_client_secret = Some("test-secret".into());
    config.github_api_base_url = provider_url.clone();
    config.github_token_url = format!("{provider_url}/token");
    // Exercise the deterministic AI fallback without spending API credits.
    config.openai_api_key = None;
    config.public_base_url = "http://127.0.0.1".into();
    let state = AppState::new(config);
    state.elastic().ensure_indices_and_seed().await.unwrap();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let elastic = state.elastic().base_url.clone();
    let original: Value = client
        .get(format!("{elastic}/builders/_doc/kelseyhightower"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server =
        tokio::spawn(async move { axum::serve(listener, app_router(state)).await.unwrap() });

    let health = json_response(
        client
            .get(format!("{base}/api/health"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(health["status"], "ok");
    let trace = health["trace_id"].as_str().unwrap();
    assert_eq!(trace.len(), 32);
    assert!(trace.bytes().all(|c| c.is_ascii_hexdigit()));
    chrono::DateTime::parse_from_rfc3339(health["timestamp_utc"].as_str().unwrap()).unwrap();

    let start = client
        .get(format!("{base}/api/auth/github"))
        .send()
        .await
        .unwrap();
    assert_eq!(start.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = reqwest::Url::parse(start.headers()["location"].to_str().unwrap()).unwrap();
    let params: std::collections::HashMap<_, _> = location.query_pairs().into_owned().collect();
    assert_eq!(params["code_challenge_method"], "S256");
    assert_eq!(params["scope"], "read:user");
    let transaction_cookies = start
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap())
        .collect::<Vec<_>>()
        .join("; ");
    let response = client
        .get(format!("{base}/api/auth/github/callback"))
        .query(&[("code", "valid-code"), ("state", params["state"].as_str())])
        .header("cookie", transaction_cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    let cookie = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("vector_sid="))
        .unwrap()
        .to_owned();
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));
    let cookie = cookie.split(';').next().unwrap();
    let get = |path: &str| client.get(format!("{base}{path}")).header("cookie", cookie);
    let post = |path: &str| {
        client
            .post(format!("{base}{path}"))
            .header("cookie", cookie)
    };
    let hydrated = json_response(get("/api/state").send().await.unwrap()).await;
    assert_eq!(hydrated["profile"]["username"], "kelseyhightower");
    assert_eq!(hydrated["rooms"].as_array().unwrap().len(), 24);
    assert!(!hydrated["profile"]["dna_tags"]
        .as_array()
        .unwrap()
        .is_empty());

    json_response(
        post("/api/surge/reset")
            .json(&json!({}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    let surge = json_response(
        post("/api/surge")
            .json(&json!({"room_id":"ballroom-b"}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(surge["room"]["occupancy_ratio"], 1.0);
    let alert = &surge["surge"]["data"];
    let alternative = &alert["recommended_alternative"];
    // Ranking varies with live GitHub DNA; verify a valid open, same-slot alternative.
    assert!(alternative["room_id"].is_string());
    assert_ne!(alternative["room_id"], "ballroom-b");
    assert!(alternative["occupancy_ratio"].as_f64().unwrap() < 0.98);
    assert_eq!(alternative["is_rerouted_alternative"], true);
    assert_eq!(alert["bumped_hallway_peers"].as_array().unwrap().len(), 2);

    let sensory = json_response(
        client
            .patch(format!("{base}/api/sensory"))
            .header("cookie", cookie)
            .json(&json!({"sensory_mode_enabled":true}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(sensory["recommended_haven"]["room_id"], "room-204b-quiet");
    assert!(
        sensory["recommended_haven"]["decibel_level"]
            .as_f64()
            .unwrap()
            < 45.0
    );
    assert!(sensory["recommended_haven"]["active_talk_id"].is_null());

    let connection = json_response(
        post("/api/handshake/connect")
            .json(&json!({"peer_username":"zircon-dev", "room_id":"ballroom-b"}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    let record = &connection["handshake"];
    for key in [
        "draft_linkedin_dm",
        "draft_email_subject",
        "draft_email_body",
    ] {
        assert!(!record[key].as_str().unwrap().is_empty());
    }
    let id = record["handshake_id"].as_str().unwrap();
    let stored = json_response(
        client
            .get(format!("{elastic}/handshakes/_doc/{id}"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(stored["_source"], *record);
    let reset = json_response(
        post("/api/surge/reset")
            .json(&json!({}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(reset["room"]["capacity_current"], 142);
    assert_eq!(reset["room"]["decibel_level"], 68.0);

    client
        .delete(format!("{elastic}/handshakes/_doc/{id}?refresh=true"))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    client
        .put(format!(
            "{elastic}/builders/_doc/kelseyhightower?refresh=true"
        ))
        .json(&original["_source"])
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    server.abort();
    provider_task.abort();
}
