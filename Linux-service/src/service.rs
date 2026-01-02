use tokio::{process::Command, io::AsyncReadExt};
use tokio::sync::mpsc;
use std::time::Instant;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use axum::http::StatusCode;
use futures_util::{SinkExt, StreamExt};

use crate::{state::AppState, session::CaptureSession};
use crate::vad::{make_vad, mode_from_env, threshold_from_env, VadDecider};

#[derive(serde::Serialize, serde::Deserialize)]
struct AudioConfig { sample_rate: u32, channels: u16, bytes_per_sample: u16 }

pub async fn start_capture(state: AppState) -> Result<(), StatusCode> {
    // Enforce mutual exclusivity: lock English first, check Hindi while holding English lock
    let mut guard = state.session.lock().await;
    if guard.is_some() { return Err(StatusCode::CONFLICT); }
    if state.hindi_session.lock().await.is_some() { return Err(StatusCode::CONFLICT); }

    // Reset VAD stats at start
    {
        let mut stats = state.english_vad.lock().await;
        stats.sent = 0; stats.skipped = 0; stats.last_state = "idle".to_string();
    }

    // Resolve project root and bin dir from executable path
    let exe_path = std::env::current_exe().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let exe_dir = exe_path.parent().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    // Check if we're in target/debug or target/release (dev mode) or standalone dist folder (release mode)
    let project_root = if exe_dir.join("bin").exists() {
        // Portable/release mode: bin/ is adjacent to exe
        exe_dir.to_path_buf()
    } else {
        // Dev mode: target/debug/<exe> => project root two levels up
        exe_dir.parent().and_then(|p| p.parent()).unwrap_or(exe_dir).to_path_buf()
    };
    let bin_dir = project_root.join("bin");

    // Debug: log paths to file
    let debug_log = project_root.join("debug.log");
    let _ = std::fs::write(&debug_log, format!("[EN] project_root: {:?}\n", project_root));
    let _ = std::fs::OpenOptions::new().append(true).open(&debug_log).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "[EN] bin_dir: {:?}", bin_dir)
    });
    
    // Spawn Go server
    #[cfg(target_os = "windows")]
    let go_server_path = bin_dir.join("go-server-en.exe");
    #[cfg(target_os = "linux")]
    let go_server_path = bin_dir.join("go-server-en-linux");
    
    let _ = std::fs::OpenOptions::new().append(true).open(&debug_log).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "[EN] go_server_path: {:?}, exists: {}", go_server_path, go_server_path.exists())
    });
    if !go_server_path.exists() { return Err(StatusCode::INTERNAL_SERVER_ERROR); }
    
    // Redirect stderr to file for debugging
    let go_err_log = project_root.join("go-server-error.log");
    let stderr_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&go_err_log)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let _ = std::fs::OpenOptions::new().append(true).open(&debug_log).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "[EN] Spawning go-server: {:?}", go_server_path)?;
        writeln!(f, "[EN] Working directory: {:?}", project_root)
    });
    
    // Capture both stdout and stderr for debugging
    let stdout_log = project_root.join("go-server-stdout.log");
    let stdout_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stdout_log)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut go_server_child = Command::new(&go_server_path)
        .current_dir(&project_root)
        .stdout(stdout_file)
        .stderr(stderr_file)
        .spawn()
        .map_err(|e| {
            let _ = std::fs::OpenOptions::new().append(true).open(&debug_log).and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "[EN] Failed to spawn go-server: {:?}", e)
            });
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let _ = std::fs::OpenOptions::new().append(true).open(&debug_log).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "[EN] go-server spawned, PID: {:?}", go_server_child.id())
    });
    
    // Wait for server to start (align with reference)
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    
    // Check if go-server is still running
    if let Ok(Some(status)) = go_server_child.try_wait() {
        let _ = std::fs::OpenOptions::new().append(true).open(&debug_log).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "[EN] go-server exited early with status: {:?}", status)?;
            writeln!(f, "[EN] Check go-server-stdout.log and go-server-error.log for details")
        });
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let _ = std::fs::OpenOptions::new().append(true).open(&debug_log).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "[EN] go-server still running after 5s wait")
    });

    // Spawn capture binary
    #[cfg(target_os = "windows")]
    let cap_override = std::env::var("CAPTURE_CMD_WINDOWS").ok();
    #[cfg(target_os = "linux")]
    let cap_override = std::env::var("CAPTURE_CMD_LINUX").ok();
    #[cfg(target_os = "windows")]
    let capture_path = cap_override.map(std::path::PathBuf::from).unwrap_or_else(|| bin_dir.join("capture_windows.exe"));
    #[cfg(target_os = "linux")]
    let capture_path = cap_override.map(std::path::PathBuf::from).unwrap_or_else(|| bin_dir.join("SpeakerCapture"));
    if !capture_path.exists() {
        let _ = go_server_child.kill().await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let mut child = Command::new(&capture_path)
        .current_dir(&project_root)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut stdout = child.stdout.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Connect to Go WS
    let go_url = state.go_server_url.clone();
    // Chunk constants
    const SAMPLE_RATE: u32 = 16000;
    const CHANNELS: u16 = 1;
    const BYTES_PER_SAMPLE: u16 = 2;
    const CHUNK_DURATION_MS: u64 = 1000;
    let chunk_bytes: usize = (SAMPLE_RATE as usize) * (CHANNELS as usize) * (BYTES_PER_SAMPLE as usize) * ((CHUNK_DURATION_MS as usize) / 1000);

    let (tx_stop, rx_stop) = tokio::sync::oneshot::channel::<()>();
    let tx_broadcast = state.tx.clone();
    let vad_stats = state.english_vad.clone();
    // Bridge oneshot -> watch so multiple tasks can observe stop
    let (stop_tx, mut stop_rx_ws) = tokio::sync::watch::channel(false);
    let mut stop_rx_reader = stop_rx_ws.clone();
    tokio::spawn(async move {
        let _ = rx_stop.await;
        let _ = stop_tx.send(true);
    });

    // Channel: audio chunks (already VAD-filtered) -> WS sender
    let (audio_tx, mut audio_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // Reader task: read stdout, chunk, VAD filter, push to channel
    let label = "EN".to_string();
    let reader_task = tokio::spawn(async move {
        let mut read_buf = [0u8; 4096];
        let mut pcm_buf: Vec<u8> = Vec::with_capacity(chunk_bytes * 2);
        // Match GodseYe: use simple energy VAD default 0.005 unless overridden
        let vad_threshold = threshold_from_env(0.005);
        let mut vad = make_vad(mode_from_env(), vad_threshold);
        loop {
            tokio::select! {
                _ = stop_rx_reader.changed() => break,
                r = stdout.read(&mut read_buf) => {
                    let n = match r { Ok(n) => n, Err(_) => break };
                    if n == 0 { break; }
                    pcm_buf.extend_from_slice(&read_buf[..n]);
                    while pcm_buf.len() >= chunk_bytes {
                        let chunk = pcm_buf.drain(..chunk_bytes).collect::<Vec<u8>>();
                        if vad.process_chunk(&chunk) {
                            // update VAD stats (speech)
                            {
                                if let Ok(mut s) = vad_stats.try_lock() {
                                    s.sent += 1; s.last_state = "speech".to_string();
                                }
                            }
                            tracing::info!(target: "vad_en", "speech: send_bytes={}, chunk={}", chunk.len(), chunk_bytes);
                            let _ = audio_tx.send(chunk);
                        } else {
                            // update VAD stats (silence)
                            {
                                if let Ok(mut s) = vad_stats.try_lock() {
                                    s.skipped += 1; s.last_state = "silence".to_string();
                                }
                            }
                            tracing::info!(target: "vad_en", "silence: skip_bytes={}, chunk={}", chunk.len(), chunk_bytes);
                        }
                    }
                }
            }
        }
    });

    // WS handler task with reconnect (like GodseYe)
    let ws_task = tokio::spawn(async move {
        let label = "EN".to_string();
        let mut backoff_ms = 500u64;
        'outer: loop {
            // Stop requested?
            if *stop_rx_ws.borrow() { break 'outer; }

            match connect_async(&go_url).await {
                Ok((ws_stream, _)) => {
                    tracing::info!(target: "ws_en", "[WS] Connected to go-server at {}", go_url);
                    backoff_ms = 500; // reset backoff on success
                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                    // Send audio configuration first
                    let config = AudioConfig { sample_rate: SAMPLE_RATE, channels: CHANNELS, bytes_per_sample: BYTES_PER_SAMPLE };
                    if let Ok(cfg) = serde_json::to_string(&config) {
                        if ws_sender.send(WsMessage::Text(cfg)).await.is_err() {
                            // force reconnect
                            continue;
                        }
                    }

                    // Receiver task to avoid starvation
                    let (rx_done_tx, mut rx_done) = tokio::sync::oneshot::channel::<()>();
                    let mut ws_rx = ws_receiver; // move into spawned task
                    let tx_b = tx_broadcast.clone();
                    tokio::spawn(async move {
                        let mut last_txt: Option<String> = None;
                        let mut last_when = Instant::now();
                        const DEBOUNCE_MS: u64 = 800;
                        while let Some(msg) = ws_rx.next().await {
                            match msg {
                                Ok(WsMessage::Text(txt)) => {
                                    let t = txt.trim().to_string();
                                    if !t.is_empty() {
                                        let now = Instant::now();
                                        let same_as_last = last_txt.as_ref().map(|s| s == &t).unwrap_or(false);
                                        let within_window = now.duration_since(last_when).as_millis() < DEBOUNCE_MS as u128;
                                        if !(same_as_last && within_window) {
                                            tracing::info!(target: "ws_en", "{}", t);
                                            let _ = tx_b.send(t.clone());
                                            last_txt = Some(t);
                                            last_when = now;
                                        }
                                    }
                                }
                                Ok(WsMessage::Binary(_)) => {}
                                Ok(WsMessage::Close(_)) => break,
                                Ok(_) => {}
                                Err(_) => break,
                            }
                        }
                        let _ = rx_done_tx.send(());
                    });

                    loop {
                        tokio::select! {
                            // Stop?
                            _ = stop_rx_ws.changed() => { let _ = ws_sender.close().await; break 'outer; }
                            // Audio to send
                            maybe_chunk = audio_rx.recv() => {
                                match maybe_chunk {
                                    Some(data) => { if ws_sender.send(WsMessage::Binary(data)).await.is_err() { break; } }
                                    None => { let _ = ws_sender.close().await; break 'outer; }
                                }
                            }
                            // Receiver ended -> reconnect
                            _ = &mut rx_done => { break; }
                        }
                    }
                }
                Err(e) => {
                    tracing::info!(target: "ws_en", "[WS] Connection failed: {:?}", e);
                    // fallthrough to backoff
                }
            }
            // Backoff and retry
            let delay = std::time::Duration::from_millis(backoff_ms.min(5_000));
            tokio::time::sleep(delay).await;
            backoff_ms = (backoff_ms * 2).min(5_000);
        }
    });

    *guard = Some(CaptureSession { child, go_server_child: Some(go_server_child), shutdown: Some(tx_stop) });
    Ok(())
}

pub async fn stop_capture(state: AppState) -> Result<(), StatusCode> {
    let mut guard = state.session.lock().await;
    if let Some(mut s) = guard.take() {
        if let Some(tx) = s.shutdown.take() { let _ = tx.send(()); }
        let _ = s.child.kill().await;
        if let Some(mut go) = s.go_server_child.take() { let _ = go.kill().await; }
        Ok(())
    } else {
        Err(axum::http::StatusCode::NO_CONTENT)
    }
}

pub async fn start_capture_hi(state: AppState) -> Result<(), StatusCode> {
    // Enforce mutual exclusivity: lock English first, then Hindi
    let en_guard = state.session.lock().await;
    if en_guard.is_some() { return Err(StatusCode::CONFLICT); }
    let mut guard = state.hindi_session.lock().await;
    if guard.is_some() { return Err(StatusCode::CONFLICT); }

    // Reset VAD stats at start (Hindi)
    {
        let mut stats = state.hindi_vad.lock().await;
        stats.sent = 0; stats.skipped = 0; stats.last_state = "idle".to_string();
    }

    let exe_path = std::env::current_exe().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let exe_dir = exe_path.parent().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    // Check if we're in target/debug or target/release (dev mode) or standalone dist folder (release mode)
    let project_root = if exe_dir.join("bin").exists() {
        // Portable/release mode: bin/ is adjacent to exe
        exe_dir.to_path_buf()
    } else {
        // Dev mode: target/debug/<exe> => project root two levels up
        exe_dir.parent().and_then(|p| p.parent()).unwrap_or(exe_dir).to_path_buf()
    };
    let bin_dir = project_root.join("bin");

    #[cfg(target_os = "windows")]
    let go_server_path = bin_dir.join("go-server-hi.exe");
    #[cfg(target_os = "linux")]
    let go_server_path = bin_dir.join("go-server-hi-linux");
    if !go_server_path.exists() { return Err(StatusCode::INTERNAL_SERVER_ERROR); }
    
    // Redirect stderr to file for debugging
    let go_err_log = project_root.join("go-server-error.log");
    let stderr_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&go_err_log)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut go_server_child = Command::new(&go_server_path)
        .current_dir(&project_root)
        .stdout(std::process::Stdio::piped())
        .stderr(stderr_file)
        .spawn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    #[cfg(target_os = "windows")]
    let cap_override = std::env::var("CAPTURE_CMD_WINDOWS").ok();
    #[cfg(target_os = "linux")]
    let cap_override = std::env::var("CAPTURE_CMD_LINUX").ok();
    #[cfg(target_os = "windows")]
    let capture_path = cap_override.map(std::path::PathBuf::from).unwrap_or_else(|| bin_dir.join("capture_windows.exe"));
    #[cfg(target_os = "linux")]
    let capture_path = cap_override.map(std::path::PathBuf::from).unwrap_or_else(|| bin_dir.join("SpeakerCapture"));
    if !capture_path.exists() {
        let _ = go_server_child.kill().await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let mut child = Command::new(&capture_path)
        .current_dir(&project_root)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut stdout = child.stdout.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let go_url = state.hindi_go_server_url.clone();
    // Chunk constants
    const SAMPLE_RATE: u32 = 16000;
    const CHANNELS: u16 = 1;
    const BYTES_PER_SAMPLE: u16 = 2;
    const CHUNK_DURATION_MS: u64 = 1000;
    let chunk_bytes: usize = (SAMPLE_RATE as usize) * (CHANNELS as usize) * (BYTES_PER_SAMPLE as usize) * ((CHUNK_DURATION_MS as usize) / 1000);

    let (tx_stop, rx_stop) = tokio::sync::oneshot::channel::<()>();
    let tx_broadcast = state.hindi_tx.clone();
    let vad_stats = state.hindi_vad.clone();
    let (stop_tx, mut stop_rx_ws) = tokio::sync::watch::channel(false);
    let mut stop_rx_reader = stop_rx_ws.clone();
    tokio::spawn(async move { let _ = rx_stop.await; let _ = stop_tx.send(true); });

    let (audio_tx, mut audio_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // Reader task (Hindi)
    let label = "HI".to_string();
    let reader_task = tokio::spawn(async move {
        let mut read_buf = [0u8; 4096];
        let mut pcm_buf: Vec<u8> = Vec::with_capacity(chunk_bytes * 2);
        let vad_threshold = threshold_from_env(0.005);
        let mut vad = make_vad(mode_from_env(), vad_threshold);
        loop {
            tokio::select! {
                _ = stop_rx_reader.changed() => break,
                r = stdout.read(&mut read_buf) => {
                    let n = match r { Ok(n) => n, Err(_) => break };
                    if n == 0 { break; }
                    pcm_buf.extend_from_slice(&read_buf[..n]);
                    while pcm_buf.len() >= chunk_bytes {
                        let chunk = pcm_buf.drain(..chunk_bytes).collect::<Vec<u8>>();
                        if vad.process_chunk(&chunk) {
                            if let Ok(mut s) = vad_stats.try_lock() { s.sent += 1; s.last_state = "speech".to_string(); }
                            tracing::info!(target: "vad_hi", "speech: send_bytes={}, chunk={}", chunk.len(), chunk_bytes);
                            let _ = audio_tx.send(chunk);
                        } else {
                            if let Ok(mut s) = vad_stats.try_lock() { s.skipped += 1; s.last_state = "silence".to_string(); }
                            tracing::info!(target: "vad_hi", "silence: skip_bytes={}, chunk={}", chunk.len(), chunk_bytes);
                        }
                    }
                }
            }
        }
    });

    // WS handler with reconnect (Hindi)
    let ws_task = tokio::spawn(async move {
        let label = "HI".to_string();
        let mut backoff_ms = 500u64;
        'outer: loop {
            if *stop_rx_ws.borrow() { break 'outer; }

            match connect_async(&go_url).await {
                Ok((ws_stream, _)) => {
                    backoff_ms = 500;
                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                    // Send audio config
                    let config = AudioConfig { sample_rate: SAMPLE_RATE, channels: CHANNELS, bytes_per_sample: BYTES_PER_SAMPLE };
                    if let Ok(cfg) = serde_json::to_string(&config) {
                        if ws_sender.send(WsMessage::Text(cfg)).await.is_err() { continue; }
                    }
                    let (rx_done_tx, mut rx_done) = tokio::sync::oneshot::channel::<()>();
                    let mut ws_rx = ws_receiver;
                    let tx_b = tx_broadcast.clone();
                    tokio::spawn(async move {
                        let mut last_txt: Option<String> = None;
                        let mut last_when = Instant::now();
                        const DEBOUNCE_MS: u64 = 800;
                        while let Some(msg) = ws_rx.next().await {
                            match msg {
                                Ok(WsMessage::Text(txt)) => {
                                    let t = txt.trim().to_string();
                                    if !t.is_empty() {
                                        let now = Instant::now();
                                        let same_as_last = last_txt.as_ref().map(|s| s == &t).unwrap_or(false);
                                        let within_window = now.duration_since(last_when).as_millis() < DEBOUNCE_MS as u128;
                                        if !(same_as_last && within_window) {
                                            tracing::info!(target: "ws_hi", "{}", t);
                                            let _ = tx_b.send(t.clone());
                                            last_txt = Some(t);
                                            last_when = now;
                                        }
                                    }
                                }
                                Ok(WsMessage::Binary(_)) => {}
                                Ok(WsMessage::Close(_)) => break,
                                Ok(_) => {}
                                Err(_) => break,
                            }
                        }
                        let _ = rx_done_tx.send(());
                    });

                    loop {
                        tokio::select! {
                            _ = stop_rx_ws.changed() => { let _ = ws_sender.close().await; break 'outer; }
                            maybe_chunk = audio_rx.recv() => {
                                match maybe_chunk {
                                    Some(data) => { if ws_sender.send(WsMessage::Binary(data)).await.is_err() { break; } }
                                    None => { let _ = ws_sender.close().await; break 'outer; }
                                }
                            }
                            _ = &mut rx_done => { break; }
                        }
                    }
                }
                Err(_) => {}
            }
            let delay = std::time::Duration::from_millis(backoff_ms.min(5_000));
            tokio::time::sleep(delay).await;
            backoff_ms = (backoff_ms * 2).min(5_000);
        }
    });

    *guard = Some(CaptureSession { child, go_server_child: Some(go_server_child), shutdown: Some(tx_stop) });
    Ok(())
}

pub async fn stop_capture_hi(state: AppState) -> Result<(), StatusCode> {
    let mut guard = state.hindi_session.lock().await;
    if let Some(mut s) = guard.take() {
        if let Some(tx) = s.shutdown.take() { let _ = tx.send(()); }
        let _ = s.child.kill().await;
        if let Some(mut go) = s.go_server_child.take() { let _ = go.kill().await; }
        Ok(())
    } else {
        Err(axum::http::StatusCode::NO_CONTENT)
    }
}
