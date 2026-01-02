# Hindi Pipeline Architecture

## 📋 Overview

The Hindi pipeline handles voice transcription and AI-powered chat interactions in Hindi (हिंदी). It mirrors the English pipeline architecture but uses Hindi-optimized models and providers.

---

## 🏗️ Pipeline Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                     HINDI PIPELINE                              │
└────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                    AUDIO FLOW                                 │
└──────────────────────────────────────────────────────────────┘

    User Microphone (Hindi Speech)
          │
          ▼
    ┌─────────────┐
    │ WebSocket   │ ──> POST /api/start-hi
    │ Connection  │
    └─────────────┘
          │
          ▼
    ┌─────────────┐
    │   VAD       │ ──> Voice Activity Detection (Hindi)
    │  (Hindi)    │     - Detects speech vs silence
    └─────────────┘     - Sends only speech chunks
          │
          │ PCM Audio Chunks
          ▼
    ┌─────────────────┐
    │  Go Server      │
    │  (Whisper STT)  │ ──> Configured for Hindi
    │  Port: TBD      │     (separate or same as English)
    └─────────────────┘
          │
          │ Hindi Transcript Text
          ▼
    ┌─────────────────┐
    │  Broadcast      │
    │  Channel (HI)   │ ──> WebSocket: /ws/transcript-hi
    └─────────────────┘
          │
          ▼
    All Connected Clients


┌──────────────────────────────────────────────────────────────┐
│                      AI FLOW                                  │
└──────────────────────────────────────────────────────────────┘

    User Text Input (Hindi)
          │
          ▼
    POST /api/ai/hi
          │
          ▼
    ┌─────────────────┐
    │ chat_hi_auto()  │ ──> src/ai/mod.rs:129-149
    └─────────────────┘
          │
          ├──> Read settings.json
          ├──> Get provider (gemini/openrouter)
          ├──> Get first-time prompt (if not sent)
          │
          ▼
    ┌────────────────────────────────────────┐
    │      PRIMARY PROVIDER SELECTION        │
    └────────────────────────────────────────┘
          │
          ├─── Provider: GEMINI (default) ───┐
          │                                   │
          └─── Provider: OPENROUTER           │
                                             │
                                             ▼
                                  ┌──────────────────┐
                                  │  Try Provider    │
                                  │  API Call        │
                                  └──────────────────┘
                                             │
                                  ┌──────────┴──────────┐
                                  │                     │
                              Success               Failure
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
- **Section**: `hi`

### Hindi Settings Structure

```json
{
  "hi": {
    "provider": "gemini",                          // Primary provider (default)
    "model": "gemini-2.5-flash",                   // Override model
    "custom_model": null,                          // Custom model name
    "prompt": "आप सहायक सहायक हैं"                // System prompt (Hindi)
  }
}
```

### Provider Options

| Provider | Default Model | Use Case |
|----------|---------------|----------|
| **gemini** (default) | `gemini-2.5-flash` | Best for multilingual, Hindi support |
| **openrouter** | `meta-llama/llama-3.1-70b` | Access to Hindi-capable models |

**Note**: Groq is NOT supported for Hindi pipeline (code restriction)

---

## 🔄 Fallback Mechanism

### Fallback Flow Diagram

```
┌──────────────────────────────────────────────────────────────┐
│              PRIMARY PROVIDER: GEMINI                         │
└──────────────────────────────────────────────────────────────┘

    Try: chat_hi_gemini_with_prompt()
         │
         ├── API: https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent
         ├── Model: gemini-2.5-flash
         ├── Headers: x-goog-api-key: {GEMINI_KEY}
         ├── Prompt: "आप सहायक सहायक हैं" (if first time)
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
│              PRIMARY PROVIDER: OPENROUTER                     │
└──────────────────────────────────────────────────────────────┘

    Try: chat_hi_openrouter_with_prompt()
         │
         ├── API: https://openrouter.ai/api/v1/chat/completions
         ├── Model: meta-llama/llama-3.1-70b (or hi.model from settings)
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
**Function**: `chat_hi_auto()`  
**Lines**: 129-149

```rust
pub async fn chat_hi_auto(state: &AppState, input: String) 
    -> anyhow::Result<AiResult> 
{
    let s = state.settings.lock().await.clone();
    let prov = sanitize_provider_hi(&s.hi.provider);
    let prompt_once = get_prompt_if_first(state, "hi").await;
    
    match prov.as_str() {
        "gemini" => {
            match chat_hi_gemini_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { 
                    mark_prompt_sent(state, "hi").await; 
                    Ok(ok) 
                }
                Err(_) => { 
                    fallback_to_openrouter_primary(state, "hi", input, &prompt_once).await 
                }
            }
        }
        "openrouter" => {
            match chat_hi_openrouter_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { 
                    mark_prompt_sent(state, "hi").await; 
                    Ok(ok) 
                }
                Err(_) => { 
                    fallback_to_openrouter_alt(state, "hi", input, &prompt_once).await 
                }
            }
        }
        other => Err(anyhow::anyhow!("Unsupported HI provider: {}", other)),
    }
}
```

### Provider Sanitization

**File**: `src/ai/mod.rs`  
**Function**: `sanitize_provider_hi()`  
**Lines**: 86-91

```rust
fn sanitize_provider_hi(p: &str) -> String {
    match p {
        "" | "default" => "gemini".to_string(),
        "gemini" | "openrouter" => p.to_string(),
        _ => "gemini".to_string(),
    }
}
```

**Allowed Providers**: `gemini`, `openrouter`  
**Default**: `gemini`  
**Excluded**: Groq (not supported for Hindi)

---

## 🔌 API Endpoints

### Complete Endpoint Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                 HINDI API ENDPOINTS                           │
└──────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    AI ENDPOINTS                              │
└─────────────────────────────────────────────────────────────┘

1. AUTO ENDPOINT (with fallback)
   POST /api/ai/hi
        │
        ├─ Function: chat_hi_auto()
        ├─ Reads: settings.json → hi.provider
        ├─ Providers: gemini | openrouter
        ├─ Fallback: Always → OpenRouter
        │
        └─ Use: Production, reliability

2. DIRECT ENDPOINTS (NO fallback)
   POST /api/ai/hi/gemini
        │
        ├─ Function: chat_hi_gemini_direct()
        ├─ Provider: Gemini ONLY
        ├─ Fallback: None
        └─ Use: Testing, debugging

   POST /api/ai/hi/openrouter
        │
        ├─ Function: chat_hi_openrouter_direct()
        ├─ Provider: OpenRouter ONLY
        ├─ Fallback: None
        └─ Use: Testing, debugging

Note: Groq NOT supported for Hindi pipeline
```

### Endpoint Comparison Matrix

| Endpoint | Provider | Fallback | Status Codes | Use Case |
|----------|----------|----------|--------------|----------|
| `/api/ai/hi` | Auto | ✅ Yes | 200/401/502 | Production |
| `/api/ai/hi/gemini` | Gemini | ❌ No | 200/401/502 | Testing |
| `/api/ai/hi/openrouter` | OpenRouter | ❌ No | 200/401/502 | Testing |

### HTTP Status Codes (Standardized)

| Code | Meaning | When It Occurs | Example |
|------|---------|----------------|----------|
| **200** | Success | API call completed | `{"output":"...","provider":"gemini"}` |
| **400** | Bad Request | Invalid config/provider | `"No model for Gemini"` |
| **401** | Unauthorized | Missing/invalid API key | `"Missing Gemini API key"` |
| **502** | Bad Gateway | Provider unreachable | `"gemini http 503"` |
| **500** | Server Error | Unexpected failure | `"Internal error"` |

### Audio Endpoints

#### Start Hindi Audio Capture
```
POST /api/start-hi
Response: 200 OK | 400 BAD_REQUEST
```

#### Stop Hindi Audio Capture
```
POST /api/stop-hi
Response: 200 OK
```

#### Get Audio Status
```
GET /api/status-hi
Response: {"running": true/false}
```

#### WebSocket Transcript Stream
```
WebSocket: /ws/transcript-hi
Format: "HI: {transcript_text}"
```

#### VAD Statistics
```
GET /api/vad-status-hi
Response: {
  "sent": 123,
  "skipped": 45,
  "last_state": "speech|silence"
}
```

### AI Endpoints

#### Auto Provider Selection (Recommended)
```bash
POST /api/ai/hi
Content-Type: application/json

Body: {"input": "आपका सवाल"}

Success Response (200):
{
  "output": "एआई प्रतिक्रिया",
  "provider": "gemini",
  "model": "gemini-2.5-flash"
}

Error Response (401):
"Missing Gemini API key"

Error Response (502):
"gemini http 503"
```

**Behavior:**
- Uses configured provider from `settings.json → hi.provider`
- Falls back to OpenRouter if primary provider fails
- System prompt sent only once per app session

#### Direct Provider Endpoints (Testing)
```bash
# Force Gemini (NO fallback)
POST /api/ai/hi/gemini
Body: {"input": "आपका सवाल"}

Success Response (200):
{
  "output": "...",
  "provider": "gemini",
  "model": "gemini-2.5-flash"
}

Error Response (401):
"Missing Gemini API key"

# Force OpenRouter (NO fallback)
POST /api/ai/hi/openrouter
Body: {"input": "आपका सवाल"}

Success Response (200):
{
  "output": "...",
  "provider": "openrouter",
  "model": "meta-llama/llama-3.1-70b"
}

Error Response (401):
"Missing OpenRouter API key"
```

**Important:**
- Direct endpoints do NOT have fallback
- Used for testing specific providers
- Returns error immediately if provider fails

---

## 📊 State Management

### Session State
- **Location**: `state.hindi_session` (Arc<Mutex<Option<AudioSession>>>)
- **Lifecycle**: Created on `/api/start-hi`, destroyed on `/api/stop-hi`
- **Contents**: WebSocket connection, VAD state, broadcast channel

### Broadcast Channel
- **Location**: `state.hindi_tx` (tokio::sync::broadcast::Sender<String>)
- **Purpose**: Broadcast Hindi transcripts to all WebSocket clients

### VAD Statistics
- **Location**: `state.hindi_vad` (Arc<Mutex<VadStats>>)
- **Fields**:
  - `sent`: Number of audio chunks sent to STT
  - `skipped`: Number of silence chunks skipped
  - `last_state`: "speech" or "silence"

### Prompt Tracking
- **Location**: `state.prompt_sent_hi` (Arc<Mutex<bool>>)
- **Purpose**: Ensure system prompt sent only once per app run
- **Reset**: On app restart

---

## 🔐 API Keys

Hindi pipeline uses the same API keys as English:

- **Gemini**: `secrets::get_key("gemini")`
- **OpenRouter**: `secrets::get_key("openrouter")`

**Endpoints**:
```
POST   /api/providers/gemini/key       # Save key
DELETE /api/providers/gemini/key       # Delete key
GET    /api/providers/state            # Check key status
```

---

## 🎛️ Model Selection Logic

### Priority Order

1. **Language-specific model** (if provider matches):
   ```json
   {"hi": {"provider": "gemini", "model": "custom-model"}}
   ```
   → Uses `custom-model`

2. **Provider default model**:
   ```json
   {"providers": {"gemini": {"default_model": "gemini-2.5-flash"}}}
   ```
   → Uses `gemini-2.5-flash`

3. **Hardcoded fallback**: Provider's default if no config

### Code Reference

**File**: `src/ai/mod.rs`  
**Function**: `choose_model()`  
**Lines**: 14-28

---

## 🚀 Usage Example

### 1. Start Hindi Audio Capture
```bash
curl -X POST http://localhost:8080/api/start-hi
```

### 2. Connect WebSocket for Transcripts
```javascript
const ws = new WebSocket('ws://localhost:8080/ws/transcript-hi');
ws.onmessage = (event) => {
  console.log(event.data); // "HI: हिंदी पाठ"
};
```

### 3. Send AI Query
```bash
curl -X POST http://localhost:8080/api/ai/hi \
  -H "Content-Type: application/json" \
  -d '{"input": "रस्ट में async/await क्या है?"}'
```

Response:
```json
{
  "output": "Async/await रस्ट में...",
  "provider": "gemini",
  "model": "gemini-2.5-flash"
}
```

---

## 🌏 Multilingual Considerations

### Why Gemini for Hindi?
- **Native Indic Language Support**: Gemini models trained on diverse languages
- **Better Hindi Understanding**: Compared to Groq's primarily English-focused models
- **Unicode Handling**: Proper Devanagari script support

### Alternative Models (via OpenRouter)
- `meta-llama/llama-3.1-70b`: Good multilingual capability
- `anthropic/claude-3.5-sonnet`: Strong Hindi support
- Custom Hindi-optimized models from OpenRouter catalog

---

## 🐛 Troubleshooting

### Issue: Hindi characters not displaying
- **Check**: Frontend using UTF-8 encoding
- **Verify**: Response headers include `Content-Type: application/json; charset=utf-8`

### Issue: Poor Hindi transcription quality
- **Check**: Microphone input quality
- **Verify**: Go server configured for Hindi model
- **Test**: Manual audio file transcription

### Issue: AI responses in English instead of Hindi
- **Verify**: System prompt in Hindi (`settings.json → hi.prompt`)
- **Check**: Input text is actually in Hindi (Devanagari script)
- **Test**: Force prompt with explicit Hindi instruction

### Issue: Fallback not working
- **Ensure**: OpenRouter API key is set
- **Verify**: `settings.json` has `providers.openrouter.default_model`
- **Check**: OpenRouter model supports Hindi

---

## 📈 Performance Metrics

- **Latency (Gemini)**: ~500-1000ms
- **Latency (OpenRouter)**: ~1-2s (varies by model)
- **Audio Chunk Size**: 16kHz, 16-bit PCM
- **VAD Threshold**: Same as English pipeline

---

## 🔍 Key Differences from English Pipeline

| Feature | English | Hindi |
|---------|---------|-------|
| Default Provider | Groq | Gemini |
| Groq Support | ✅ Yes | ❌ No |
| Broadcast Channel | `state.tx` | `state.hindi_tx` |
| Session State | `state.session` | `state.hindi_session` |
| Prompt Sent Flag | `prompt_sent_en` | `prompt_sent_hi` |
| WebSocket Path | `/ws/transcript` | `/ws/transcript-hi` |

---

---

## 📦 Recent Updates (2025-10-27)

### Fixed Issues

1. **HTTP Status Code Standardization** ✅
   - Auto endpoint now returns 401 for missing keys (was 400)
   - Direct endpoints return 401 for missing keys (was 500)
   - Consistent error handling across all endpoints

2. **Direct Endpoint Implementation** ✅
   - Added `/api/ai/hi/gemini` (direct, no fallback)
   - Added `/api/ai/hi/openrouter` (direct, no fallback)
   - Enables provider-specific testing

3. **Prompt Management Fix** ✅
   - System prompt now sent correctly on first request
   - Subsequent requests don't resend prompt
   - Proper state management across sessions

### Breaking Changes

❌ None - All changes are backward compatible

### Migration Guide

No migration needed. Existing code continues to work:

```bash
# Existing auto endpoint (unchanged)
POST /api/ai/hi  # Still works with fallback

# New direct endpoints (optional)
POST /api/ai/hi/gemini      # New in 2025-10-27
POST /api/ai/hi/openrouter  # New in 2025-10-27
```

### Related Documents

- [ENDPOINT_COMPARISON.md](./ENDPOINT_COMPARISON.md) - Complete endpoint comparison
- [01_ENGLISH_PIPELINE.md](./01_ENGLISH_PIPELINE.md) - English pipeline
- [03_SCREENSHOT_PIPELINE.md](./03_SCREENSHOT_PIPELINE.md) - Screenshot pipeline
- [FIXED-ISSUES.md](../FIXED-ISSUES.md) - Complete list of fixes

---

**Last Updated**: 2025-10-27 (Post-Fix)  
**Version**: 2.0
