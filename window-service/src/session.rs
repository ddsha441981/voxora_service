use tokio::sync::oneshot;

#[derive(Debug)]
pub struct CaptureSession {
    pub child: tokio::process::Child,
    pub go_server_child: Option<tokio::process::Child>,
    pub shutdown: Option<oneshot::Sender<()>>,
}
