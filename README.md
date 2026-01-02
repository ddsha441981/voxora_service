# Voxora Service

A sophisticated multilingual voice-to-text transcription and AI assistant service written in Rust. Provides real-time audio processing, multiple AI provider support, and a comprehensive web-based interface with support for English, Hindi, and screen capture analysis capabilities.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Platform Support](#platform-support)
- [Project Structure](#project-structure)
- [Installation](#installation)
- [Usage](#usage)
- [API Endpoints](#api-endpoints)
- [Configuration](#configuration)
- [AI Providers](#ai-providers)
- [Development](#development)

---

## Overview

Voxora Service is a production-ready Rust application that combines:
- **Real-time Speech-to-Text** using Whisper-based transcription
- **AI Assistant Integration** with multiple providers (Groq, Gemini, OpenRouter)
- **Screen Capture Analysis** (Windows) for context-aware assistance
- **Multilingual Support** for English and Hindi
- **Web-based Interface** with responsive design for desktop and mobile
- **Secure Storage** using OS keyring for API keys

---

## Features

| Feature | Description |
|---------|-------------|
| **Voice Activity Detection** | Energy-based VAD with configurable thresholds |
| **Real-time Transcription** | WebSocket streaming of live transcripts |
| **Multilingual** | Separate pipelines for English (Port 8085) and Hindi (Port 8086) |
| **AI Provider Fallback** | Automatic fallback between Groq, Gemini, and OpenRouter |
| **Screen Analysis** | Windows-only screen capture with vision AI |
| **Remote LLM** | AnythingLLM integration for RAG capabilities |
| **Session History** | SQLite-based storage with export/import |
| **Secure Secrets** | OS keyring integration for API key storage |
| **Mobile Access** | QR code generation for easy mobile connection |

---

## Architecture

### High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           VOXORA SERVICE CORE                               │
│                         (Axum + Tokio + Rust)                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │   Routes     │  │    State     │  │   Config     │  │   Secrets    │   │
│  │   Module     │  │   Manager    │  │   Manager    │  │   Manager    │   │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        WebSocket Broadcast Channel                    │  │
│  │                    (Real-time Transcript Streaming)                   │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
        ▼                           ▼                           ▼
┌───────────────┐         ┌───────────────┐         ┌───────────────┐
│  Web Browser │         │ Audio Pipeline│         │  AI Providers │
│    (UI)      │         │   (Go/Whisper)│         │ (External)    │
└───────────────┘         └───────────────┘         └───────────────┘
```

### Audio Processing Pipeline

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         AUDIO PROCESSING PIPELINE                            │
└──────────────────────────────────────────────────────────────────────────────┘

    English Pipeline (Port 8085)        Hindi Pipeline (Port 8086)

    ┌─────────────┐                    ┌─────────────┐
    │   Audio     │                    │   Audio     │
    │   Source    │                    │   Source    │
    └──────┬──────┘                    └──────┬──────┘
           │                                  │
           ▼                                  ▼
    ┌─────────────┐                    ┌─────────────┐
    │  Go Server  │                    │  Go Server  │
    │  (Whisper)  │                    │  (Whisper)  │
    └──────┬──────┘                    └──────┬──────┘
           │                                  │
           ▼                                  ▼
    ┌─────────────┐                    ┌─────────────┐
    │     VAD     │                    │     VAD     │
    │   Detector  │                    │   Detector  │
    └──────┬──────┘                    └──────┬──────┘
           │                                  │
           ▼                                  ▼
    ┌────────────────────────────────────────────────┐
    │         WebSocket Broadcast Channel            │
    │            (to UI Clients)                     │
    └────────────────────────────────────────────────┘
```

### AI Integration Architecture

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          AI INTEGRATION LAYER                                │
└──────────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │                    AI Request Router                         │
    │  (Language-aware, Provider Selection, Fallback Logic)       │
    └─────────────────────────────────────────────────────────────┘
                                     │
         ┌───────────────────────────┼───────────────────────────┐
         │                           │                           │
         ▼                           ▼                           ▼
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│      Groq       │       │     Gemini      │       │   OpenRouter    │
│  (Fast Inference)│      │  (Multimodal)   │       │  (Multi-Model)  │
│  English Only   │       │  Text + Vision  │       │  Fallback       │
└─────────────────┘       └─────────────────┘       └─────────────────┘
         │                           │                           │
         └───────────────────────────┴───────────────────────────┘
                                     │
                                     ▼
                        ┌───────────────────────┐
                        │   Response Handler    │
                        │ (Streaming + Caching) │
                        └───────────────────────┘
```

### Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                             DATA FLOW DIAGRAM                                │
└──────────────────────────────────────────────────────────────────────────────┘

 USER SPEECH
      │
      ▼
 ┌─────────┐
 │  Audio  │
 │ Capture │
 └────┬────┘
      │
      ▼
 ┌─────────────────┐
 │  Go Whisper     │
 │  Server         │
 │  (STT)          │
 └────┬────────────┘
      │
      │ Transcript
      ▼
 ┌─────────────────┐
 │  VAD Analysis   │
 └────┬────────────┘
      │
      ▼
 ┌─────────────────────────┐
 │  WebSocket Broadcast    │
 └────┬────────────────────┘
      │
      ├─────────────────┬──────────────────┐
      ▼                 ▼                  ▼
 ┌─────────┐     ┌─────────┐       ┌──────────┐
 │ Web UI  │     │ Session │       │ AI Chat  │
 │ Display │     │ History │       │ Context  │
 └─────────┘     └─────────┘       └──────────┘
                                           │
                                           ▼
                                    ┌──────────┐
                                    │ AI       │
                                    │ Provider │
                                    └──────────┘
```

---

## Platform Support

### Linux Platform
```
┌──────────────────────────────────────────────────────────────────────────────┐
│                              LINUX PLATFORM                                  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Features:                                                                   │
│  ✅ Full audio capture and transcription                                     │
│  ✅ English and Hindi pipeline support                                       │
│  ✅ Systemd service integration                                              │
│  ✅ All AI providers (Groq, Gemini, OpenRouter)                              │
│  ✅ AnythingLLM remote integration                                           │
│  ❌ Screen capture (not available)                                           │
│                                                                              │
│  Deployment:                                                                 │
│  - Binary: voxora-service                                                   │
│  - Service: systemd unit file                                               │
│  - Audio: Native Linux audio capture                                        │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Windows Platform
```
┌──────────────────────────────────────────────────────────────────────────────┐
│                             WINDOWS PLATFORM                                 │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Features:                                                                   │
│  ✅ Full audio capture and transcription                                     │
│  ✅ English and Hindi pipeline support                                       │
│  ✅ Windows service support                                                  │
│  ✅ All AI providers (Groq, Gemini, OpenRouter)                              │
│  ✅ AnythingLLM remote integration                                           │
│  ✅ Screen capture with voxora-helper.exe                                    │
│                                                                              │
│  Deployment:                                                                 │
│  - Binary: voxora-service.exe                                               │
│  - Helper: voxora-helper.exe (runs in user session for screen capture)      │
│  - Service: Windows Service                                                  │
│  - Audio: WASAPI/Windows audio capture                                      │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Platform Comparison

| Feature | Linux | Windows |
|---------|-------|---------|
| Audio Capture | ✅ Native | ✅ WASAPI |
| English Pipeline | ✅ Port 8085 | ✅ Port 8085 |
| Hindi Pipeline | ✅ Port 8086 | ✅ Port 8086 |
| Screen Capture | ❌ N/A | ✅ voxora-helper |
| Service Integration | systemd | Windows Service |
| Console Hiding | N/A | ✅ Available |
| All AI Providers | ✅ | ✅ |
| AnythingLLM | ✅ | ✅ |

---

## Project Structure

```
voxora_service/
├── Linux-service/              # Linux-specific implementation
│   ├── src/                    # Source files (same as Windows)
│   │   ├── main.rs            # Entry point
│   │   ├── state.rs           # Application state
│   │   ├── routes.rs          # HTTP/WebSocket routes
│   │   ├── config.rs          # Configuration management
│   │   ├── secrets.rs         # Keyring integration
│   │   ├── service.rs         # Go server management
│   │   ├── session.rs         # Session tracking
│   │   ├── vad.rs             # Voice Activity Detection
│   │   ├── startup.rs         # Startup handler
│   │   ├── ui.rs              # Web UI components
│   │   └── ai/                # AI provider modules
│   │       ├── mod.rs
│   │       ├── groq.rs
│   │       ├── gemini.rs
│   │       └── openrouter.rs
│   ├── static/                # Web UI assets
│   ├── bin/                   # Helper binaries
│   │   ├── go-server-en-linux
│   │   ├── go-server-hi-linux
│   │   └── SpeakerCapture
│   ├── scripts/               # Setup scripts
│   │   └── setup_ubuntu.sh
│   ├── Cargo.toml
│   └── dist/                  # Portable distribution
│
├── window-service/             # Windows-specific implementation
│   ├── src/                    # Source files (same as Linux)
│   │   └── [same files as Linux]
│   │   └── bin/
│   │       └── helper.rs      # Windows screen capture helper
│   ├── static/                # Web UI assets
│   ├── bin/                   # Helper binaries
│   │   ├── go-server-en.exe
│   │   ├── go-server-hi.exe
│   │   ├── capture_windows.exe
│   │   └── SpeakerCapture.exe
│   ├── scripts/               # Setup scripts
│   │   └── build_windows.ps1
│   ├── Cargo.toml
│   └── dist/                  # Portable distribution
│
└── docs/                      # Shared documentation
    ├── 01_ARCHITECTURE.md
    ├── 02_AI_PROVIDERS.md
    └── ...
```

### Source File Overview

| File | Purpose |
|------|---------|
| `main.rs` | Application entry point, server setup (0.0.0.0:8080) |
| `state.rs` | Thread-safe application state management |
| `routes.rs` | 40+ HTTP/WebSocket endpoints |
| `config.rs` | JSON-based settings management |
| `secrets.rs` | OS keyring API key storage |
| `service.rs` | Go server process lifecycle |
| `session.rs` | Active session tracking |
| `vad.rs` | Voice activity detection |
| `startup.rs` | Startup page handler |
| `ui.rs` | Web interface components |
| `ai/groq.rs` | Groq API integration |
| `ai/gemini.rs` | Gemini API integration |
| `ai/openrouter.rs` | OpenRouter API integration |

---

## Installation

### Linux (Ubuntu/Debian)

```bash
# Navigate to Linux service directory
cd Linux-service

# Run setup script
sudo ./scripts/setup_ubuntu.sh

# Or manually:
cargo build --release
sudo cp target/release/voxora-service /usr/local/bin/
sudo chmod +x /usr/local/bin/voxora-service
```

**Systemd Service**

```ini
[Unit]
Description=Voxora Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/voxora-service
Restart=on-failure
User=voxora

[Install]
WantedBy=multi-user.target
```

### Windows

```powershell
# Navigate to Windows service directory
cd window-service

# Run build script
.\scripts\build_windows.ps1

# Or manually:
cargo build --release
```

**Windows Service Setup**

```powershell
# Install as service (requires administrator)
sc create VoxoraService binPath="C:\Path\To\voxora-service.exe" start=auto
sc start VoxoraService
```

### Portable Distribution

Both platforms support portable deployment:

```
voxora-portable/
├── bin/                    # Platform-specific binaries
├── static/                 # Web assets
├── data/                   # Runtime data (created on first run)
│   ├── settings.json
│   └── sessions.db
└── voxora-service          # Main executable
```

---

## Usage

### Starting the Service

**Linux:**
```bash
sudo systemctl start voxora-service
sudo systemctl enable voxora-service  # Auto-start on boot
```

**Windows:**
```powershell
sc start VoxoraService
```

### Accessing the Web Interface

1. Open browser to `http://localhost:8080`
2. Or use the startup page which shows your LAN IP and QR code
3. On mobile, scan the QR code for easy access

### Configuring API Keys

API keys are stored securely in the OS keyring:

```bash
# Via Web UI
Settings → API Keys → Add Provider

# Via REST API
curl -X POST http://localhost:8080/api/keys/groq \
  -H "Content-Type: application/json" \
  -d '{"api_key": "gsk_..."}'
```

### Starting Audio Capture

```bash
# Start English transcription
curl -X POST http://localhost:8080/api/audio/start/en

# Start Hindi transcription
curl -X POST http://localhost:8080/api/audio/start/hi

# Stop capture
curl -X POST http://localhost:8080/api/audio/stop
```

---

## API Endpoints

### WebSocket Endpoints

| Endpoint | Purpose |
|----------|---------|
| `WS /` | Real-time transcript streaming |
| `WS /mobile` | Mobile-optimized streaming |

### REST API Categories

**Audio Pipeline**
- `POST /api/audio/start/{lang}` - Start capture (en/hi)
- `POST /api/audio/stop` - Stop capture
- `GET /api/audio/status` - Current status
- `GET /api/audio/vad` - VAD statistics

**AI Chat**
- `POST /api/chat/completions` - Send chat message
- `POST /api/chat/stream` - Streaming response

**Settings**
- `GET /api/settings` - Get current settings
- `PUT /api/settings` - Update settings
- `POST /api/settings/reset` - Reset to defaults

**API Keys**
- `GET /api/keys` - List stored keys
- `POST /api/keys/{provider}` - Store key
- `DELETE /api/keys/{provider}` - Remove key

**Screen Capture** (Windows only)
- `GET /api/screen/capture` - Capture screen
- `POST /api/screen/analyze` - Analyze with AI

**Sessions**
- `GET /api/sessions` - List sessions
- `GET /api/sessions/{id}` - Get session details
- `DELETE /api/sessions/{id}` - Delete session
- `GET /api/sessions/export` - Export all sessions
- `POST /api/sessions/import` - Import sessions

See [docs/04_API_ENDPOINTS.md](Linux-service/docs/04_API_ENDPOINTS.md) for complete API documentation.

---

## Configuration

Settings are stored in `data/settings.json`:

```json
{
  "providers": {
    "groq": {
      "models": {
        "english": "llama-3.3-70b-versatile",
        "hindi": "llama-3.3-70b-versatile"
      },
      "enabled": true,
      "priority": 1
    },
    "gemini": {
      "models": {
        "english": "gemini-2.0-flash-exp",
        "hindi": "gemini-2.0-flash-exp",
        "screen_capture": "gemini-2.0-flash-exp"
      },
      "enabled": true,
      "priority": 2
    },
    "openrouter": {
      "models": {
        "english": "anthropic/claude-3.5-sonnet",
        "hindi": "anthropic/claude-3.5-sonnet"
      },
      "enabled": true,
      "priority": 3
    }
  },
  "prompts": {
    "english": "You are a helpful AI assistant...",
    "hindi": "आप एक सहायक AI सहायक हैं...",
    "screen_capture": "Analyze this screenshot..."
  },
  "remote_llm": {
    "enabled": false,
    "url": "http://localhost:3001"
  },
  "timeouts": {
    "default": 30,
    "streaming": 60
  }
}
```

---

## AI Providers

### Supported Providers

| Provider | Models | Features | Priority |
|----------|--------|----------|----------|
| **Groq** | Llama 3.x, Mixtral | Fastest inference, streaming | 1 |
| **Gemini** | 2.0 Flash, Pro | Multimodal (text + vision) | 2 |
| **OpenRouter** | Claude, GPT-4, Llama | Fallback, model variety | 3 |

### Provider Selection Logic

```
┌─────────────────────────────────────────────────────────────┐
│                    PROVIDER SELECTION                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Check provider enabled status                           │
│  2. Check language availability                             │
│  3. Check API key presence                                  │
│  4. Try providers in priority order                         │
│  5. Fallback to next provider on failure                    │
│  6. Return error if all providers fail                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Special Features

- **Screen Capture Analysis** (Windows): Automatically uses Gemini for vision
- **Language-specific Prompts**: Different system prompts per language
- **Streaming Support**: Groq supports real-time streaming responses
- **AnythingLLM Integration**: Remote LLM for RAG and document queries

---

## Development

### Building from Source

```bash
# Linux
cd Linux-service
cargo build --release

# Windows
cd window-service
cargo build --release
```

### Dependencies

Key Rust dependencies:

| Dependency | Version | Purpose |
|------------|---------|---------|
| axum | 0.7 | Web framework with WebSocket |
| tokio | 1 | Async runtime |
| tower-http | 0.5 | CORS, static files |
| serde | 1 | Serialization |
| keyring | 2 | Secure credential storage |
| reqwest | 0.12 | HTTP client for AI APIs |
| rusqlite | 0.31 | Session database |
| scrap | 0.5 | Screen capture (Windows) |
| qrcode | 0.13 | QR code generation |

### Running in Development

```bash
cargo run

# With debug logging
RUST_LOG=debug cargo run
```

### Testing

```bash
cargo test

# Run specific tests
cargo test --test api_tests
```

---

## Security

- **API Keys**: Stored in OS keyring, never in plaintext
- **CORS**: Configurable cross-origin policy
- **Local Processing**: Audio processed locally when possible
- **No Cloud Exposure**: Default configuration keeps data local

---

## Troubleshooting

### Common Issues

**Port already in use**
```bash
# Check what's using port 8080
sudo lsof -i :8080
```

**Go server not starting**
- Ensure binaries in `bin/` directory have execute permissions
- Check logs for detailed error messages

**Screen capture not working (Windows)**
- Ensure `voxora-helper.exe` is running
- Check Windows permissions for screen capture

**API key errors**
- Verify keys are stored: `GET /api/keys`
- Check key has required permissions

---

## License

See LICENSE file for details.

---

## Contributing

Contributions welcome! Please read CONTRIBUTING.md for guidelines.

---

## Support

- GitHub Issues: [Project Issues]
- Documentation: See `docs/` directory
- API Reference: See `docs/04_API_ENDPOINTS.md`