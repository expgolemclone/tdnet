use std::collections::HashSet;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tdnet_viewer::cli::Period;
use tdnet_viewer::http_client::TdnetClient;
use tdnet_viewer::server::{AppState, router};
use tower::ServiceExt;

fn test_app() -> axum::Router {
    let client = TdnetClient::new().unwrap();
    let state = AppState::new(
        client,
        "20260325".to_string(),
        Period::Day,
        None,
        HashSet::new(),
    );
    router(state)
}

#[tokio::test]
async fn api_init_returns_initial_params() {
    let response = test_app()
        .oneshot(
            Request::builder()
                .uri("/api/init")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn api_list_rejects_missing_date() {
    let response = test_app()
        .oneshot(
            Request::builder()
                .uri("/api/list")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn api_list_rejects_invalid_period() {
    let response = test_app()
        .oneshot(
            Request::builder()
                .uri("/api/list?date=20260325&period=bad")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
