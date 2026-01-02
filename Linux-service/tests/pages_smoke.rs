mod common;

use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn pages_smoke() {
    let t = common::build_app().await;
    let app = t.app;

    // GET /
    let res = app.clone().oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // GET /app
    let res = app.clone().oneshot(Request::builder().uri("/app").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // GET /mobile
    let res = app.clone().oneshot(Request::builder().uri("/mobile").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // GET /startup
    let res = app.clone().oneshot(Request::builder().uri("/startup").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
