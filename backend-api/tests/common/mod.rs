#![allow(dead_code)]
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Json, Router,
};
use backend_api::{
    auth::{sign_session_cookie, SessionToken},
    AppConfig,
};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

pub fn config() -> AppConfig {
    let mut config = AppConfig::from_env();
    config.session_secret = b"vector-integration-test-only-signing-secret-32bytes".to_vec();
    config.openai_api_key = None;
    config.public_base_url = "http://127.0.0.1:8080".into();
    config.admin_users = vec!["kelseyhightower".into(), "zircon-dev".into()];
    config
}

// Tests sign fixtures directly. The production app exposes no fixture-login endpoint.
pub fn cookie(config: &AppConfig, username: &str, id: u64) -> String {
    format!(
        "vector_sid={}",
        sign_session_cookie(
            &SessionToken::new(username.into(), id),
            &config.session_secret,
            config.is_secure()
        )
        .value()
    )
}

pub async fn mock_github() -> (String, tokio::task::JoinHandle<()>, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    async fn exchange(
        State(calls): State<Arc<AtomicUsize>>,
        Form(form): Form<HashMap<String, String>>,
    ) -> Response {
        calls.fetch_add(1, Ordering::SeqCst);
        if form.get("code").map(String::as_str) != Some("valid-code")
            || form.get("code_verifier").is_none_or(|v| v.len() < 43)
            || !form.contains_key("redirect_uri")
        {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error":"bad_verification_code"})),
            )
                .into_response();
        }
        Json(json!({"access_token":"provider-test-token"})).into_response()
    }
    async fn identity(State(calls): State<Arc<AtomicUsize>>, headers: HeaderMap) -> Response {
        calls.fetch_add(1, Ordering::SeqCst);
        if headers.get("authorization").and_then(|v| v.to_str().ok())
            != Some("Bearer provider-test-token")
        {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        Json(json!({"id":110292,"login":"kelseyhightower","name":"Kelsey Hightower"}))
            .into_response()
    }
    let app = Router::new()
        .route("/token", post(exchange))
        .route("/user", get(identity))
        .route(
            "/users/kelseyhightower",
            get(|| async {
                Json(json!({"id":110292,"login":"kelseyhightower","name":"Kelsey Hightower"}))
            }),
        )
        .route(
            "/users/kelseyhightower/repos",
            get(|| async { Json(json!([])) }),
        )
        .with_state(calls.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (url, task, calls)
}
