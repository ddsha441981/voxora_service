pub struct TestApp {
    pub app: axum::Router,
    _tmp: tempfile::TempDir,
    _settings: tempfile::NamedTempFile,
}

pub async fn build_app() -> TestApp {
    use voxora_service::{config, routes, state::AppState};

    let tmp = tempfile::tempdir().expect("tempdir");
    let settings_file = tempfile::NamedTempFile::new_in(tmp.path()).expect("settings temp file");
    let settings_path = settings_file.path().to_path_buf();

    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(16);
    let settings = config::Settings::default();

    let state = AppState::new(
        tx,
        "ws://127.0.0.1:8085/ws".to_string(),
        settings,
        settings_path,
    );

    let app = routes::router(state);

    TestApp { app, _tmp: tmp, _settings: settings_file }
}
