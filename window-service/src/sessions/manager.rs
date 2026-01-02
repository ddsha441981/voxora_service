use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use uuid::Uuid;
use anyhow::{Result, bail};

use super::storage::SessionStorage;
use super::models::{Session, Message, MessageRole, SessionWithMessages};

#[derive(Clone)]
pub struct SessionManager {
    storage: SessionStorage,
    current_session_id: Arc<RwLock<Option<String>>>,
}

impl SessionManager {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let storage = SessionStorage::new(data_dir)?;
        
        Ok(Self {
            storage,
            current_session_id: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn create_session(&self, title: Option<String>) -> Result<Session> {
        let title = title.unwrap_or_else(|| format!("New Session"));
        let session = self.storage.create_session(title).await?;
        
        // Auto-set as current session if none exists
        let mut current = self.current_session_id.write().await;
        if current.is_none() {
            *current = Some(session.id.clone());
        }
        
        Ok(session)
    }

    pub async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        self.storage.get_session(id).await
    }

    pub async fn get_session_with_messages(&self, id: &str, limit: i64, offset: i64) -> Result<Option<SessionWithMessages>> {
        let session = match self.storage.get_session(id).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let messages = self.storage.get_messages(id, limit, offset).await?;

        Ok(Some(SessionWithMessages {
            session,
            messages,
        }))
    }

    pub async fn list_sessions(&self, limit: i64, offset: i64) -> Result<Vec<Session>> {
        self.storage.list_sessions(limit, offset).await
    }

    pub async fn update_session_title(&self, id: &str, title: String) -> Result<()> {
        // Verify session exists
        if self.storage.get_session(id).await?.is_none() {
            bail!("Session not found");
        }
        
        self.storage.update_session_title(id, title).await
    }

    pub async fn delete_session(&self, id: &str) -> Result<()> {
        // If deleting current session, clear it
        let mut current = self.current_session_id.write().await;
        if current.as_deref() == Some(id) {
            *current = None;
        }
        
        self.storage.delete_session(id).await
    }

    pub async fn add_message(
        &self,
        session_id: &str,
        content: String,
        role: MessageRole,
        language: Option<String>,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<Message> {
        // Verify session exists
        if self.storage.get_session(session_id).await?.is_none() {
            bail!("Session not found");
        }

        let message = Message {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            timestamp: Utc::now().timestamp(),
            role,
            content,
            language,
            provider,
            model,
        };

        self.storage.add_message(message.clone()).await?;
        
        Ok(message)
    }

    pub async fn add_message_to_current(
        &self,
        content: String,
        role: MessageRole,
        language: Option<String>,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<Message> {
        let current_id = self.current_session_id.read().await.clone();
        
        let session_id = match current_id {
            Some(id) => id,
            None => {
                // Auto-create a session if none exists
                let session = self.create_session(None).await?;
                session.id
            }
        };

        self.add_message(&session_id, content, role, language, provider, model).await
    }

    pub async fn get_messages(&self, session_id: &str, limit: i64, offset: i64) -> Result<Vec<Message>> {
        self.storage.get_messages(session_id, limit, offset).await
    }

    pub async fn search_sessions(&self, query: &str, limit: i64) -> Result<Vec<Session>> {
        self.storage.search_sessions(query, limit).await
    }

    pub async fn get_current_session_id(&self) -> Option<String> {
        self.current_session_id.read().await.clone()
    }

    pub async fn set_current_session(&self, id: Option<String>) -> Result<()> {
        // If setting a specific session, verify it exists
        if let Some(ref session_id) = id {
            if self.storage.get_session(session_id).await?.is_none() {
                bail!("Session not found");
            }
        }
        
        let mut current = self.current_session_id.write().await;
        *current = id;
        
        Ok(())
    }

    pub async fn get_current_session(&self) -> Result<Option<Session>> {
        let current_id = self.current_session_id.read().await.clone();
        
        match current_id {
            Some(id) => self.storage.get_session(&id).await,
            None => Ok(None),
        }
    }

    pub async fn ensure_current_session(&self) -> Result<Session> {
        match self.get_current_session().await? {
            Some(session) => Ok(session),
            None => {
                // Create a new session and set it as current
                let session = self.create_session(None).await?;
                Ok(session)
            }
        }
    }
}
