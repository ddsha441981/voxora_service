// voxora-helper.exe
// Runs in user session (Session 1) to capture screen
// Provides HTTP API for service to request captures

use axum::{
    Router,
    routing::get,
    Json,
    http::StatusCode,
};
use image::ImageEncoder;
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;

#[cfg(target_os = "windows")]
use scrap::{Capturer, Display};
use image::{ImageBuffer, Rgba};
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize, Deserialize)]
struct CaptureResponse {
    image: String,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[cfg(target_os = "windows")]
fn capture_screen() -> Result<CaptureResponse, String> {
    use std::{thread, time::Duration};
    
    let display = Display::primary().map_err(|e| format!("Failed to get primary display: {}", e))?;
    let mut capturer = Capturer::new(display).map_err(|e| format!("Failed to create capturer: {}", e))?;
    
    let (w, h) = (capturer.width(), capturer.height());
    
    // Wait for a non-empty frame (up to 2 seconds)
    let start = std::time::Instant::now();
    let frame = loop {
        match capturer.frame() {
            Ok(buf) => {
                // Check if frame has any non-zero data
                if buf.iter().any(|&b| b != 0) {
                    break buf.to_vec();
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(format!("Failed to capture frame: {}", e)),
        }
        
        if start.elapsed() > Duration::from_secs(2) {
            return Err("Timeout waiting for non-empty frame".to_string());
        }
    };
    
    // Convert BGRA to RGBA
    let mut img = ImageBuffer::<Rgba<u8>, _>::new(w as u32, h as u32);
    let stride = if h > 0 { frame.len() / h } else { 0 };
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * stride + x * 4;
            if idx + 3 < frame.len() {
                let b = frame[idx];
                let g = frame[idx + 1];
                let r = frame[idx + 2];
                img.put_pixel(x as u32, y as u32, Rgba([r, g, b, 255]));
            }
        }
    }
    
    // Encode to PNG
    let mut png_data = Vec::new();
    {
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        encoder.write_image(
            img.as_raw(),
            w as u32,
            h as u32,
            image::ColorType::Rgba8,
        ).map_err(|e| format!("Failed to encode PNG: {}", e))?;
    }
    
    // Convert to base64
    let base64_image = general_purpose::STANDARD.encode(&png_data);
    
    Ok(CaptureResponse {
        image: base64_image,
        width: w as u32,
        height: h as u32,
    })
}

#[cfg(not(target_os = "windows"))]
fn capture_screen() -> Result<CaptureResponse, String> {
    Err("Screen capture not supported on this platform".to_string())
}

async fn capture_handler() -> Result<Json<CaptureResponse>, (StatusCode, String)> {
    match capture_screen() {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[cfg(target_os = "windows")]
fn hide_console_window() {
    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};
    
    unsafe {
        let window = GetConsoleWindow();
        if !window.is_null() {
            ShowWindow(window, SW_HIDE);
        }
    }
}

#[tokio::main]
async fn main() {
    // Hide console window on Windows
    #[cfg(target_os = "windows")]
    hide_console_window();
    
    // Build router
    let app = Router::new()
        .route("/capture", get(capture_handler))
        .route("/health", get(health_handler));
    
    // Bind to localhost only (security)
    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    
    println!("Voxora Helper starting on {}", addr);
    
    // Start server (using axum 0.7 API)
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind helper to {}: {}", addr, e);
            eprintln!("Another instance might be running.");
            return;
        }
    };
    
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Helper server error: {}", e);
    }
}
