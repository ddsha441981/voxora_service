mod common;

use axum::{body::Body, http::{Request, StatusCode}};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn settings_roundtrip_and_mutations() {
    let t = common::build_app().await;
    let app = t.app;

    // Initial GET /api/settings
    let res = app.clone().oneshot(Request::builder().uri("/api/settings").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Update English settings
    let body = json!({
        "provider": "groq",
        "model": "llama-3.1-8b-instant",
        "custom_model": null,
        "prompt": "Test EN prompt"
    }).to_string();
    let res = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/settings/en")
        .header("content-type", "application/json")
        .body(Body::from(body)).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Update Hindi settings
    let body = json!({
        "provider": "gemini",
        "model": "gemini-2.5-flash",
        "custom_model": null,
        "prompt": "Test HI prompt"
    }).to_string();
    let res = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/settings/hi")
        .header("content-type", "application/json")
        .body(Body::from(body)).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Update Screen Capture settings
    let body = json!({
        "provider": "gemini",
        "model": "gemini-2.5-flash",
        "custom_model": null,
        "prompt": "Test SC prompt"
    }).to_string();
    let res = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/settings/sc")
        .header("content-type", "application/json")
        .body(Body::from(body)).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Update providers config
    let body = json!({
        "groq": {"default_model":"llama-3.1-8b-instant","extra_models":""},
        "gemini": {"default_model":"gemini-2.5-flash","extra_models":""},
        "openrouter": {"default_model":"meta-llama/llama-3.1-70b","extra_models":"qwen/qwen2-7b-instruct"}
    }).to_string();
    let res = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/settings/providers")
        .header("content-type", "application/json")
        .body(Body::from(body)).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Update SC providers config
    let body = json!({
        "groq": {"default_model":"meta-llama/llama-4-scout-17b-16e-instruct","extra_models":""},
        "gemini": {"default_model":"gemini-2.5-flash","extra_models":""},
        "openrouter": {"default_model":"google/gemini-2.5-flash","extra_models":""}
    }).to_string();
    let res = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/settings/providers-sc")
        .header("content-type", "application/json")
        .body(Body::from(body)).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Fallback settings
    let body = json!({ "openrouter_choice": "openai" }).to_string();
    let res = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/settings/fallback")
        .header("content-type", "application/json")
        .body(Body::from(body)).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Screen-capture fallback
    let body = json!({ "or_fallback": true, "model": "google/gemini-2.5-flash" }).to_string();
    let res = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/settings/sc-fallback")
        .header("content-type", "application/json")
        .body(Body::from(body)).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
