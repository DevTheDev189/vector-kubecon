mod common;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use backend_api::{
    app_router,
    auth::{sign_session_cookie, verify_session_cookie, SessionToken},
    AppState,
};
use chrono::Utc;
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn protected_routes_reject_anonymous_headers_expired_and_legacy_cookies() {
    let config = common::config();
    let mut expired = SessionToken::new("kelseyhightower".into(), 110292);
    expired.exp = Utc::now().timestamp() - 1;
    let mut legacy = SessionToken::new("kelseyhightower".into(), 110292);
    legacy.auth_version = 0;
    for token in [&expired, &legacy] {
        assert!(verify_session_cookie(
            sign_session_cookie(token, &config.session_secret, false).value(),
            &config.session_secret
        )
        .is_none());
    }
    let app = app_router(AppState::new(config.clone()));
    for cookie in [
        String::new(),
        format!(
            "vector_sid={}",
            sign_session_cookie(&expired, &config.session_secret, false).value()
        ),
        format!(
            "vector_sid={}",
            sign_session_cookie(&legacy, &config.session_secret, false).value()
        ),
    ] {
        for (method, path) in [
            ("GET", "/api/state"),
            ("GET", "/api/me"),
            ("GET", "/api/rooms"),
            ("GET", "/api/events/stream"),
            ("GET", "/api/handshake/badge"),
            ("POST", "/api/sync-github"),
            ("POST", "/api/surge"),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .header("x-vector-user", "kelseyhightower")
                        .header("cookie", &cookie)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "{method} {path}"
            );
        }
    }
    let removed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/impersonate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(removed.status(), StatusCode::NOT_FOUND);
    let mut missing = config.clone();
    missing.github_client_id = None;
    missing.github_client_secret = None;
    let response = app_router(AppState::new(missing))
        .oneshot(
            Request::builder()
                .uri("/api/auth/github")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let ordinary = common::cookie(&config, "attendee", 99);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/surge")
                .header("cookie", &ordinary)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .header("cookie", common::cookie(&config, "kelseyhightower", 110292))
                .header("origin", "https://untrusted.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/sync-github")
                .header("cookie", ordinary)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"username":"kelseyhightower"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn oauth_requires_matching_state_and_pkce_before_contacting_github() {
    let (url, provider, calls) = common::mock_github().await;
    let mut config = common::config();
    config.github_client_id = Some("test-client".into());
    config.github_client_secret = Some("test-secret".into());
    config.github_api_base_url = url.clone();
    config.github_token_url = format!("{url}/token");
    let app = app_router(AppState::new(config));
    let start = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/github")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(start.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = reqwest::Url::parse(start.headers()["location"].to_str().unwrap()).unwrap();
    let params: std::collections::HashMap<_, _> = location.query_pairs().into_owned().collect();
    assert_eq!(params["scope"], "read:user");
    assert_eq!(params["code_challenge_method"], "S256");
    let cookies: Vec<_> = start
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_owned())
        .collect();
    let verifier = cookies
        .iter()
        .find_map(|v| v.strip_prefix("vector_oauth_verifier="))
        .unwrap();
    use base64::Engine;
    use sha2::{Digest, Sha256};
    assert_eq!(
        params["code_challenge"],
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(Sha256::digest(verifier.as_bytes()))
    );
    for (state, cookie) in [
        ("wrong".to_string(), cookies.join("; ")),
        (params["state"].clone(), String::new()),
        (
            params["state"].clone(),
            format!("vector_oauth_state={}", params["state"]),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/auth/github/callback?code=valid-code&state={state}"
                    ))
                    .header("cookie", cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response
            .headers()
            .get_all("set-cookie")
            .iter()
            .all(|v| !v.to_str().unwrap().starts_with("vector_sid=")));
        assert!(response
            .headers()
            .get_all("set-cookie")
            .iter()
            .any(|v| v.to_str().unwrap().starts_with("vector_oauth_state=;")));
    }
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    let failed = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/auth/github/callback?code=invalid-code&state={}",
                    params["state"]
                ))
                .header("cookie", cookies.join("; "))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(failed.status(), StatusCode::BAD_GATEWAY);
    let bytes = to_bytes(failed.into_body(), 65536).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(!body.to_string().contains("test-secret"));
    provider.abort();
}
