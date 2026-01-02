# English Pipeline Architecture

## 📋 Overview

The English pipeline handles voice transcription and AI-powered chat interactions in English. It features automatic provider selection, fallback mechanisms, and real-time WebSocket-based transcript delivery.

---

## 🏗️ Pipeline Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                    ENGLISH PIPELINE                             │
└────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                    AUDIO FLOW                                 │
└──────────────────────────────────────────────────────────────┘

    User Microphone
          │
          ▼
    ┌─────────────┐
    │ WebSocket   │ ──> POST /api/start
    │ Connection  │
    └─────────────┘
          │
          ▼
    ┌─────────────┐
    │   VAD       │ ──> Voice Activity Detection
    │ (English)   │     - Detects speech vs silence
    └─────────────┘     - Sends only speech chunks
          │
          │ PCM Audio Chunks
          ▼
    ┌─────────────────┐
    │  Go Server      │
    │  (Whisper STT)  │ ──> ws://127.0.0.1:8085/ws
    │  Port: 8085     │
    └─────────────────┘
          │
          │ Transcript Text
          ▼
    ┌─────────────────┐
    │  Broadcast      │
    │  Channel        │ ──> WebSocket: /ws/transcript
    └─────────────────┘
          │
          ▼
    All Connected Clients


┌──────────────────────────────────────────────────────────────┐
│                      AI FLOW                                  │
└──────────────────────────────────────────────────────────────┘

    User Text Input
          │
          ▼
    POST /api/ai/en
          │
          ▼
    ┌─────────────────┐
    │ chat_en_auto()  │ ──> src/ai/mod.rs:101-127
    └─────────────────┘
          │
          ├──> Read settings.json
          ├──> Get provider (groq/gemini/openrouter)
          ├──> Get first-time prompt (if not sent)
          │
          ▼
    ┌────────────────────────────────────────┐
    │      PRIMARY PROVIDER SELECTION        │
    └────────────────────────────────────────┘
          │
          ├─── Provider: GROQ ───┐
          │                      │
          ├─── Provider: GEMINI ─┤
          │                      │
          └─── Provider: OPENROUTER
                               │
                               ▼
                    ┌──────────────────┐
                    │  Try Provider    │
                    │  API Call        │
                    └──────────────────┘
                               │
                    ┌──────────┴──────────┐
                    │                     │
                Success                 Failure
                    │                     │
                    │                     ▼
                    │          ┌──────────────────┐
                    │          │  FALLBACK LOGIC  │
                    │          └──────────────────┘
                    │                     │
                    │          (See Fallback Section)
                    │                     │
                    └──────────┬──────────┘
                               │
                               ▼
                        ┌────────────┐
                        │   Result   │
                        │  {output,  │
                        │  provider, │
                        │  model}    │
                        └────────────┘
```

---

## ⚙️ Configuration

### Settings Location
- **File**: `data/settings.json`
- **Section**: `en`

### English Settings Structure

```json
{
  "en": {
    "provider": "groq",                               // Primary provider
    "model": "gemini-2.5-flash",                      // Override model
    "custom_model": null,                             // Custom model name
    "prompt": "You are helpful assistant..."         // System prompt
  }
}
```

### Provider Options

| Provider | Default Model | Use Case |
|----------|---------------|----------|
| **groq** (default) | `llama-3.1-8b-instant` | Fast inference, low latency |
| **gemini** | `gemini-2.5-flash` | Balanced performance |
| **openrouter** | `meta-llama/llama-3.1-70b` | Access to multiple models |

---

## 🔄 Fallback Mechanism

### Fallback Flow Diagram

```
┌──────────────────────────────────────────────────────────────┐
│              PRIMARY PROVIDER: GROQ                           │
└──────────────────────────────────────────────────────────────┘

    Try: chat_en_groq_with_prompt()
         │
         ├── API: https://api.groq.com/openai/v1/chat/completions
         ├── Model: llama-3.1-8b-instant
         ├── Headers: Authorization: Bearer {GROQ_KEY}
         │
         ├── Success ──> Return Result
         │
         └── Failure
                 │
                 ▼
    ┌────────────────────────────────────────┐
    │  FALLBACK: OpenRouter Primary          │
    │  Function: fallback_to_openrouter_     │
    │            primary()                   │
    └────────────────────────────────────────┘
         │
         ├── Model: providers.openrouter.default_model
         ├── Current: "meta-llama/llama-3.1-70b"
         ├── API: https://openrouter.ai/api/v1/chat/completions
         │
         └── Return Result


┌──────────────────────────────────────────────────────────────┐
│              PRIMARY PROVIDER: GEMINI                         │
└──────────────────────────────────────────────────────────────┘

    Try: chat_en_gemini_with_prompt()
         │
         ├── API: https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent
         ├── Model: gemini-2.5-flash
         ├── Headers: x-goog-api-key: {GEMINI_KEY}
         │
         ├── Success ──> Return Result
         │
         └── Failure
                 │
                 ▼
    ┌────────────────────────────────────────┐
    │  FALLBACK: OpenRouter Primary          │
    │  Model: meta-llama/llama-3.1-70b      │
    └────────────────────────────────────────┘


┌──────────────────────────────────────────────────────────────┐
│              PRIMARY PROVIDER: OPENROUTER                     │
└──────────────────────────────────────────────────────────────┘

    Try: chat_en_openrouter_with_prompt()
         │
         ├── API: https://openrouter.ai/api/v1/chat/completions
         ├── Model: meta-llama/llama-3.1-70b (from settings)
         ├── Headers: Authorization: Bearer {OPENROUTER_KEY}
         │
         ├── Success ──> Return Result
         │
         └── Failure
                 │
                 ▼
    ┌────────────────────────────────────────┐
    │  FALLBACK: OpenRouter Alternative      │
    │  Function: fallback_to_openrouter_alt()│
    └────────────────────────────────────────┘
         │
         ├── Model Selection Logic:
         │   - If fallback.openrouter_choice == "claude"
         │     → Use "anthropic/claude-3.5-sonnet"
         │   - Else
         │     → Use "openai/gpt-4o-mini"
         │
         └── Return Result
```

---

## 🎯 Code Reference

### Main Pipeline Function

**File**: `src/ai/mod.rs`  
**Function**: `chat_en_auto()`  
**Lines**: 101-127

```rust
pub async fn chat_en_auto(state: &AppState, input: String) 
    -> anyhow::Result<AiResult> 
{
    let s = state.settings.lock().await.clone();
    let prov = sanitize_provider_en(&s.en.provider);
    let prompt_once = get_prompt_if_first(state, "en").await;
    
    match prov.as_str() {
        "groq" => {
            match chat_en_groq_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { 
                    mark_prompt_sent(state, "en").await; 
                    Ok(ok) 
                }
                Err(_) => { 
                    fallback_to_openrouter_primary(state, "en", input, &prompt_once).await 
                }
            }
        }
        // Similar for "gemini" and "openrouter"...
    }
}
```

### Provider Sanitization

**File**: `src/ai/mod.rs`  
**Function**: `sanitize_provider_en()`  
**Lines**: 79-84

```rust
fn sanitize_provider_en(p: &str) -> String {
    match p {
        "" | "default" => "groq".to_string(),
        "groq" | "gemini" | "openrouter" => p.to_string(),
        _ => "groq".to_string(),
    }
}
```

**Allowed Providers**: `groq`, `gemini`, `openrouter`  
**Default**: `groq`

---

## 🔌 API Endpoints

### Endpoint Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    ENGLISH AI ENDPOINTS                          │
└─────────────────────────────────────────────────────────────────┘

        ┌─────────────────────────────────────────┐
        │     /api/ai/en (AUTO - Recommended)     │
        │        ✅ With Fallback Support         │
        └─────────────────────────────────────────┘
                        │
                        ├─── settings.en.provider = "groq"
                        │         │
                        │         ├─ Try: Groq
                        │         └─ Fail → OpenRouter (default model)
                        │
                        ├─── settings.en.provider = "gemini"
                        │         │
                        │         ├─ Try: Gemini
                        │         └─ Fail → OpenRouter (default model)
                        │
                        └─── settings.en.provider = "openrouter"
                                  │
                                  ├─ Try: OpenRouter (primary model)
                                  └─ Fail → OpenRouter (alt model: GPT/Claude)

        ┌─────────────────────────────────────────┐
        │    DIRECT PROVIDER ENDPOINTS            │
        │        ❌ NO Fallback Support           │
        └─────────────────────────────────────────┘
                        │
                        ├─ /api/ai/en/groq
                        │    └─ Only Groq → Fail: Return Error
                        │
                        ├─ /api/ai/en/gemini
                        │    └─ Only Gemini → Fail: Return Error
                        │
                        └─ /api/ai/en/openrouter
                             └─ Only OpenRouter → Fail: Return Error
```

### Detailed Fallback Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                  AUTO ENDPOINT: /api/ai/en                        │
│              Function: chat_en_auto() [FIXED 2025-10-27]         │
└──────────────────────────────────────────────────────────────────┘

   User Request
        │
        ▼
   ┌─────────────────┐
   │ Read Settings   │
   │ en.provider     │
   └─────────────────┘
        │
        ├────────────────┬────────────────┬─────────────────┐
        │                │                │                 │
        v                v                v                 v
   "groq"          "gemini"      "openrouter"     other→groq
        │                │                │
        v                v                v
   ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐
   │    Groq     │  │   Gemini    │  │   OpenRouter    │
   │   Primary   │  │   Primary   │  │    Primary      │
   └─────────────┘  └─────────────┘  └─────────────────┘
        │                │                │
     Success?         Success?         Success?
        │                │                │
    ├───┴───┐        ├───┴───┐        ├───┴───┐
   Yes     No       Yes     No       Yes     No
    │       │        │       │        │       │
    │       │        │       │        │       │
    v       v        v       v        v       v
   ✅    ┌────┐    ✅    ┌────┐    ✅    ┌────┐
   OK    │FB1 │    OK    │FB1 │    OK    │FB2 │
         └────┘         └────┘         └────┘
           │              │              │
           v              v              v
    ┌──────────────────────────────────────┐
    │ Fallback 1 (FB1):                    │
    │ OpenRouter Primary                   │
    │ Model: providers.openrouter.         │
    │        default_model                 │
    │ Default: meta-llama/llama-3.1-70b   │
    └──────────────────────────────────────┘
           │
           v
          ✅ Return Result


    ┌──────────────────────────────────────┐
    │ Fallback 2 (FB2):                    │
    │ OpenRouter Alternative               │
    │ Model based on:                      │
    │   fallback.openrouter_choice         │
    │   - "claude" → claude-3.5-sonnet     │
    │   - else → openai/gpt-4o-mini        │
    └──────────────────────────────────────┘
           │
           v
          ✅ Return Result
```

### Direct Endpoints Flow

```
┌──────────────────────────────────────────────────────────────────┐
│          DIRECT ENDPOINTS (NO FALLBACK) [NEW 2025-10-27]         │
└──────────────────────────────────────────────────────────────────┘

   POST /api/ai/en/groq
        │
        ├─ Function: chat_en_groq_direct()
        │
        v
   ┌─────────────┐
   │    Groq     │
   │    ONLY     │
   └─────────────┘
        │
     Success?
        │
    ├───┴───┐
   Yes     No
    │       │
    v       v
   ✅      ❌ Error
   OK     401/502/500


   POST /api/ai/en/gemini
        │
        ├─ Function: chat_en_gemini_direct()
        │
        v
   ┌─────────────┐
   │   Gemini    │
   │    ONLY     │
   └─────────────┘
        │
     Success?
        │
    ├───┴───┐
   Yes     No
    │       │
    v       v
   ✅      ❌ Error
   OK     401/502/500


   POST /api/ai/en/openrouter
        │
        ├─ Function: chat_en_openrouter_direct()
        │
        v
   ┌─────────────┐
   │ OpenRouter  │
   │    ONLY     │
   └─────────────┘
        │
     Success?
        │
    ├───┴───┐
   Yes     No
    │       │
    v       v
   ✅      ❌ Error
   OK     401/502/500
```

### Error Code Mapping [NEW 2025-10-27]

```
┌──────────────────────────────────────────────────────────────────┐
│              STANDARDIZED ERROR RESPONSES                         │
└──────────────────────────────────────────────────────────────────┘

   Error Type                    HTTP Status    Example Message
   ────────────────────────────────────────────────────────────────
   Missing API Key          →    401           "Missing Groq API key"
   Invalid API Key          →    401           "Unauthorized"
   No Model Configured      →    400           "No model for Groq"
   Unsupported Provider     →    400           "Unsupported EN provider"
   Network Error            →    502           "groq http 503"
   Unknown Error            →    500           Internal server error
```

### Audio Endpoints

#### Start English Audio Capture
```
POST /api/start
Response: 200 OK | 400 BAD_REQUEST
```

#### Stop English Audio Capture
```
POST /api/stop
Response: 200 OK
```

#### Get Audio Status
```
GET /api/status
Response: {"running": true/false}
```

#### WebSocket Transcript Stream
```
WebSocket: /ws/transcript
Format: "EN: {transcript_text}"
```

#### VAD Statistics
```
GET /api/vad-status
Response: {
  "sent": 123,
  "skipped": 45,
  "last_state": "speech|silence"
}
```

### AI Endpoints

#### Auto Provider Selection
```
POST /api/ai/en
Body: {"input": "your question"}
Response: {
  "output": "AI response",
  "provider": "groq",
  "model": "llama-3.1-8b-instant"
}
```

#### Force Specific Provider
```
POST /api/ai/en/groq       # Force Groq
POST /api/ai/en/gemini     # Force Gemini
POST /api/ai/en/openrouter # Force OpenRouter

Body: {"input": "your question"}
```

---

## 📊 State Management

### Session State
- **Location**: `state.session` (Arc<Mutex<Option<AudioSession>>>)
- **Lifecycle**: Created on `/api/start`, destroyed on `/api/stop`
- **Contents**: WebSocket connection, VAD state, broadcast channel

### VAD Statistics
- **Location**: `state.english_vad` (Arc<Mutex<VadStats>>)
- **Fields**:
  - `sent`: Number of audio chunks sent to STT
  - `skipped`: Number of silence chunks skipped
  - `last_state`: "speech" or "silence"

### Prompt Tracking
- **Location**: `state.prompt_sent_en` (Arc<Mutex<bool>>)
- **Purpose**: Ensure system prompt sent only once per app run
- **Reset**: On app restart

---

## 🔐 API Keys

All API keys stored securely in system keyring via `secrets.rs`:

- **Groq**: `secrets::get_key("groq")`
- **Gemini**: `secrets::get_key("gemini")`
- **OpenRouter**: `secrets::get_key("openrouter")`

**Endpoints**:
```
POST   /api/providers/groq/key       # Save key
DELETE /api/providers/groq/key       # Delete key
GET    /api/providers/state          # Check key status
```

---

## 🎛️ Model Selection Logic

### Priority Order

1. **Language-specific model** (if provider matches):
   ```json
   {"en": {"provider": "groq", "model": "custom-model"}}
   ```
   → Uses `custom-model`

2. **Provider default model**:
   ```json
   {"providers": {"groq": {"default_model": "llama-3.1-8b-instant"}}}
   ```
   → Uses `llama-3.1-8b-instant`

3. **Hardcoded fallback**: Provider's default if no config

### Code Reference

**File**: `src/ai/mod.rs`  
**Function**: `choose_model()`  
**Lines**: 14-28

---

## 🚀 Usage Example

### 1. Start Audio Capture
```bash
curl -X POST http://localhost:8080/api/start
```

### 2. Connect WebSocket for Transcripts
```javascript
const ws = new WebSocket('ws://localhost:8080/ws/transcript');
ws.onmessage = (event) => {
  console.log(event.data); // "EN: transcribed text"
};
```

### 3. Send AI Query
```bash
curl -X POST http://localhost:8080/api/ai/en \
  -H "Content-Type: application/json" \
  -d '{"input": "Explain async/await in Rust"}'
```

Response:
```json
{
  "output": "Async/await in Rust allows...",
  "provider": "groq",
  "model": "llama-3.1-8b-instant"
}
```

---

## 🐛 Troubleshooting

### Issue: Transcription not working
- **Check**: Go server running on port 8085
- **Verify**: `ENGLISH_GO_SERVER_URL` env var
- **Test**: `curl ws://127.0.0.1:8085/ws`

### Issue: AI requests failing
- **Check**: API key exists (`GET /api/providers/state`)
- **Verify**: Fallback to OpenRouter working
- **Logs**: Check for network errors

### Issue: Fallback not triggering
- **Ensure**: OpenRouter API key is set
- **Verify**: `settings.json` has `providers.openrouter.default_model`

---

## 📈 Performance Metrics

- **Latency (Groq)**: ~200-500ms
- **Latency (Gemini)**: ~500-1000ms
- **Latency (OpenRouter)**: ~1-2s (varies by model)
- **Audio Chunk Size**: 16kHz, 16-bit PCM
- **VAD Threshold**: Configurable (see `vad.rs`)

---

**Last Updated**: 2025-10-26  
**Version**: 1.0
