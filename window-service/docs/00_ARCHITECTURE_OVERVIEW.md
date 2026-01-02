# Voxora Service - Architecture Overview

## 📐 System Architecture

Voxora is a multilingual voice-to-text transcription and AI assistant service built with Rust, featuring real-time audio processing, multiple AI provider support, and a web-based interface.

```
┌─────────────────────────────────────────────────────────────────┐
│                      VOXORA SERVICE                              │
│                     (Rust/Axum Server)                           │
│                      Port: 8080                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  Web UI     │      │   Audio     │      │  AI/LLM     │
│  Interface  │      │  Pipeline   │      │  Pipeline   │
└─────────────┘      └─────────────┘      └─────────────┘
        │                     │                     │
        │                     │                     │
        ▼                     ▼                     ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│ - Startup   │      │ - English   │      │ - Groq      │
│ - Landing   │      │ - Hindi     │      │ - Gemini    │
│ - App       │      │ - VAD       │      │ - OpenRouter│
│ - Mobile    │      │ - WebSocket │      │ - Custom    │
└─────────────┘      └─────────────┘      └─────────────┘
```

---

## 🏗️ Core Components

### 1. **Main Server** (`main.rs`)
- Entry point for the application
- Sets up logging infrastructure (daily rolling logs)
- Initializes AppState with configuration
- Binds to `0.0.0.0:8080`
- Manages WebSocket broadcast channels

### 2. **Router** (`routes.rs`)
- Defines all HTTP/WebSocket endpoints
- Handles CORS and static file serving
- Groups endpoints by functionality:
  - UI Pages (`/`, `/home`, `/app`, `/mobile`, `/startup`)
  - English Audio (`/api/start`, `/api/stop`, `/ws/transcript`)
  - Hindi Audio (`/api/start-hi`, `/api/stop-hi`, `/ws/transcript-hi`)
  - AI Endpoints (`/api/ai/en`, `/api/ai/hi`, `/api/ai/sc`)
  - Settings Management (`/api/settings/*`)
  - Provider Keys (`/api/providers/:name/key`)
  - Remote LLM (`/api/remote/*`)
  - Screen Capture (`/api/capture`)

### 3. **State Management** (`state.rs`)
- Central application state (AppState)
- Thread-safe shared state using `Arc<Mutex<T>>`
- Manages:
  - Active sessions (English/Hindi)
  - Settings configuration
  - VAD statistics
  - WebSocket broadcast channels
  - Remote LLM selection
  - Prompt-sent flags (per language)

### 4. **Configuration** (`config.rs`)
- Settings file: `data/settings.json`
- Provider configurations (Groq, Gemini, OpenRouter)
- Language-specific settings (EN, HI, SC)
- Fallback configurations
- Remote LLM settings

### 5. **Secrets Management** (`secrets.rs`)
- Uses system keyring for secure storage
- Stores API keys for:
  - Groq
  - Gemini
  - OpenRouter
  - Custom providers
  - AnythingLLM (remote)
- Never logs or exposes keys in plaintext

---

## 🔄 Data Flow

### Audio Transcription Flow (English/Hindi)

```
┌──────────────┐
│  User Device │
│  (Microphone)│
└──────┬───────┘
       │
       │ Audio Stream
       ▼
┌──────────────────┐
│  VAD (Voice      │
│  Activity        │──── Silence ───> Skip
│  Detection)      │
└──────┬───────────┘
       │
       │ Speech Detected
       ▼
┌──────────────────┐
│  Go Server       │
│  (Whisper/STT)   │
│  Port: 8085      │
└──────┬───────────┘
       │
       │ Transcription
       ▼
┌──────────────────┐
│  WebSocket       │
│  Broadcast       │──────> All Connected Clients
└──────────────────┘
```

### AI Chat Flow (English/Hindi/Screenshot)

```
┌──────────────┐
│  User Input  │
│  (Text/Image)│
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│  Pipeline Router │
│  (ai::mod.rs)    │
└──────┬───────────┘
       │
       ├────> Primary Provider (Groq/Gemini/OpenRouter)
       │      │
       │      ├── Success ──> Return Result
       │      │
       │      └── Failure ──┐
       │                    │
       └────────────────────┘
                            │
                            ▼
                ┌──────────────────────┐
                │  Fallback Pipeline   │
                │  (OpenRouter)        │
                └──────────────────────┘
                            │
                            ▼
                      Return Result
```

---

## 📂 Project Structure

```
voxora-service/
├── src/
│   ├── main.rs              # Entry point (service binary)
│   ├── routes.rs            # API endpoints
│   ├── state.rs             # Application state
│   ├── config.rs            # Configuration management
│   ├── secrets.rs           # Keyring/secrets
│   ├── service.rs           # Audio service logic
│   ├── session.rs           # Session management
│   ├── vad.rs               # Voice Activity Detection
│   ├── ui.rs                # HTML templates
│   ├── startup.rs           # Startup page logic
│   ├── ai/
│   │   ├── mod.rs           # AI pipeline orchestration
│   │   ├── groq.rs          # Groq API integration
│   │   ├── gemini.rs        # Gemini API integration
│   │   └── openrouter.rs    # OpenRouter API integration
│   └── bin/
│       └── helper.rs        # Helper binary for screen capture
├── data/
│   └── settings.json        # Configuration file
├── static/                  # Static assets (CSS/JS)
├── logs/                    # Application logs
│   ├── pcm_en.log          # English VAD logs
│   ├── pcm_hi.log          # Hindi VAD logs
│   ├── ws_en.log           # English WebSocket logs
│   └── ws_hi.log           # Hindi WebSocket logs
└── docs/                    # Documentation
    ├── 00_ARCHITECTURE_OVERVIEW.md
    ├── 01_ENGLISH_PIPELINE.md
    ├── 02_HINDI_PIPELINE.md
    ├── 03_SCREENSHOT_PIPELINE.md
    ├── 04_API_ENDPOINTS.md
    ├── 05_CONFIGURATION_GUIDE.md
    ├── 06_FALLBACK_PIPELINE.md
    └── 07_ANYTHINGLLM_PIPELINE.md
```

---

## 🔌 External Dependencies

### Audio Processing
- **Go Server** (Port 8085): Whisper-based speech-to-text
  - Default: `ws://127.0.0.1:8085/ws`
  - Configurable via `ENGLISH_GO_SERVER_URL` env var

### Screen Capture (Windows)
- **Helper Binary** (`voxora-helper.exe`): Runs in user session
  - Port: 8081 (localhost HTTP server)
  - Purpose: Captures screens from Session 1 (user desktop)
  - Communication: Service calls helper via HTTP
  - See: [SCREEN_CAPTURE_HELPER.md](../SCREEN_CAPTURE_HELPER.md)

### AI Providers
- **Groq**: Fast LLM inference
- **Gemini**: Google's multimodal AI (text + vision)
- **OpenRouter**: Unified API for multiple LLMs

### Remote LLM (Optional)
- **AnythingLLM**: Self-hosted LLM platform
- Features: Workspace management, RAG, streaming

---

## 🔒 Security Features

1. **Keyring Storage**: API keys stored in OS-level secure storage
2. **No Plain-text Secrets**: Keys never logged or exposed
3. **CORS Protection**: Configurable cross-origin policies
4. **Local-First**: All processing happens locally or via user-controlled APIs

---

## 📊 Logging Strategy

- **Daily Rolling Logs**: Separate files for each component
- **Feature-Based Logs**: VAD and WebSocket logs per language
- **No Console Output**: Logs written to files only (Windows service compatible)
- **Tracing Targets**:
  - `vad_en`: English VAD events
  - `vad_hi`: Hindi VAD events
  - `ws_en`: English WebSocket events
  - `ws_hi`: Hindi WebSocket events

---

## 🚀 Startup Sequence

1. **Initialize Logging**: Create logs directory, setup rolling file writers
2. **Load Configuration**: Read `data/settings.json`
3. **Create AppState**: Initialize shared state with default values
4. **Setup Router**: Configure all routes and middleware
5. **Bind Server**: Listen on `0.0.0.0:8080`
6. **Serve Requests**: Handle incoming HTTP/WebSocket connections

---

## 🎯 Key Design Principles

1. **Separation of Concerns**: Clear boundaries between audio, AI, and UI layers
2. **Provider Flexibility**: Easy to add new AI providers
3. **Language Isolation**: Independent pipelines for each language
4. **Graceful Degradation**: Fallback mechanisms for provider failures
5. **State Immutability**: Settings changes require explicit persistence
6. **Thread Safety**: All shared state protected by async mutexes

---

## 📈 Scalability Considerations

- **Single-threaded Audio Processing**: One active session per language
- **Concurrent AI Requests**: Multiple simultaneous AI calls supported
- **WebSocket Broadcasting**: Efficient pub-sub pattern for transcripts
- **Stateless API**: Most endpoints are stateless (except audio sessions)

---

## 🔮 Future Enhancements

- Multi-user support (currently single-user)
- More language support beyond English/Hindi
- Custom AI provider SDK
- Audio recording/playback
- Advanced VAD tuning UI

---

**Last Updated**: 2025-10-27  
**Version**: 1.1

### Recent Changes
- Added helper binary for Windows screen capture (Session 0/1 architecture)
- Updated project structure with `src/bin/helper.rs`
- Added screen capture helper to external dependencies
