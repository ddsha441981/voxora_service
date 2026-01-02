use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub timestamp: i64,
    pub role: MessageRole,
    pub content: String,
    pub language: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

impl MessageRole {
    pub fn as_str(&self) -> &str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "assistant" => MessageRole::Assistant,
            _ => MessageRole::User,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWithMessages {
    #[serde(flatten)]
    pub session: Session,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMessageRequest {
    pub content: String,
    pub role: MessageRole,
    pub language: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSessionRequest {
    pub title: String,
}
