use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use backend_api::{app_router, config::AppConfig, state::AppState};

#[tokio::test]
async fn test_health_check_endpoint() {
    let mut config = AppConfig::from_env();
    config.port = 8089;
    let state = AppState::new(config);
    let app = app_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
