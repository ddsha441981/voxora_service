use std::sync::Arc;
use tokio::sync::broadcast;
use chrono::Utc;
use uuid::Uuid;

use super::manager::SessionManager;
use super::models::{Message, MessageRole};

/// Helper to save transcript to current session
pub async fn save_transcript(
    manager: &Arc<SessionManager>,
    content: String,
    language: &str,
) -> anyhow::Result<()> {
    if content.trim().is_empty() {
        return Ok(());
    }

    manager.add_message_to_current(
        content,
        MessageRole::User,
        Some(language.to_string()),
        None,
        None,
    ).await?;

    Ok(())
}

/// Helper to save AI response to current session
pub async fn save_ai_response(
    manager: &Arc<SessionManager>,
    content: String,
    language: &str,
    provider: Option<String>,
    model: Option<String>,
) -> anyhow::Result<()> {
    if content.trim().is_empty() {
        return Ok(());
    }

    manager.add_message_to_current(
        content,
        MessageRole::Assistant,
        Some(language.to_string()),
        provider,
        model,
    ).await?;

    Ok(())
}

/// Start background task that listens to transcript broadcasts and saves them
pub fn start_transcript_listener(
    manager: Arc<SessionManager>,
    mut rx: broadcast::Receiver<String>,
    language: &'static str,
) {
    tokio::spawn(async move {
        while let Ok(transcript) = rx.recv().await {
            // Clean up transcript (remove language prefix if present)
            let clean_text = transcript
                .trim()
                .trim_start_matches("EN:")
                .trim_start_matches("HI:")
                .trim()
                .trim_matches('"')
                .to_string();

            if !clean_text.is_empty() {
                if let Err(e) = save_transcript(&manager, clean_text, language).await {
                    tracing::warn!("Failed to save {} transcript to session: {}", language, e);
                }
            }
        }
    });
}
