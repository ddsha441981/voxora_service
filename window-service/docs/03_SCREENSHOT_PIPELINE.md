# Screenshot Pipeline Architecture

## 📋 Overview

The Screenshot (SC) pipeline analyzes screen captures using vision-capable AI models. It's designed for context-aware assistance, code review, debugging, and visual content analysis.

**Windows Service Architecture**: Uses a helper application pattern to capture screens from user session.

---

## 🏗️ Pipeline Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                  SCREENSHOT PIPELINE (Windows)                  │
└────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                  CAPTURE FLOW (HELPER PATTERN)                │
└──────────────────────────────────────────────────────────────┘

    User Trigger
          │
          ▼
    POST /api/capture  (Service - Session 0)
          │
          ▼
    HTTP GET http://127.0.0.1:8081/capture
          │
          ▼
    ┌──────────────────────────────────────────┐
    │  voxora-helper.exe (Session 1)           │
    │  • Runs in user session                  │
    │  • HTTP server on port 8081              │
    │  • Invisible (no window/tray)            │
    │  • Auto-starts on login                  │
    └──────────────────────────────────────────┘
          │
          ▼
    ┌──────────────────┐
    │  Screen Capture  │ ──> Uses scrap crate
    │  (Primary        │     - Captures primary display
    │   Display)       │     - BGRA to RGBA conversion
    └──────────────────┘     - PNG encoding
          │
          │ Raw Pixels (BGRA)
          ▼
    ┌──────────────────┐
    │  Image Processing│
    │  - Convert BGRA  │
    │    to RGBA       │
    │  - Encode PNG    │
    │  - Base64 encode │
    └──────────────────┘
          │
          │ JSON: {image: base64, width, height}
          ▼
    Return to Service (Session 0)
          │
          │ Base64 PNG String
          ▼
    ┌──────────────────┐
    │  Vision AI       │
    │  Analysis        │
    └──────────────────┘
          │
          ▼
    Return Analysis


┌──────────────────────────────────────────────────────────────┐
│                     AI VISION FLOW                            │
└──────────────────────────────────────────────────────────────┘

    Base64 PNG Image + Optional Text Input
          │
          ▼
    POST /api/ai/sc  OR  POST /api/capture
          │
          ▼
    ┌─────────────────┐
    │ chat_sc_auto()  │ ──> src/ai/mod.rs:151-177
    └─────────────────┘     OR
    ┌─────────────────────┐
    │ sc_analyze_image()  │ ──> src/ai/mod.rs:201-263
    └─────────────────────┘
          │
          ├──> Read settings.json (sc section)
          ├──> Get provider (gemini/groq/openrouter)
          ├──> Get first-time prompt (if not sent)
          │
          ▼
    ┌────────────────────────────────────────┐
    │      PRIMARY PROVIDER SELECTION        │
    └────────────────────────────────────────┘
          │
          ├─── Provider: GEMINI (vision) ──┐
          │                                 │
          ├─── Provider: GROQ (via OR) ────┤
          │                                 │
          └─── Provider: OPENROUTER (vision)
                                           │
                                           ▼
                                ┌──────────────────┐
                                │  Try Vision API  │
                                │  with Image      │
                                └──────────────────┘
                                           │
                                ┌──────────┴──────────┐
                                │                     │
                            Success               Failure
                                │                     │
                                │                     ▼
                                │          ┌──────────────────┐
                                │          │  FALLBACK LOGIC  │
                                │          │  (Conditional)   │
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
- **Section**: `sc`

### Screenshot Settings Structure

```json
{
  "sc": {
    "provider": "groq",                           // Primary provider
    "model": null,                                 // Override model (optional)
    "custom_model": null,                          // Custom model name
    "prompt": "You are ScreenCapture‑AI..."       // Specialized system prompt
  },
  "sc_providers": {
    "groq": {
      "default_model": "meta-llama/llama-4-scout-17b-16e-instruct",
      "extra_models": ""
    },
    "gemini": {
      "default_model": "gemini-2.5-flash",
      "extra_models": ""
    },
    "openrouter": {
      "default_model": "google/gemini-2.5-flash",
      "extra_models": ""
    }
  },
  "sc_fallback_or": true,                          // Enable OpenRouter fallback
  "sc_fallback_or_model": "google/gemini-2.5-flash"  // Fallback model
}
```

### Provider Options

| Provider | Default Model | Vision Support | Use Case |
|----------|---------------|----------------|----------|
| **groq** (default) | `meta-llama/llama-4-scout-17b-16e-instruct` | ✅ (via OpenRouter) | Fast, code-focused |
| **gemini** | `gemini-2.5-flash` | ✅ Native | Strong multimodal |
| **openrouter** | `google/gemini-2.5-flash` | ✅ Native | Access to many vision models |

---

## 🔄 Fallback Mechanism

### Key Difference: Conditional Fallback

Unlike English/Hindi pipelines, Screenshot fallback is **conditional** and controlled by:
- `sc_fallback_or`: Boolean flag (default: `true`)
- `sc_fallback_or_model`: Specific fallback model

### Fallback Flow Diagram

```
┌──────────────────────────────────────────────────────────────┐
│              PRIMARY PROVIDER: GEMINI                         │
└──────────────────────────────────────────────────────────────┘

    Try: gemini::generate_with_image()
         │
         ├── API: https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent
         ├── Model: gemini-2.5-flash (from sc_providers)
         ├── Headers: x-goog-api-key: {GEMINI_KEY}
         ├── Body: {contents: [{parts: [{text}, {inline_data: {mime_type, data}}]}]}
         │
         ├── Success ──> Return Result
         │
         └── Failure
                 │
                 ▼
         ┌────────────────────┐
         │ Check: sc_fallback │
         │        _or == true?│
         └────────────────────┘
                 │
           ┌─────┴─────┐
          Yes         No
           │           │
           │           └──> Return Original Error
           │
           ▼
    ┌────────────────────────────────────────┐
    │  FALLBACK: OpenRouter Vision           │
    │  Model: sc_fallback_or_model          │
    │  Current: "google/gemini-2.5-flash"   │
    └────────────────────────────────────────┘
           │
           ├── API: https://openrouter.ai/api/v1/chat/completions
           ├── Function: openrouter::chat_with_image()
           │
           └── Return Result


┌──────────────────────────────────────────────────────────────┐
│              PRIMARY PROVIDER: GROQ (via OpenRouter)          │
└──────────────────────────────────────────────────────────────┘

    Try: openrouter::chat_with_image() with Groq model
         │
         ├── API: https://openrouter.ai/api/v1/chat/completions
         ├── Model: meta-llama/llama-4-scout-17b-16e-instruct
         ├── Headers: Authorization: Bearer {OPENROUTER_KEY}
         ├── Body: {model, messages: [{role, content: [{type:text}, {type:image_url}]}]}
         │
         ├── Success ──> Return Result (provider: "groq")
         │
         └── Failure
                 │
                 ▼
         ┌────────────────────┐
         │ Check: sc_fallback │
         │        _or == true?│
         └────────────────────┘
                 │
           ┌─────┴─────┐
          Yes         No
           │           │
           │           └──> Return Original Error
           │
           ▼
    ┌────────────────────────────────────────┐
    │  FALLBACK: OpenRouter Vision           │
    │  Model: sc_fallback_or_model          │
    │  Current: "google/gemini-2.5-flash"   │
    └────────────────────────────────────────┘


┌──────────────────────────────────────────────────────────────┐
│              PRIMARY PROVIDER: OPENROUTER                     │
└──────────────────────────────────────────────────────────────┘

    Try: openrouter::chat_with_image()
         │
         ├── API: https://openrouter.ai/api/v1/chat/completions
         ├── Model: google/gemini-2.5-flash (or sc.model from settings)
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
    │  Model: sc_fallback_or_model or based │
    │         on fallback.openrouter_choice  │
    └────────────────────────────────────────┘
```

---

## 🎯 Code Reference

### Main Pipeline Functions

#### For Chat-Style Interaction

**File**: `src/ai/mod.rs`  
**Function**: `chat_sc_auto()`  
**Lines**: 151-177

```rust
pub async fn chat_sc_auto(state: &AppState, input: String) 
    -> anyhow::Result<AiResult> 
{
    let s = state.settings.lock().await.clone();
    let prov = sanitize_provider_sc(&s.sc.provider);
    let prompt_once = get_prompt_if_first(state, "sc").await;
    
    match prov.as_str() {
        "gemini" => {
            match chat_en_gemini_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { /* ... */ }
                Err(_) => { 
                    fallback_to_openrouter_primary(state, "sc", input, &prompt_once).await 
                }
            }
        }
        // Similar for "groq" and "openrouter"...
    }
}
```

#### For Image Analysis

**File**: `src/ai/mod.rs`  
**Function**: `sc_analyze_image()`  
**Lines**: 201-263

```rust
pub async fn sc_analyze_image(state: &AppState, base64_png: String) 
    -> anyhow::Result<AiResult> 
{
    let s = state.settings.lock().await.clone();
    let prov = sanitize_provider_sc(&s.sc.provider);
    let sc_fallback = s.sc_fallback_or;
    let sc_fallback_model = s.sc_fallback_or_model.clone()
        .or_else(|| s.sc_providers.openrouter.default_model.clone());
    let prompt_once = get_prompt_if_first(state, "sc").await;
    
    match prov.as_str() {
        "gemini" => {
            let res = gemini::generate_with_image(&key, &model, &prompt_once, &base64_png).await;
            if res.is_ok() { return res; }
            
            // Conditional fallback
            if sc_fallback {
                let out = openrouter::chat_with_image(&key, &model, &prompt_once, &base64_png).await?;
                return Ok(AiResult { output: out, provider: "openrouter".into(), model });
            }
            res
        }
        // Similar for "groq" and "openrouter"...
    }
}
```

### Provider Sanitization

**File**: `src/ai/mod.rs`  
**Function**: `sanitize_provider_sc()`  
**Lines**: 93-98

```rust
fn sanitize_provider_sc(p: &str) -> String {
    match p {
        "" | "default" => "gemini".to_string(),
        "gemini" | "groq" | "openrouter" => p.to_string(),
        _ => "gemini".to_string(),
    }
}
```

**Allowed Providers**: `gemini`, `groq`, `openrouter`  
**Default**: `gemini`

---

## 🔌 API Endpoints

### Complete Endpoint Architecture

```
┌──────────────────────────────────────────────────────────────┐
│            SCREEN CAPTURE API ENDPOINTS                      │
└──────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                VISION AI ENDPOINTS                           │
└─────────────────────────────────────────────────────────────┘

1. IMAGE CAPTURE + ANALYSIS (One-Shot)
   POST /api/capture
        │
        ├─ Function: sc_analyze_image()
        ├─ Process: Screen Capture → PNG → Base64 → Vision AI
        ├─ Providers: gemini (vision) | groq* | openrouter (vision)
        ├─ Fallback: Conditional (sc_fallback_or)
        │
        └─ Use: Screen analysis, code review
        
        *Note: "groq" uses OpenRouter infrastructure
               (Groq doesn't support vision natively)

2. TEXT-BASED QUERY (No Image)
   POST /api/ai/sc
        │
        ├─ Function: chat_sc_auto()
        ├─ Providers: gemini | groq | openrouter
        ├─ Fallback: Always → OpenRouter
        │
        └─ Use: Context queries without image

Note: NO direct endpoints (e.g., /api/ai/sc/gemini) for SC
```

### Endpoint Comparison Matrix

| Endpoint | Input Type | Provider | Vision | Fallback | Status Codes |
|----------|------------|----------|--------|----------|-------------|
| `/api/capture` | Image (auto-captured) | Auto | Required | Conditional | 200/401/502 |
| `/api/ai/sc` | Text only | Auto | No | Always | 200/401/502 |

### HTTP Status Codes (Standardized)

| Code | Meaning | When It Occurs | Example |
|------|---------|----------------|----------|
| **200** | Success | Vision analysis completed | `{"output":"...","provider":"gemini"}` |
| **400** | Bad Request | Invalid config/unsupported OS | `"Not implemented on this OS"` |
| **401** | Unauthorized | Missing/invalid API key | `"Missing Gemini API key"` |
| **502** | Bad Gateway | Provider unreachable | `"gemini http 503"` |
| **500** | Server Error | Capture failure, unexpected error | `"Screen capture failed"` |

### Screen Capture Endpoint

#### Capture and Analyze (One-Shot)
```bash
POST /api/capture

Success Response (200):
{
  "output": "**Problem Identified:**\nSyntax error on line 42...\n\n**Solution:**\n```rust\nlet x = 5;\n```",
  "provider": "gemini",
  "model": "gemini-2.5-flash"
}

Error Response (401):
"Missing Gemini API key"

Error Response (502):
"gemini http 503"

Error Response (500):
"Screen capture failed"
```

**Implementation Details**:
1. Captures primary display (Windows only)
2. Converts BGRA → RGBA → PNG → Base64
3. Calls `sc_analyze_image()` with base64 image
4. Returns vision AI analysis

**Behavior:**
- Uses configured provider from `settings.json → sc.provider`
- Falls back to OpenRouter if `sc_fallback_or == true`
- System prompt sent only once per app session
- Groq mode uses OpenRouter with vision model

**Platform Support:**
- Windows: ✅ Supported (scrap crate)
- Linux/macOS: ❌ Not implemented

**Code**: `src/routes.rs:309-317`

### AI Endpoints

#### Screenshot-Aware Chat (Text Only)
```bash
POST /api/ai/sc
Content-Type: application/json

Body: {"input": "Explain async/await in Rust"}

Success Response (200):
{
  "output": "Async/await in Rust allows...",
  "provider": "groq",
  "model": "meta-llama/llama-4-scout-17b-16e-instruct"
}

Error Response (401):
"Missing Groq API key"

Error Response (502):
"groq http 503"
```

**Behavior:**
- Text-only queries (no image processing)
- Uses same fallback logic as English pipeline
- Falls back to OpenRouter if primary provider fails
- Supports all three providers: gemini, groq, openrouter

**Use Cases:**
- Follow-up questions after screen capture
- Context-aware queries without new screenshot
- General coding questions with SC-specific prompt

---

## 📊 State Management

### Prompt Tracking
- **Location**: `state.prompt_sent_sc` (Arc<Mutex<bool>>)
- **Purpose**: Ensure specialized screenshot prompt sent only once
- **Reset**: On app restart

### No Session State
Unlike English/Hindi, screenshot pipeline is **stateless**:
- No continuous audio capture
- No WebSocket connections
- One-shot request/response pattern

---

## 🔐 API Keys

Screenshot pipeline requires:

- **Gemini**: `secrets::get_key("gemini")` (for Gemini vision)
- **OpenRouter**: `secrets::get_key("openrouter")` (for Groq and fallbacks)

**Endpoints** (same as other pipelines):
```
POST   /api/providers/gemini/key
POST   /api/providers/openrouter/key
DELETE /api/providers/{name}/key
GET    /api/providers/state
```

---

## 🎛️ Model Selection Logic

### Priority Order (Vision)

1. **Screenshot-specific model** (if provider matches):
   ```json
   {"sc": {"provider": "gemini", "model": "gemini-2.5-flash"}}
   ```

2. **SC Provider default model**:
   ```json
   {"sc_providers": {"gemini": {"default_model": "gemini-2.5-flash"}}}
   ```

3. **Hardcoded fallback**: Provider's default

### Code Reference

**File**: `src/ai/mod.rs`  
**Function**: `choose_model()` with `lang="sc"`  
**Lines**: 23-25

---

## 📸 Screen Capture Implementation

### Platform Support
- **Windows**: ✅ Supported (using helper application pattern)
- **Linux/macOS**: ❌ Not implemented

### Architecture: Helper Application Pattern

Windows Services run in **Session 0** (non-interactive) and cannot access the user's desktop. To capture screens, we use a helper application pattern:

```
┌─────────────────────────────────────┐
│  Session 0 (Service)                │
│  ┌────────────────────────────┐     │
│  │  voxora-service.exe        │     │
│  │  Port: 8080                │     │
│  │  ❌ Cannot capture screen   │     │
│  └────────────────────────────┘     │
└─────────────────────────────────────┘
         │ HTTP GET :8081/capture
         ▼
┌─────────────────────────────────────┐
│  Session 1+ (User)                  │
│  ┌────────────────────────────┐     │
│  │  voxora-helper.exe         │     │
│  │  Port: 8081 (localhost)    │     │
│  │  ✅ Can capture screen      │     │
│  │  👻 Invisible (no UI)       │     │
│  └────────────────────────────┘     │
└─────────────────────────────────────┘
```

### Helper Binary

**File**: `src/bin/helper.rs`  
**Purpose**: Runs in user session to capture screens  
**Communication**: HTTP server on `127.0.0.1:8081`

**Features**:
- Invisible (console window hidden)
- Auto-starts on user login (Startup folder shortcut)
- Lightweight (~10-15 MB memory)
- Localhost-only (secure)

**Endpoints**:
```http
GET /capture  → Returns {image: "base64...", width: 1920, height: 1080}
GET /health   → Returns {status: "ok", version: "0.1.0"}
```

### Service Integration

**File**: `src/routes.rs`  
**Function**: `capture_via_helper()`

```rust
async fn capture_via_helper() -> Result<String, String> {
    // Call helper HTTP endpoint
    let response = reqwest::get("http://127.0.0.1:8081/capture")
        .await
        .map_err(|e| format!("Helper not responding: {}", e))?;
    
    // Parse JSON response
    let data: serde_json::Value = response.json().await
        .map_err(|e| format!("Invalid helper response: {}", e))?;
    
    // Extract base64 image
    let image = data["image"].as_str()
        .ok_or("No image in response")?;
    
    Ok(image.to_string())
}
```

### Helper Capture Process

**File**: `src/bin/helper.rs`

```rust
fn capture_png_base64() -> Result<String, String> {
    let display = Display::primary()?;
    let mut cap = Capturer::new(display)?;
    let (w, h) = (cap.width(), cap.height());
    
    // Wait for non-empty frame (up to 2s)
    let frame = loop {
        match cap.frame() {
            Ok(buf) if buf contains non-zero => break buf,
            Err(WouldBlock) => sleep(20ms),
            Err(e) => return Err(e),
        }
        if elapsed > 2s { break empty_frame; }
    };
    
    // Convert BGRA to RGBA
    let mut img = ImageBuffer::<Rgba<u8>>::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let idx = y*stride + x*4;
            let (b, g, r) = (frame[idx], frame[idx+1], frame[idx+2]);
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    
    // Encode PNG and Base64
    let png = PngEncoder::encode(img);
    Ok(base64::encode(&png))
}
```

### Installation

**Build**:
```powershell
cargo build --release --bins
```

Produces:
- `target/release/voxora-service.exe` (Service)
- `target/release/voxora-helper.exe` (Helper)

**Install** (via `install-service.ps1`):
1. Installs service to Session 0
2. Creates helper shortcut in Startup folder
3. Starts helper in Session 1
4. Helper auto-starts on future logins

### Why Helper Pattern?

| Approach | Works? | Reason |
|----------|--------|--------|
| Direct capture in service | ❌ | Session 0 has no desktop access |
| Named pipe to user app | ⚠️ | Complex, requires user interaction |
| **Helper HTTP server** | ✅ | Simple, secure, invisible |
| RPC/COM | ⚠️ | Overcomplicated for this use case |

**Benefits**:
- ✅ Works from Windows Service
- ✅ Completely invisible to user
- ✅ Auto-starts on login
- ✅ Simple HTTP communication
- ✅ Secure (localhost only)
- ✅ Fast (~200-500ms total latency)

---

## 🧠 Specialized System Prompt

The screenshot pipeline uses a detailed, task-oriented prompt:

```
You are ScreenCapture‑AI. You see only a screenshot. Provide the most 
relevant, actionable help with minimal words and zero fluff. Do not 
invent facts; if unsure, state what is missing.

If it's a coding context (IDE/editor/terminal/build logs/errors/questions):
•  Identify the problem or goal in 1–2 concise lines.
•  Root cause: 1–3 bullets, specific to the screenshot (no generic theory).
•  Solution: show the minimal, correct fix first.
   ◦  Include a small, self‑contained code example in a fenced block.
   ◦  If shell commands are needed, include them in one block.
   ◦  If config is involved, show only the changed lines or a unified diff.
•  Alternatives (max 2): when truly useful, list trade‑offs in one‑liners.
•  Wrap up with a 3–5 bullet summary.
•  Never paste long files; prefer patches, snippets, or exact lines to edit.

If it's general text/UI/doc/page:
•  4–6 bullet summary (key points only).
•  Top 3 recommended actions with concrete steps.

If information is insufficient:
•  Ask exactly one precise clarifying question at the end and stop.
```

**Location**: `data/settings.json → sc.prompt`  
**Lines**: 18 (settings.json)

---

## 🚀 Usage Example

### 1. Capture and Analyze Current Screen
```bash
curl -X POST http://localhost:8080/api/capture
```

Response:
```json
{
  "output": "**Problem Identified:**\nSyntax error on line 42: missing semicolon...\n\n**Root Cause:**\n• Rust requires semicolons...\n\n**Solution:**\n```rust\nlet x = 5; // Add semicolon here\n```\n",
  "provider": "gemini",
  "model": "gemini-2.5-flash"
}
```

### 2. Text-Based Screenshot Context Query
```bash
curl -X POST http://localhost:8080/api/ai/sc \
  -H "Content-Type: application/json" \
  -d '{"input": "Explain the error in my terminal"}'
```

---

## 🐛 Troubleshooting

### Issue: "Helper not responding" or capture timeouts

**Check if helper is running**:
```powershell
Get-Process voxora-helper
```

**Start helper manually**:
```powershell
cd C:\path\to\dist\voxora-portable
Start-Process .\voxora-helper.exe -WindowStyle Hidden
```

**Test helper directly**:
```powershell
curl http://127.0.0.1:8081/health
curl http://127.0.0.1:8081/capture | ConvertFrom-Json | Select width, height
```

### Issue: Helper not starting on login

**Check Startup folder**:
```powershell
$StartupFolder = [Environment]::GetFolderPath('Startup')
Get-ChildItem $StartupFolder | Where-Object Name -like "*Voxora*"
```

**Manually create shortcut**:
```powershell
$WScriptShell = New-Object -ComObject WScript.Shell
$Shortcut = $WScriptShell.CreateShortcut("$StartupFolder\Voxora Helper.lnk")
$Shortcut.TargetPath = "C:\path\to\voxora-helper.exe"
$Shortcut.WorkingDirectory = "C:\path\to\dist\voxora-portable"
$Shortcut.WindowStyle = 7  # Hidden
$Shortcut.Save()
```

### Issue: Port 8081 already in use

**Find conflicting process**:
```powershell
netstat -ano | findstr :8081
```

**Kill process**:
```powershell
Stop-Process -Id <PID>
```

### Issue: "Not implemented on this OS"
- **Platform**: Currently Windows-only
- **Solution**: Run on Windows or implement capture for your OS
- **Linux/macOS**: Would not need helper (can capture directly from service)

### Issue: Black/empty screenshots
- **Cause**: Display not ready or protected content (DRM)
- **Check**: Wait longer (2s timeout) or try different display
- **Verify**: Frame has non-zero pixels
- **Test**: Run helper manually and check output

### Issue: Vision API errors
- **Check**: API key for Gemini or OpenRouter exists
- **Verify**: Model supports vision (has `vision` in capabilities)
- **Test**: Try fallback model manually

### Issue: Fallback not triggering
- **Check**: `sc_fallback_or` is `true` in settings
- **Verify**: `sc_fallback_or_model` is set
- **Ensure**: OpenRouter API key exists

### Issue: Poor analysis quality
- **Check**: Screen resolution (too high = downscaled by API)
- **Verify**: Content is readable (not too small text)
- **Test**: Adjust system prompt for specific use case

---

## 📈 Performance Metrics

- **Helper HTTP Call**: ~5-10ms (localhost)
- **Capture Time**: ~50-200ms (Windows, 1080p display)
- **PNG Encoding**: ~100-300ms
- **Base64 Encoding**: ~50ms
- **API Latency**:
  - Gemini Vision: ~1-3s
  - OpenRouter Vision: ~2-5s (varies by model)
- **Total End-to-End**: ~2-6s

**Memory Usage**:
- Helper: ~10-15 MB (idle)
- Service: ~20-30 MB (idle)

**CPU Usage**:
- Helper: <1% (idle), 5-10% (capturing)
- Service: <1% (idle)

---

## 📦 Recent Updates (2025-10-27)

### Fixed Issues

1. **HTTP Status Code Standardization** ✅
   - Capture endpoint returns 401 for missing keys (was 500)
   - Text endpoint returns 401 for missing keys (was 400)
   - Consistent error handling across both endpoints

2. **Conditional Fallback Clarification** ✅
   - Image capture (`/api/capture`) has conditional fallback
   - Controlled by `sc_fallback_or` setting (default: true)
   - Text endpoint (`/api/ai/sc`) always has fallback

3. **Groq Vision Mode Documentation** ✅
   - Clarified Groq uses OpenRouter for vision
   - Requires OpenRouter API key, not Groq key
   - Provider reported as "groq-via-openrouter"

4. **Prompt Management Fix** ✅
   - System prompt sent correctly on first request
   - Subsequent requests don't resend prompt
   - Proper state management across sessions

### Breaking Changes

❌ None - All changes are backward compatible

### Migration Guide

No migration needed. Existing code continues to work:

```bash
# Existing endpoints (unchanged)
POST /api/capture  # Still works with conditional fallback
POST /api/ai/sc    # Still works with always fallback

# New settings (optional)
{
  "sc_fallback_or": true,  # Control image fallback
  "sc_fallback_or_model": "google/gemini-2.5-flash"
}
```

### Native Groq Vision Support (NEW 2025-10-27)

When using `sc.provider = "groq"` for screen capture:

```json
// Configuration
{
  "sc": {
    "provider": "groq",
    "model": "meta-llama/llama-4-scout-17b-16e-instruct"
  }
}
```

**Behavior:**
1. Uses **Groq API key** (native support)
2. Calls Groq's native vision API: `https://api.groq.com/openai/v1/chat/completions`
3. Uses model: `meta-llama/llama-4-scout-17b-16e-instruct` (natively multimodal)
4. Returns `provider: "groq"` in response
5. Falls back to OpenRouter if enabled and Groq fails

**Supported Models:**
- `meta-llama/llama-4-scout-17b-16e-instruct` ✅ (17B parameters, 16 experts)
  - **Input**: Text, images
  - **Output**: Text
  - **Speed**: ~750 tokens/second
  - **Documentation**: https://console.groq.com/docs/model/meta-llama/llama-4-scout-17b-16e-instruct

**Migration Note:**
Previous versions routed Groq through OpenRouter. Now uses native Groq API for better performance and correct API key usage.

---

## 🔍 Key Differences from Audio Pipelines

| Feature | English/Hindi | Screenshot |
|---------|---------------|------------|
| Input Type | Audio (PCM) | Image (PNG Base64) |
| Session State | Stateful | Stateless |
| WebSocket | Yes | No |
| Continuous | Yes | One-shot |
| VAD | Yes | No |
| Fallback Control | Always | Conditional (`sc_fallback_or`) for image |
| Provider Config | `providers` | `sc_providers` |
| Fallback Model | Auto-selected | Explicit (`sc_fallback_or_model`) |
| Vision Support | N/A | Required (for `/api/capture`) |
| Direct Endpoints | Yes (3 for EN, 2 for HI) | No |

---

## 🎨 Use Cases

### 1. Code Review
- Analyze IDE screenshots for bugs
- Suggest fixes for compile errors
- Review code quality

### 2. Debugging
- Interpret terminal error messages
- Analyze stack traces
- Debug UI issues

### 3. Learning
- Explain code snippets
- Interpret documentation
- Understand diagrams

### 4. Documentation
- Summarize UI screens
- Extract text from images
- Analyze flowcharts

---

### Related Documents

- [ENDPOINT_COMPARISON.md](./ENDPOINT_COMPARISON.md) - Complete endpoint comparison
- [01_ENGLISH_PIPELINE.md](./01_ENGLISH_PIPELINE.md) - English pipeline
- [02_HINDI_PIPELINE.md](./02_HINDI_PIPELINE.md) - Hindi pipeline
- [FIXED-ISSUES.md](../FIXED-ISSUES.md) - Complete list of fixes

---

**Last Updated**: 2025-10-27 (Helper Pattern Implementation)  
**Version**: 2.1  
**Platform**: Windows (Session 0/1 Architecture)

See also: [SCREEN_CAPTURE_HELPER.md](../SCREEN_CAPTURE_HELPER.md) for detailed helper implementation
