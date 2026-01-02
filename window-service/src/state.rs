use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use crate::session::CaptureSession;

#[derive(Default, Clone, Debug)]
pub struct VadStats {
    pub sent: u64,
    pub skipped: u64,
    pub last_state: String, // "speech" | "silence" | "idle"
}

#[derive(Clone)]
pub struct AppState {
    // English pipeline
    pub session: Arc<Mutex<Option<CaptureSession>>>,
    pub tx: broadcast::Sender<String>,
    pub go_server_url: String,
    pub english_vad: Arc<Mutex<VadStats>>,
    // Hindi pipeline
    pub hindi_session: Arc<Mutex<Option<CaptureSession>>>,
    pub hindi_tx: broadcast::Sender<String>,
    pub hindi_go_server_url: String,
    pub hindi_vad: Arc<Mutex<VadStats>>,
    // Settings
    pub settings: Arc<Mutex<crate::config::Settings>>, 
    pub settings_path: std::path::PathBuf,
    // Prompt state (send system prompt only on first successful request per language)
    pub prompt_sent_en: Arc<Mutex<bool>>,
    pub prompt_sent_hi: Arc<Mutex<bool>>,
    pub prompt_sent_sc: Arc<Mutex<bool>>,
    // Remote LLM selection (independent of EN/HI pipeline)
    pub remote_selected: Arc<Mutex<Option<crate::startup::RemoteSelection>>>,
    // Session manager for history
    pub session_manager: Option<Arc<crate::sessions::SessionManager>>,
}

impl AppState {
    pub fn new(tx: broadcast::Sender<String>, go_server_url: String, settings: crate::config::Settings, settings_path: std::path::PathBuf) -> Self {
        let (hindi_tx, _rx) = broadcast::channel::<String>(100);
        let hindi_go_server_url = std::env::var("HINDI_GO_SERVER_URL").unwrap_or_else(|_| "ws://127.0.0.1:8086/ws".to_string());
        Self {
            session: Arc::new(Mutex::new(None)),
            tx,
            go_server_url,
            english_vad: Arc::new(Mutex::new(VadStats { sent: 0, skipped: 0, last_state: "idle".to_string() })),
            hindi_session: Arc::new(Mutex::new(None)),
            hindi_tx,
            hindi_go_server_url,
            hindi_vad: Arc::new(Mutex::new(VadStats { sent: 0, skipped: 0, last_state: "idle".to_string() })),
            settings: Arc::new(Mutex::new(settings)),
            settings_path,
            prompt_sent_en: Arc::new(Mutex::new(false)),
            prompt_sent_hi: Arc::new(Mutex::new(false)),
            prompt_sent_sc: Arc::new(Mutex::new(false)),
            remote_selected: Arc::new(Mutex::new(None)),
            session_manager: None, // Will be set after creation
        }
    }
    
    pub fn set_session_manager(&mut self, manager: Arc<crate::sessions::SessionManager>) {
        self.session_manager = Some(manager);
    }
}
