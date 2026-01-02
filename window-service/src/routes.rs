use axum::{Router, routing::{get, post}, response::{Html, IntoResponse}, extract::State, http::StatusCode, Json};
use axum::extract::ws::{WebSocketUpgrade, Message as AxumMessage};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use futures_util::{SinkExt, StreamExt};

use crate::state::AppState;
use crate::service;
use crate::ui;
use crate::secrets;
use crate::config;
use crate::ai;
use crate::sessions;
#[cfg(target_os = "windows")]
use scrap::{Capturer, Display};
#[cfg(target_os = "windows")]
use image::{ImageBuffer, Rgba};
use base64::engine::general_purpose;
use base64::Engine as _;

#[derive(serde::Serialize)]
pub struct StatusPayload { pub running: bool }

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(startup_page))
        .route("/startup", get(startup_page))
        .route("/home", get(landing))
        .route("/app", get(app_page))
        .route("/mobile", get(mobile_page))
        .route("/api/start", post(start))
        .route("/api/stop", post(stop))
        .route("/api/status", get(status))
        .route("/api/status-hi", get(status_hi))
        .route("/api/vad-status", get(vad_status))
        .route("/api/vad-status-hi", get(vad_status_hi))
        .route("/ws/transcript", get(ws_transcript))
        .route("/api/start-hi", post(start_hi))
        .route("/api/stop-hi", post(stop_hi))
        .route("/ws/transcript-hi", get(ws_transcript_hi))
        // Secret management endpoints (keyring)
        .route("/api/providers/:name/key", post(save_provider_key))
        .route("/api/providers/:name/key", axum::routing::delete(delete_provider_key))
        .route("/api/providers/state", get(providers_state))
        // Settings config (non-secret)
        .route("/api/settings", get(get_settings))
.route("/api/settings/providers", post(save_providers_cfg))
        .route("/api/settings/en", post(save_en_settings))
        .route("/api/settings/hi", post(save_hi_settings))
        .route("/api/settings/sc", post(save_sc_settings))
        .route("/api/settings/providers-sc", post(save_sc_providers_cfg))
        .route("/api/settings/fallback", post(save_fallback_settings))
        .route("/api/settings/sc-fallback", post(save_sc_fallback))
        // Remote LLM endpoints (independent)
        .route("/api/remote/select", post(remote_select))
        .route("/api/remote/select", axum::routing::delete(remote_clear))
        .route("/api/remote/status", get(remote_status))
        .route("/api/remote/test-auth", post(remote_test_auth))
        .route("/api/remote/config", get(get_remote_cfg))
        .route("/api/remote/config", post(save_remote_cfg))
        .route("/api/remote/key", post(save_remote_key))
        .route("/api/remote/key", axum::routing::delete(delete_remote_key))
        .route("/api/remote/workspace", get(get_remote_workspace))
        .route("/api/remote/workspaces", get(remote_list_workspaces))
        .route("/api/remote/ask", post(remote_ask))
        // AI endpoints (English)
        .route("/api/ai/en", post(ai_en_auto))
        .route("/api/ai/en/groq", post(ai_en_groq))
        .route("/api/ai/en/gemini", post(ai_en_gemini))
        .route("/api/ai/en/openrouter", post(ai_en_openrouter))
        // AI endpoints (Hindi)
        .route("/api/ai/hi", post(ai_hi_auto))
        .route("/api/ai/hi/gemini", post(ai_hi_gemini))
        .route("/api/ai/hi/openrouter", post(ai_hi_openrouter))
        // AI endpoint (Screen Capture)
        .route("/api/ai/sc", post(ai_sc_auto))
        .route("/api/capture", post(capture_screen_now))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
        .layer(CorsLayer::very_permissive())
}

pub async fn get_remote_cfg(State(state): State<AppState>) -> impl IntoResponse {
    let has_key = crate::secrets::has_key("anythingllm");
    let slug = crate::secrets::get_key("anythingllm_workspace");
    let s = state.settings.lock().await.remote.clone();
    Json(RemoteCfgResp { has_key, slug, chat_default: s.chat_default, stream_default: s.stream_default, chat_mode: s.chat_mode })
}

pub async fn save_remote_cfg(State(state): State<AppState>, Json(req): Json<RemoteCfgReq>) -> impl IntoResponse {
    // slug in keyring as requested earlier
    if let Some(slug) = req.slug.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let _ = crate::secrets::save_key("anythingllm_workspace", slug);
    }
    if let Some(false) = req.slug.as_ref().map(|s| s.trim().is_empty()) { /* ignore */ }
    
    let mut guard = state.settings.lock().await;
    if let Some(flag) = req.chat_default { guard.remote.chat_default = flag; }
    if let Some(flag) = req.stream_default { guard.remote.stream_default = flag; }
    if let Some(mode) = req.chat_mode.as_ref() {
        let m = mode.trim().to_lowercase();
        if m == "chat" || m == "query" { guard.remote.chat_mode = m; }
    }
    let s = guard.clone();
    drop(guard);
    
    match config::save_to(&state.settings_path, &s) {
        Ok(_) => axum::http::StatusCode::OK,
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(serde::Deserialize)]
struct KeyPayloadRemote { api_key: String }
pub async fn save_remote_key(Json(payload): Json<KeyPayloadRemote>) -> impl IntoResponse {
    match crate::secrets::save_key("anythingllm", &payload.api_key) {
        Ok(_) => axum::http::StatusCode::OK,
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}
pub async fn delete_remote_key() -> impl IntoResponse {
    match crate::secrets::delete_key("anythingllm") {
        Ok(_) => axum::http::StatusCode::OK,
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(serde::Serialize)]
struct RemoteWorkspaceResp { slug: String, name: Option<String> }
pub async fn get_remote_workspace(State(state): State<AppState>) -> impl IntoResponse {
    let url = match state.remote_selected.lock().await.as_ref().map(|s| s.url.clone()) { Some(u) => u, None => return (StatusCode::BAD_REQUEST, "no remote selected").into_response() };
    let slug = match crate::secrets::get_key("anythingllm_workspace") { Some(s) => s, None => return (StatusCode::BAD_REQUEST, "no slug").into_response() };
    let key = match crate::secrets::get_key("anythingllm") { Some(k) => k, None => return (StatusCode::UNAUTHORIZED, "no api key").into_response() };
    let endpoint = format!("{}/api/v1/workspace/{}", url.trim_end_matches('/'), slug);
    let timeout = state.settings.lock().await.timeouts.workspace_secs;
    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(timeout)).build() { Ok(c) => c, Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "client").into_response() };
    match client.get(&endpoint).header("Authorization", format!("Bearer {}", key)).send().await {
        Ok(resp) if resp.status().is_success() => {
            let name = resp.json::<serde_json::Value>().await.ok()
                .and_then(|v| v.get("workspace").and_then(|w| w.get("name")).and_then(|n| n.as_str()).map(|s| s.to_string()));
            Json(RemoteWorkspaceResp { slug, name }).into_response()
        }
        Ok(resp) if resp.status() == StatusCode::NOT_FOUND => (StatusCode::NOT_FOUND, "workspace not found").into_response(),
        Ok(resp) if resp.status() == StatusCode::UNAUTHORIZED => (StatusCode::UNAUTHORIZED, "invalid api key").into_response(),
        _ => (StatusCode::BAD_GATEWAY, "remote unreachable").into_response(),
    }
}

#[derive(serde::Serialize)]
struct RemoteWorkspacesResp { items: Vec<RemoteWorkspaceResp> }
pub async fn remote_list_workspaces(State(state): State<AppState>) -> impl IntoResponse {
    let url = match state.remote_selected.lock().await.as_ref().map(|s| s.url.clone()) { Some(u) => u, None => return (StatusCode::BAD_REQUEST, "no remote selected").into_response() };
    let key = match crate::secrets::get_key("anythingllm") { Some(k) => k, None => return (StatusCode::UNAUTHORIZED, "no api key").into_response() };
    let endpoint = format!("{}/api/v1/workspaces", url.trim_end_matches('/'));
    let timeout = state.settings.lock().await.timeouts.workspace_secs;
    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(timeout)).build() { Ok(c) => c, Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "client").into_response() };
    match client.get(&endpoint).header("Authorization", format!("Bearer {}", key)).send().await {
        Ok(resp) if resp.status().is_success() => {
            let v: serde_json::Value = match resp.json().await { Ok(j)=>j, Err(_)=>serde_json::json!({}) };
            let mut items = Vec::new();
            if let Some(arr) = v.get("workspaces").and_then(|x| x.as_array()) {
                for w in arr {
                    let slug = w.get("slug").and_then(|s| s.as_str()).unwrap_or_default().to_string();
                    let name = w.get("name").and_then(|s| s.as_str()).map(|s| s.to_string());
                    if !slug.is_empty() { items.push(RemoteWorkspaceResp { slug, name }); }
                }
            }
            Json(RemoteWorkspacesResp { items }).into_response()
        }
        Ok(resp) if resp.status() == StatusCode::UNAUTHORIZED => (StatusCode::UNAUTHORIZED, "invalid api key").into_response(),
        _ => (StatusCode::BAD_GATEWAY, "remote unreachable").into_response(),
    }
}

// ===== Remote Ask (AnythingLLM) =====
#[derive(serde::Deserialize)]
struct RemoteAskReq { input: String, stream: Option<bool>, mode: Option<String> }
#[derive(serde::Serialize)]
struct RemoteAskResp { output: String, provider: String, info: serde_json::Value }

pub async fn remote_ask(State(state): State<AppState>, Json(req): Json<RemoteAskReq>) -> impl IntoResponse {
    // Validate selection
    let base = match state.remote_selected.lock().await.as_ref().map(|s| s.url.clone()) { Some(u)=>u, None=> return (StatusCode::BAD_REQUEST, "no remote selected").into_response() };
    let key = match crate::secrets::get_key("anythingllm") { Some(k)=>k, None=> return (StatusCode::UNAUTHORIZED, "no api key").into_response() };
    let slug = match crate::secrets::get_key("anythingllm_workspace") { Some(s)=>s, None=> return (StatusCode::BAD_REQUEST, "no slug").into_response() };
    let stream = req.stream.unwrap_or(false);
    let mode = req.mode.unwrap_or_else(|| "chat".to_string());

    let timeout = state.settings.lock().await.timeouts.chat_secs;
    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(timeout)).build() { Ok(c)=>c, Err(_)=>return (StatusCode::INTERNAL_SERVER_ERROR, "client").into_response() };
    let api = format!("{}/api", base.trim_end_matches('/'));

    // Query mode: require sources first
    if mode.eq_ignore_ascii_case("query") {
        let vs_url = format!("{}/v1/workspace/{}/vector-search", api, slug);
        let payload = serde_json::json!({"query": req.input});
        match client.post(&vs_url).header("Authorization", format!("Bearer {}", key)).json(&payload).send().await {
            Ok(r) if r.status().is_success() => {
                let v: serde_json::Value = r.json().await.unwrap_or(serde_json::json!({}));
                let has_hits = v.get("results").and_then(|x| x.as_array()).map(|a| !a.is_empty()).unwrap_or(false);
                if !has_hits { return Json(RemoteAskResp { output: "No relevant sources found.".into(), provider: "anythingllm".into(), info: serde_json::json!({"mode":"query","stream":stream,"vector_hits":0}) }).into_response(); }
            }
            _ => return (StatusCode::BAD_GATEWAY, "vector search failed").into_response(),
        }
    }

    // Chat call
    let chat_path = if stream { "stream-chat" } else { "chat" };
    let chat_url = format!("{}/v1/workspace/{}/{}", api, slug, chat_path);
    let payload = serde_json::json!({"message": req.input});
    match client.post(&chat_url)
        .header("Authorization", format!("Bearer {}", key))
        .header("Accept", "application/json, text/event-stream")
        .json(&payload)
        .send().await {
        Ok(r) if r.status().is_success() => {
            if stream {
                // Parse SSE: lines of "data: {json}"
                let body = r.text().await.unwrap_or_default();
                let mut out = String::new();
                let mut sources: Option<serde_json::Value> = None;
                let mut finalize_text: Option<String> = None;
                let mut parse_errors = 0;
                
                for line in body.lines() {
                    let t = line.trim_start();
                    if let Some(rest) = t.strip_prefix("data:") {
                        let js = rest.trim();
                        if js.is_empty() || js == "[DONE]" { continue; }
                        
                        match serde_json::from_str::<serde_json::Value>(js) {
                            Ok(v) => {
                                // Try to extract text from various possible structures
                                match v.get("type").and_then(|x| x.as_str()) {
                                    Some("textResponseChunk") => {
                                        if let Some(s) = v.get("textResponse").and_then(|x| x.as_str()) { 
                                            out.push_str(s); 
                                        }
                                    }
                                    Some("finalizeResponseStream") => {
                                        if let Some(s) = v.get("sources").cloned() { sources = Some(s); }
                                        // Try multiple paths for final text
                                        finalize_text = v.get("response")
                                            .and_then(|r| r.get("textResponse").and_then(|x| x.as_str()).map(|s| s.to_string()))
                                            .or_else(|| v.get("response").and_then(|x| x.as_str()).map(|s| s.to_string()))
                                            .or_else(|| v.get("message").and_then(|x| x.as_str()).map(|s| s.to_string()));
                                    }
                                    _ => {
                                        // Unknown event type - log but don't fail
                                    }
                                }
                            }
                            Err(_) => {
                                parse_errors += 1;
                                // Skip malformed JSON but continue processing
                                if parse_errors > 50 { 
                                    // Too many errors, likely not SSE format
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // Use finalize text if available, otherwise use accumulated chunks
                if let Some(ft) = finalize_text { out = ft; }
                
                // If we got nothing, return error
                if out.is_empty() && parse_errors > 0 {
                    return (StatusCode::BAD_GATEWAY, "Failed to parse streaming response").into_response();
                }
                
                let info = if let Some(s) = sources { 
                    serde_json::json!({"mode":mode,"stream":true,"sources":s}) 
                } else { 
                    serde_json::json!({"mode":mode,"stream":true}) 
                };
                Json(RemoteAskResp { output: out, provider: "anythingllm".into(), info }).into_response()
            } else {
                let txt = r.text().await.unwrap_or_default();
                // If the server erroneously returns SSE in non-stream mode, parse it
                if txt.contains("\ndata:") || txt.starts_with("data:") {
                    let mut out = String::new();
                    for line in txt.lines() {
                        let t = line.trim_start();
                        if let Some(rest) = t.strip_prefix("data:") {
                            let js = rest.trim();
                            if js.is_empty() { continue; }
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(js) {
                                if v.get("type").and_then(|x| x.as_str()) == Some("textResponseChunk") {
                                    if let Some(s) = v.get("textResponse").and_then(|x| x.as_str()) { out.push_str(s); }
                                }
                            }
                        }
                    }
                    return Json(RemoteAskResp { output: out, provider: "anythingllm".into(), info: serde_json::json!({"mode":mode,"stream":false,"parsed":"sse"}) }).into_response();
                }
                // Otherwise parse JSON body (extract final text as String)
                let out = serde_json::from_str::<serde_json::Value>(&txt).ok()
                    .and_then(|j| {
                        j.get("textResponse").and_then(|x| x.as_str().map(|s| s.to_string()))
                        .or_else(|| j.get("response").and_then(|r| r.get("textResponse").and_then(|x| x.as_str().map(|s| s.to_string()))))
                        .or_else(|| j.get("response").and_then(|x| x.as_str().map(|s| s.to_string())))
                        .or_else(|| j.get("message").and_then(|x| x.as_str().map(|s| s.to_string())))
                        .or_else(|| j.get("data").and_then(|d| {
                            d.get("textResponse").and_then(|x| x.as_str().map(|s| s.to_string()))
                                .or_else(|| d.get("response").and_then(|r| r.get("textResponse").and_then(|x| x.as_str().map(|s| s.to_string()))))
                        }))
                    })
                    .unwrap_or_else(|| txt);
                Json(RemoteAskResp { output: out, provider: "anythingllm".into(), info: serde_json::json!({"mode":mode,"stream":false}) }).into_response()
            }
        }
        Ok(r) => (r.status(), r.text().await.unwrap_or_else(|_| "error".into())).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, "chat failed").into_response(),
    }
}

// Call helper to capture screen (works from Session 0)
async fn capture_via_helper() -> Result<String, String> {
    use std::time::Duration;
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Call helper on localhost:8081
    let response = client
        .get("http://127.0.0.1:8081/capture")
        .send()
        .await
        .map_err(|e| format!("Helper not responding. Make sure voxora-helper.exe is running: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Helper returned error: {}", response.status()));
    }
    
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse helper response: {}", e))?;
    
    data.get("image")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Helper response missing 'image' field".to_string())
}

pub async fn capture_screen_now(State(state): State<AppState>) -> impl IntoResponse {
    // Call helper to capture screen (helper runs in user session)
    match capture_via_helper().await {
        Ok(b64) => match ai::sc_analyze_image(&state, b64).await {
            Ok(res) => Json(AiResponse { output: res.output, provider: res.provider, model: res.model }).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, e).into_response(),
    }
}

async fn landing() -> impl IntoResponse { Html(ui::landing_html()) }
async fn app_page() -> impl IntoResponse { Html(ui::app_html()) }
async fn startup_page() -> impl IntoResponse { Html(crate::startup::startup_html()) }

async fn mobile_page() -> impl IntoResponse {
    // Compute LAN IPs
    let mut addrs: Vec<String> = Vec::new();
    if let Ok(ifaces) = get_if_addrs::get_if_addrs() {
        for iface in ifaces {
            if let std::net::IpAddr::V4(ipv4) = iface.ip() {
                if !ipv4.is_loopback() && !ipv4.is_link_local() { addrs.push(ipv4.to_string()); }
            }
        }
    }
    if addrs.is_empty() { addrs.push("127.0.0.1".to_string()); }
    let html = ui::mobile_html(&addrs, 8080);
    Html(html)
}

pub async fn start(State(state): State<AppState>) -> impl IntoResponse {
    match service::start_capture(state).await { Ok(_) => StatusCode::OK, Err(code) => code }
}

pub async fn stop(State(state): State<AppState>) -> impl IntoResponse {
    match service::stop_capture(state).await { Ok(_) => StatusCode::OK, Err(code) => code }
}

pub async fn start_hi(State(state): State<AppState>) -> impl IntoResponse {
    match service::start_capture_hi(state).await { Ok(_) => StatusCode::OK, Err(code) => code }
}

pub async fn stop_hi(State(state): State<AppState>) -> impl IntoResponse {
    match service::stop_capture_hi(state).await { Ok(_) => StatusCode::OK, Err(code) => code }
}

pub async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let running = state.session.lock().await.is_some();
    Json(StatusPayload { running })
}

pub async fn status_hi(State(state): State<AppState>) -> impl IntoResponse {
    let running = state.hindi_session.lock().await.is_some();
    Json(StatusPayload { running })
}

pub async fn ws_transcript(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (mut sink, _src) = socket.split();
        let mut rx = state.tx.subscribe();
        while let Ok(txt) = rx.recv().await {
            if sink.send(AxumMessage::Text(format!("EN: {}", txt))).await.is_err() { break; }
        }
    })
}

pub async fn ws_transcript_hi(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (mut sink, _src) = socket.split();
        let mut rx = state.hindi_tx.subscribe();
        while let Ok(txt) = rx.recv().await {
            if sink.send(AxumMessage::Text(format!("HI: {}", txt))).await.is_err() { break; }
        }
    })
}

#[derive(serde::Serialize)]
pub struct VadStatusPayload { sent: u64, skipped: u64, last_state: String }

pub async fn vad_status(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.english_vad.lock().await.clone();
    Json(VadStatusPayload { sent: stats.sent, skipped: stats.skipped, last_state: stats.last_state })
}

pub async fn vad_status_hi(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.hindi_vad.lock().await.clone();
    Json(VadStatusPayload { sent: stats.sent, skipped: stats.skipped, last_state: stats.last_state })
}

// ===== Provider keyring endpoints =====
#[derive(serde::Deserialize)]
struct KeyPayload { api_key: String }

pub async fn save_provider_key(axum::extract::Path(name): axum::extract::Path<String>, Json(payload): Json<KeyPayload>) -> impl IntoResponse {
    match secrets::save_key(&name, &payload.api_key) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn delete_provider_key(axum::extract::Path(name): axum::extract::Path<String>) -> impl IntoResponse {
    match secrets::delete_key(&name) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(serde::Serialize)]
struct ProviderState { has_key: bool }
#[derive(serde::Serialize)]
struct ProvidersState { groq: ProviderState, gemini: ProviderState, openrouter: ProviderState, custom: ProviderState }

pub async fn providers_state() -> impl IntoResponse {
    Json(ProvidersState {
        groq: ProviderState { has_key: secrets::has_key("groq") },
        gemini: ProviderState { has_key: secrets::has_key("gemini") },
        openrouter: ProviderState { has_key: secrets::has_key("openrouter") },
        custom: ProviderState { has_key: secrets::has_key("custom") },
    })
}

// ===== Settings file endpoints =====
pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let s = state.settings.lock().await.clone();
    Json(s)
}

#[derive(serde::Deserialize)]
pub struct ProvidersCfgPayload { groq: config::ProviderCfg, gemini: config::ProviderCfg, openrouter: config::ProviderCfg, groq_streaming: Option<bool> }
///pub struct ProvidersCfgPayload { groq: config::ProviderCfg, gemini: config::ProviderCfg, openrouter: config::ProviderCfg }
#[derive(serde::Deserialize)]
pub struct ProvidersCfgScPayload { groq: config::ProviderCfg, gemini: config::ProviderCfg, openrouter: config::ProviderCfg }
#[derive(serde::Deserialize)]
pub struct FallbackPayload { openrouter_choice: String }
#[derive(serde::Deserialize)]
pub struct ScFallbackPayload { or_fallback: bool, model: Option<String> }

pub async fn save_providers_cfg(State(state): State<AppState>, Json(payload): Json<ProvidersCfgPayload>) -> impl IntoResponse {
    let mut guard = state.settings.lock().await;
    guard.providers.groq = payload.groq;
    guard.providers.gemini = payload.gemini;
    guard.providers.openrouter = payload.openrouter;
    if let Some(b) = payload.groq_streaming { guard.groq_streaming = b; }
    let s = guard.clone();
    drop(guard);
    
    match config::save_to(&state.settings_path, &s) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(serde::Deserialize)]
pub struct LangPayload { provider: String, model: Option<String>, custom_model: Option<String>, prompt: String }

pub async fn save_en_settings(State(state): State<AppState>, Json(payload): Json<LangPayload>) -> impl IntoResponse {
    let mut guard = state.settings.lock().await;
    guard.en.provider = payload.provider;
    guard.en.model = payload.model;
    guard.en.custom_model = payload.custom_model;
    guard.en.prompt = payload.prompt;
    let s = guard.clone();
    drop(guard);
    
    // Reset prompt state when prompt changes
    crate::ai::reset_prompt_state(&state, "en").await;
    
    match config::save_to(&state.settings_path, &s) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn save_hi_settings(State(state): State<AppState>, Json(payload): Json<LangPayload>) -> impl IntoResponse {
    let mut guard = state.settings.lock().await;
    guard.hi.provider = payload.provider;
    guard.hi.model = payload.model;
    guard.hi.custom_model = payload.custom_model;
    guard.hi.prompt = payload.prompt;
    let s = guard.clone();
    drop(guard);
    
    // Reset prompt state when prompt changes
    crate::ai::reset_prompt_state(&state, "hi").await;
    
    match config::save_to(&state.settings_path, &s) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn save_sc_settings(State(state): State<AppState>, Json(payload): Json<LangPayload>) -> impl IntoResponse {
    let mut guard = state.settings.lock().await;
    guard.sc.provider = payload.provider;
    guard.sc.model = payload.model;
    guard.sc.custom_model = payload.custom_model;
    guard.sc.prompt = payload.prompt;
    let s = guard.clone();
    drop(guard);
    
    // Reset prompt state when prompt changes
    crate::ai::reset_prompt_state(&state, "sc").await;
    
    match config::save_to(&state.settings_path, &s) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn save_sc_providers_cfg(State(state): State<AppState>, Json(payload): Json<ProvidersCfgScPayload>) -> impl IntoResponse {
    let mut guard = state.settings.lock().await;
    guard.sc_providers.groq = payload.groq;
    guard.sc_providers.gemini = payload.gemini;
    guard.sc_providers.openrouter = payload.openrouter;
    let s = guard.clone();
    drop(guard);
    
    match config::save_to(&state.settings_path, &s) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn save_fallback_settings(State(state): State<AppState>, Json(payload): Json<FallbackPayload>) -> impl IntoResponse {
    let mut guard = state.settings.lock().await;
    guard.fallback.openrouter_choice = payload.openrouter_choice;
    let s = guard.clone();
    drop(guard);
    
    match config::save_to(&state.settings_path, &s) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn save_sc_fallback(State(state): State<AppState>, Json(payload): Json<ScFallbackPayload>) -> impl IntoResponse {
    let mut guard = state.settings.lock().await;
    guard.sc_fallback_or = payload.or_fallback;
    guard.sc_fallback_or_model = payload.model;
    let s = guard.clone();
    drop(guard);
    
    match config::save_to(&state.settings_path, &s) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR
    }
}

// ===== AI endpoints =====
#[derive(serde::Deserialize)]
struct AiRequest { input: String }
#[derive(serde::Serialize)]
struct AiResponse { output: String, provider: String, model: String }

// Helper to map AI errors to proper HTTP status codes
fn map_ai_error(e: anyhow::Error) -> (StatusCode, String) {
    let err_msg = e.to_string();
    if err_msg.contains("Missing") && err_msg.contains("API key") {
        (StatusCode::UNAUTHORIZED, err_msg)
    } else if err_msg.contains("No model") || err_msg.contains("Unsupported") {
        (StatusCode::BAD_REQUEST, err_msg)
    } else if err_msg.contains("http") {
        (StatusCode::BAD_GATEWAY, err_msg)
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, err_msg)
    }
}

pub async fn ai_en_auto(State(state): State<AppState>, Json(req): Json<AiRequest>) -> impl IntoResponse {
    // Save user's question to session FIRST
    if let Some(ref mgr) = state.session_manager {
        let _ = sessions::integration::save_transcript(
            mgr,
            req.input.clone(),
            "en",
        ).await;
    }
    
    match ai::chat_en_auto(&state, req.input).await {
        Ok(res) => {
            // Save AI response to session
            if let Some(ref mgr) = state.session_manager {
                let _ = sessions::integration::save_ai_response(
                    mgr,
                    res.output.clone(),
                    "en",
                    Some(res.provider.clone()),
                    Some(res.model.clone()),
                ).await;
            }
            Json(AiResponse { output: res.output, provider: res.provider, model: res.model }).into_response()
        }
        Err(e) => {
            let (status, msg) = map_ai_error(e);
            (status, msg).into_response()
        }
    }
}

pub async fn ai_en_groq(State(state): State<AppState>, Json(req): Json<AiRequest>) -> impl IntoResponse {
    match ai::chat_en_groq_direct(&state, req.input).await {
        Ok(res) => Json(AiResponse { output: res.output, provider: res.provider, model: res.model }).into_response(),
        Err(e) => {
            let (status, msg) = map_ai_error(e);
            (status, msg).into_response()
        }
    }
}
pub async fn ai_en_gemini(State(state): State<AppState>, Json(req): Json<AiRequest>) -> impl IntoResponse {
    match ai::chat_en_gemini_direct(&state, req.input).await {
        Ok(res) => Json(AiResponse { output: res.output, provider: res.provider, model: res.model }).into_response(),
        Err(e) => {
            let (status, msg) = map_ai_error(e);
            (status, msg).into_response()
        }
    }
}
pub async fn ai_en_openrouter(State(state): State<AppState>, Json(req): Json<AiRequest>) -> impl IntoResponse {
    match ai::chat_en_openrouter_direct(&state, req.input).await {
        Ok(res) => Json(AiResponse { output: res.output, provider: res.provider, model: res.model }).into_response(),
        Err(e) => {
            let (status, msg) = map_ai_error(e);
            (status, msg).into_response()
        }
    }
}

pub async fn ai_hi_gemini(State(state): State<AppState>, Json(req): Json<AiRequest>) -> impl IntoResponse {
    match ai::chat_hi_gemini_direct(&state, req.input).await {
        Ok(res) => Json(AiResponse { output: res.output, provider: res.provider, model: res.model }).into_response(),
        Err(e) => {
            let (status, msg) = map_ai_error(e);
            (status, msg).into_response()
        }
    }
}
pub async fn ai_hi_openrouter(State(state): State<AppState>, Json(req): Json<AiRequest>) -> impl IntoResponse {
    match ai::chat_hi_openrouter_direct(&state, req.input).await {
        Ok(res) => Json(AiResponse { output: res.output, provider: res.provider, model: res.model }).into_response(),
        Err(e) => {
            let (status, msg) = map_ai_error(e);
            (status, msg).into_response()
        }
    }
}

pub async fn ai_hi_auto(State(state): State<AppState>, Json(req): Json<AiRequest>) -> impl IntoResponse {
    // Save user's question to session FIRST
    if let Some(ref mgr) = state.session_manager {
        let _ = sessions::integration::save_transcript(
            mgr,
            req.input.clone(),
            "hi",
        ).await;
    }
    
    match ai::chat_hi_auto(&state, req.input).await {
        Ok(res) => {
            // Save AI response to session
            if let Some(ref mgr) = state.session_manager {
                let _ = sessions::integration::save_ai_response(
                    mgr,
                    res.output.clone(),
                    "hi",
                    Some(res.provider.clone()),
                    Some(res.model.clone()),
                ).await;
            }
            Json(AiResponse { output: res.output, provider: res.provider, model: res.model }).into_response()
        }
        Err(e) => {
            let (status, msg) = map_ai_error(e);
            (status, msg).into_response()
        }
    }
}

pub async fn ai_sc_auto(State(state): State<AppState>, Json(req): Json<AiRequest>) -> impl IntoResponse {
    match ai::chat_sc_auto(&state, req.input).await {
        Ok(res) => Json(AiResponse { output: res.output, provider: res.provider, model: res.model }).into_response(),
        Err(e) => {
            let (status, msg) = map_ai_error(e);
            (status, msg).into_response()
        }
    }
}

// ===== Remote LLM handlers =====
#[derive(serde::Deserialize)]
struct RemoteSelectReq { server: String, url: String }
#[derive(serde::Serialize)]
struct RemoteStatusResp { selected: Option<crate::startup::RemoteSelection>, online: bool }
#[derive(serde::Serialize)]
struct RemoteCfgResp { has_key: bool, slug: Option<String>, chat_default: bool, stream_default: bool, chat_mode: String }
#[derive(serde::Deserialize)]
struct RemoteCfgReq { slug: Option<String>, chat_default: Option<bool>, stream_default: Option<bool>, chat_mode: Option<String> }

pub async fn remote_select(State(state): State<AppState>, Json(req): Json<RemoteSelectReq>) -> impl IntoResponse {
    // basic normalize and health check
    let url = req.url.trim().to_string();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return (StatusCode::BAD_REQUEST, "invalid url").into_response();
    }
    let timeout = state.settings.lock().await.timeouts.health_check_secs;
    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(timeout)).build() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "client").into_response(),
    };
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let sel = crate::startup::RemoteSelection { server: req.server, url };
            {
                let mut g = state.remote_selected.lock().await;
                *g = Some(sel.clone());
            }
            (StatusCode::OK, axum::Json(sel)).into_response()
        }
        _ => (StatusCode::BAD_GATEWAY, "remote not reachable").into_response(),
    }
}

pub async fn remote_clear(State(state): State<AppState>) -> impl IntoResponse {
    let mut g = state.remote_selected.lock().await;
    *g = None;
    StatusCode::OK
}

pub async fn remote_status(State(state): State<AppState>) -> impl IntoResponse {
    let sel = { state.remote_selected.lock().await.clone() };
    let mut online = false;
    if let Some(s) = &sel {
        let timeout = state.settings.lock().await.timeouts.health_check_secs;
        if let Ok(client) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(timeout)).build() {
            if let Ok(resp) = client.get(&s.url).send().await { online = resp.status().is_success(); }
        }
    }
    Json(RemoteStatusResp { selected: sel, online })
}

#[derive(serde::Deserialize)]
struct RemoteTestReq { url: String, api_key: String }
#[derive(serde::Serialize)]
struct RemoteTestResp { ok: bool }

pub async fn remote_test_auth(State(state): State<AppState>, Json(req): Json<RemoteTestReq>) -> impl IntoResponse {
    let url = req.url.trim().trim_end_matches('/').to_string();
    let endpoint = format!("{}/api/v1/workspaces", url);
    let timeout = state.settings.lock().await.timeouts.health_check_secs;
    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(timeout)).build() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "client").into_response(),
    };
    let r = client.get(&endpoint).header("Authorization", format!("Bearer {}", req.api_key)).send().await;
    match r {
        Ok(resp) if resp.status().is_success() => {
            // Persist on success for personal use
            let _ = crate::secrets::save_key("anythingllm", &req.api_key);
            (StatusCode::OK, Json(RemoteTestResp { ok: true })).into_response()
        }
        Ok(resp) if resp.status() == StatusCode::UNAUTHORIZED => (StatusCode::UNAUTHORIZED, "invalid api key").into_response(),
        _ => (StatusCode::BAD_GATEWAY, "remote not reachable").into_response(),
    }
}
