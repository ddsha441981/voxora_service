mod common;

use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use serde_json::Value;

#[tokio::test]
async fn status_and_vad_defaults() {
    let t = common::build_app().await;
    let app = t.app;

    // /api/status
    let res = app.clone().oneshot(Request::builder().uri("/api/status").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["running"], false);

    // /api/status-hi
    let res = app.clone().oneshot(Request::builder().uri("/api/status-hi").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["running"], false);

    // /api/vad-status
    let res = app.clone().oneshot(Request::builder().uri("/api/vad-status").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["sent"], 0);
    assert_eq!(v["skipped"], 0);
    assert_eq!(v["last_state"], "idle");

    // /api/vad-status-hi
    let res = app.clone().oneshot(Request::builder().uri("/api/vad-status-hi").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["sent"], 0);
    assert_eq!(v["skipped"], 0);
    assert_eq!(v["last_state"], "idle");
}
