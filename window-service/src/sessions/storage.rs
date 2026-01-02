use std::path::PathBuf;
use std::sync::Arc;
use rusqlite::{Connection, params, OptionalExtension};
use tokio::sync::Mutex;
use chrono::Utc;
use uuid::Uuid;
use anyhow::{Result, Context};

use super::models::{Session, Message, MessageRole};

#[derive(Clone)]
pub struct SessionStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SessionStorage {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        // Ensure data directory exists
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)
                .context("Failed to create data directory")?;
        }

        let db_path = data_dir.join("sessions.db");
        let conn = Connection::open(&db_path)
            .context("Failed to open sessions database")?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        // Initialize schema
        storage.init_schema()?;

        Ok(storage)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.try_lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                language TEXT,
                provider TEXT,
                model TEXT,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session_id 
             ON messages(session_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_timestamp 
             ON messages(timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    pub async fn create_session(&self, title: String) -> Result<Session> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        let session = Session {
            id: id.clone(),
            title: title.clone(),
            created_at: now,
            updated_at: now,
            message_count: 0,
        };

        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO sessions (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, title, now, now],
        )?;

        Ok(session)
    }

    pub async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT s.id, s.title, s.created_at, s.updated_at, COUNT(m.id) as message_count
             FROM sessions s
             LEFT JOIN messages m ON s.id = m.session_id
             WHERE s.id = ?1
             GROUP BY s.id"
        )?;

        let session = stmt.query_row(params![id], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                message_count: row.get(4)?,
            })
        }).optional()?;

        Ok(session)
    }

    pub async fn list_sessions(&self, limit: i64, offset: i64) -> Result<Vec<Session>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT s.id, s.title, s.created_at, s.updated_at, COUNT(m.id) as message_count
             FROM sessions s
             LEFT JOIN messages m ON s.id = m.session_id
             GROUP BY s.id
             ORDER BY s.updated_at DESC
             LIMIT ?1 OFFSET ?2"
        )?;

        let sessions = stmt.query_map(params![limit, offset], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                message_count: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    pub async fn update_session_title(&self, id: &str, title: String) -> Result<()> {
        let now = Utc::now().timestamp();
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE sessions SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, now, id],
        )?;

        Ok(())
    }

    pub async fn delete_session(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        
        // Delete messages first (even though we have CASCADE)
        conn.execute("DELETE FROM messages WHERE session_id = ?1", params![id])?;
        
        // Delete session
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;

        Ok(())
    }

    pub async fn add_message(&self, message: Message) -> Result<()> {
        let now = Utc::now().timestamp();
        let conn = self.conn.lock().await;

        conn.execute(
            "INSERT INTO messages (id, session_id, timestamp, role, content, language, provider, model)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                message.id,
                message.session_id,
                message.timestamp,
                message.role.as_str(),
                message.content,
                message.language,
                message.provider,
                message.model,
            ],
        )?;

        // Update session's updated_at timestamp
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![now, message.session_id],
        )?;

        Ok(())
    }

    pub async fn get_messages(&self, session_id: &str, limit: i64, offset: i64) -> Result<Vec<Message>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT id, session_id, timestamp, role, content, language, provider, model
             FROM messages
             WHERE session_id = ?1
             ORDER BY timestamp ASC
             LIMIT ?2 OFFSET ?3"
        )?;

        let messages = stmt.query_map(params![session_id, limit, offset], |row| {
            Ok(Message {
                id: row.get(0)?,
                session_id: row.get(1)?,
                timestamp: row.get(2)?,
                role: MessageRole::from_str(&row.get::<_, String>(3)?),
                content: row.get(4)?,
                language: row.get(5)?,
                provider: row.get(6)?,
                model: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    pub async fn search_sessions(&self, query: &str, limit: i64) -> Result<Vec<Session>> {
        let conn = self.conn.lock().await;
        let search_term = format!("%{}%", query);
        
        let mut stmt = conn.prepare(
            "SELECT DISTINCT s.id, s.title, s.created_at, s.updated_at, COUNT(m.id) as message_count
             FROM sessions s
             LEFT JOIN messages m ON s.id = m.session_id
             WHERE s.title LIKE ?1 OR m.content LIKE ?1
             GROUP BY s.id
             ORDER BY s.updated_at DESC
             LIMIT ?2"
        )?;

        let sessions = stmt.query_map(params![search_term, limit], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                message_count: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    pub async fn get_message_count(&self, session_id: &str) -> Result<i32> {
        let conn = self.conn.lock().await;
        
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        Ok(count)
    }
}
