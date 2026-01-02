use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, filter::Targets};

mod state;
mod session;
mod sessions;
mod ui;
mod service;
mod routes;
mod vad;
mod secrets;
mod config;
mod ai;
mod startup;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logs directory anchored at project root
    let exe = std::env::current_exe()?;
    let exe_dir = exe.parent().ok_or(anyhow::anyhow!("no exe dir"))?;
    // Check if we're in target/debug or target/release (dev mode) or standalone dist folder (release mode)
    let project_root = if exe_dir.join("bin").exists() {
        // Portable/release mode: bin/ is adjacent to exe
        exe_dir
    } else {
        // Dev mode: target/debug/<exe> => project root two levels up
        exe_dir.parent().and_then(|p| p.parent()).unwrap_or(exe_dir)
    };
    let logs_dir = project_root.join("logs");
    if !logs_dir.exists() { let _ = std::fs::create_dir_all(&logs_dir); }

    // Create daily rolling file writers for feature logs
    fn mk_daily_writer(dir: &std::path::Path, base_name: &str) -> (tracing_appender::non_blocking::NonBlocking, WorkerGuard) {
        let rolling = tracing_appender::rolling::daily(dir, base_name);
        tracing_appender::non_blocking(rolling)
    }
    let (vad_en_writer, vad_en_guard) = mk_daily_writer(&logs_dir, "pcm_en.log");
    let (vad_hi_writer, vad_hi_guard) = mk_daily_writer(&logs_dir, "pcm_hi.log");
    let (ws_en_writer, ws_en_guard) = mk_daily_writer(&logs_dir, "ws_en.log");
    let (ws_hi_writer, ws_hi_guard) = mk_daily_writer(&logs_dir, "ws_hi.log");

    // File layers filtered by target (no console layer; avoid terminal logs)
    let vad_en_layer = fmt::layer().with_writer(vad_en_writer).with_ansi(false).with_target(false).with_filter(Targets::new().with_target("vad_en", tracing::Level::INFO));
    let vad_hi_layer = fmt::layer().with_writer(vad_hi_writer).with_ansi(false).with_target(false).with_filter(Targets::new().with_target("vad_hi", tracing::Level::INFO));
    let ws_en_layer  = fmt::layer().with_writer(ws_en_writer ).with_ansi(false).with_target(false).with_filter(Targets::new().with_target("ws_en", tracing::Level::INFO));
    let ws_hi_layer  = fmt::layer().with_writer(ws_hi_writer ).with_ansi(false).with_target(false).with_filter(Targets::new().with_target("ws_hi", tracing::Level::INFO));

    let subscriber = tracing_subscriber::registry()
        .with(vad_en_layer)
        .with(vad_hi_layer)
        .with(ws_en_layer)
        .with(ws_hi_layer);
    subscriber.init();

    // Keep guards alive
    let _guards: Vec<WorkerGuard> = vec![vad_en_guard, vad_hi_guard, ws_en_guard, ws_hi_guard];

    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(100);
    let go_server_url = std::env::var("ENGLISH_GO_SERVER_URL").unwrap_or_else(|_| "ws://127.0.0.1:8085/ws".to_string());

    // Settings file path
    let settings_path = project_root.join("data").join("settings.json");
    let settings = config::load_from(&settings_path).unwrap_or_default();

    let mut state = AppState::new(tx, go_server_url, settings, settings_path);

    // Initialize session manager (isolated from main state)
    let session_manager = Arc::new(sessions::SessionManager::new(project_root.join("data"))?);
    state.set_session_manager(session_manager.clone());
    let session_state = sessions::SessionState::new((*session_manager).clone());
    
    // NOTE: We do NOT auto-save all transcripts
    // Only save when user clicks "Ask AI" button (handled in AI endpoints)

    // Create sessions router with its own state
    let sessions_app = sessions::session_router()
        .with_state(session_state);
    
    // Merge main app with sessions app (both now have their own states)
    let app = routes::router(state)
        .merge(axum::Router::new().nest("/api/sessions", sessions_app));

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
