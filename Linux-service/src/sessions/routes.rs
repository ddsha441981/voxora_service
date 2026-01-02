use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, delete, put},
    extract::{State, Path, Query},
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use super::manager::SessionManager;
use super::models::{
    Session, SessionWithMessages, CreateSessionRequest, 
    AddMessageRequest, UpdateSessionRequest,
};

#[derive(Clone)]
pub struct SessionState {
    pub manager: Arc<SessionManager>,
}

impl SessionState {
    pub fn new(manager: SessionManager) -> Self {
        Self {
            manager: Arc::new(manager),
        }
    }
}

pub fn session_router() -> Router<SessionState> {
    Router::new()
        .route("/", get(list_sessions).post(create_session))
        .route("/:id", get(get_session).delete(delete_session).put(update_session))
        .route("/:id/messages", get(get_messages).post(add_message))
        .route("/current", get(get_current_session).post(set_current_session).delete(clear_current_session))
        .route("/search", get(search_sessions))
}

#[derive(Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 { 100 }

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_search_limit")]
    limit: i64,
}

fn default_search_limit() -> i64 { 50 }

// List all sessions
async fn list_sessions(
    State(state): State<SessionState>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    match state.manager.list_sessions(pagination.limit, pagination.offset).await {
        Ok(sessions) => Json(sessions).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Create new session
async fn create_session(
    State(state): State<SessionState>,
    Json(req): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    match state.manager.create_session(req.title).await {
        Ok(session) => (StatusCode::CREATED, Json(session)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Get session with messages
async fn get_session(
    State(state): State<SessionState>,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    match state.manager.get_session_with_messages(&id, pagination.limit, pagination.offset).await {
        Ok(Some(session)) => Json(session).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Session not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Update session title
async fn update_session(
    State(state): State<SessionState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl IntoResponse {
    match state.manager.update_session_title(&id, req.title).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) if e.to_string().contains("not found") => {
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Delete session
async fn delete_session(
    State(state): State<SessionState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.manager.delete_session(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Get messages for a session
async fn get_messages(
    State(state): State<SessionState>,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    match state.manager.get_messages(&id, pagination.limit, pagination.offset).await {
        Ok(messages) => Json(messages).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Add message to session
async fn add_message(
    State(state): State<SessionState>,
    Path(id): Path<String>,
    Json(req): Json<AddMessageRequest>,
) -> impl IntoResponse {
    match state.manager.add_message(
        &id,
        req.content,
        req.role,
        req.language,
        req.provider,
        req.model,
    ).await {
        Ok(message) => (StatusCode::CREATED, Json(message)).into_response(),
        Err(e) if e.to_string().contains("not found") => {
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Get current session
async fn get_current_session(
    State(state): State<SessionState>,
) -> impl IntoResponse {
    match state.manager.get_current_session().await {
        Ok(Some(session)) => Json(session).into_response(),
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct SetCurrentRequest {
    session_id: Option<String>,
}

// Set current session
async fn set_current_session(
    State(state): State<SessionState>,
    Json(req): Json<SetCurrentRequest>,
) -> impl IntoResponse {
    match state.manager.set_current_session(req.session_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) if e.to_string().contains("not found") => {
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Clear current session
async fn clear_current_session(
    State(state): State<SessionState>,
) -> impl IntoResponse {
    match state.manager.set_current_session(None).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => {
            tracing::error!("Failed to clear current session: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// Search sessions
async fn search_sessions(
    State(state): State<SessionState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    match state.manager.search_sessions(&query.q, query.limit).await {
        Ok(sessions) => Json(sessions).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
