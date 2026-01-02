# API Endpoints Reference

## 📋 Overview

Complete reference for all HTTP and WebSocket endpoints in Voxora service.

**Base URL**: `http://localhost:8080`

---

## 🌐 UI Pages

### GET /
**Description**: Redirects to startup page  
**Response**: HTML

### GET /startup
**Description**: Initial startup/welcome page  
**Response**: HTML

### GET /home
**Description**: Landing page  
**Response**: HTML

### GET /app
**Description**: Main application page  
**Response**: HTML

### GET /mobile
**Description**: Mobile-friendly page with LAN IP addresses  
**Response**: HTML  
**Details**: Automatically detects and displays local network IPs

---

## 🎤 English Audio Pipeline

### POST /api/start
**Description**: Start English audio capture and transcription  
**Request**: No body  
**Response**: 
- `200 OK` - Started successfully
- `400 BAD_REQUEST` - Already running or error

### POST /api/stop
**Description**: Stop English audio capture  
**Request**: No body  
**Response**: `200 OK`

### GET /api/status
**Description**: Check if English audio capture is running  
**Response**:
```json
{
  "running": true
}
```

### GET /api/vad-status
**Description**: Get Voice Activity Detection statistics for English  
**Response**:
```json
{
  "sent": 123,
  "skipped": 45,
  "last_state": "speech"
}
```

### WebSocket /ws/transcript
**Description**: Real-time English transcript stream  
**Protocol**: WebSocket  
**Message Format**: `"EN: {transcript_text}"`  
**Example**:
```javascript
const ws = new WebSocket('ws://localhost:8080/ws/transcript');
ws.onmessage = (event) => {
  console.log(event.data); // "EN: Hello world"
};
```

---

## 🇮🇳 Hindi Audio Pipeline

### POST /api/start-hi
**Description**: Start Hindi audio capture and transcription  
**Request**: No body  
**Response**: 
- `200 OK` - Started successfully
- `400 BAD_REQUEST` - Already running or error

### POST /api/stop-hi
**Description**: Stop Hindi audio capture  
**Request**: No body  
**Response**: `200 OK`

### GET /api/status-hi
**Description**: Check if Hindi audio capture is running  
**Response**:
```json
{
  "running": false
}
```

### GET /api/vad-status-hi
**Description**: Get Voice Activity Detection statistics for Hindi  
**Response**:
```json
{
  "sent": 89,
  "skipped": 23,
  "last_state": "silence"
}
```

### WebSocket /ws/transcript-hi
**Description**: Real-time Hindi transcript stream  
**Protocol**: WebSocket  
**Message Format**: `"HI: {transcript_text}"`  
**Example**:
```javascript
const ws = new WebSocket('ws://localhost:8080/ws/transcript-hi');
ws.onmessage = (event) => {
  console.log(event.data); // "HI: नमस्ते"
};
```

---

## 🤖 AI Endpoints - English

### POST /api/ai/en
**Description**: AI chat with automatic provider selection (English)  
**Request**:
```json
{
  "input": "Explain async/await in Rust"
}
```
**Response**:
```json
{
  "output": "Async/await in Rust is...",
  "provider": "groq",
  "model": "llama-3.1-8b-instant"
}
```
**Status Codes**:
- `200 OK` - Success
- `400 BAD_REQUEST` - Invalid input or provider error

### POST /api/ai/en/groq
**Description**: Force Groq provider for English  
**Request**: Same as `/api/ai/en`  
**Response**: Same as `/api/ai/en`

### POST /api/ai/en/gemini
**Description**: Force Gemini provider for English  
**Request**: Same as `/api/ai/en`  
**Response**: Same as `/api/ai/en`

### POST /api/ai/en/openrouter
**Description**: Force OpenRouter provider for English  
**Request**: Same as `/api/ai/en`  
**Response**: Same as `/api/ai/en`

---

## 🤖 AI Endpoints - Hindi

### POST /api/ai/hi
**Description**: AI chat with automatic provider selection (Hindi)  
**Request**:
```json
{
  "input": "रस्ट में async/await क्या है?"
}
```
**Response**:
```json
{
  "output": "रस्ट में async/await...",
  "provider": "gemini",
  "model": "gemini-2.5-flash"
}
```

### POST /api/ai/hi/gemini
**Description**: Force Gemini provider for Hindi  
**Request**: Same as `/api/ai/hi`  
**Response**: Same as `/api/ai/hi`

### POST /api/ai/hi/openrouter
**Description**: Force OpenRouter provider for Hindi  
**Request**: Same as `/api/ai/hi`  
**Response**: Same as `/api/ai/hi`

---

## 🤖 AI Endpoints - Screenshot

### POST /api/ai/sc
**Description**: Screenshot-aware AI chat (text only)  
**Request**:
```json
{
  "input": "Explain this code pattern"
}
```
**Response**:
```json
{
  "output": "This code pattern shows...",
  "provider": "groq",
  "model": "meta-llama/llama-4-scout-17b-16e-instruct"
}
```

### POST /api/capture
**Description**: Capture screen and analyze with AI  
**Request**: No body (captures primary display)  
**Response**:
```json
{
  "output": "**Problem Identified:**\nSyntax error...",
  "provider": "gemini",
  "model": "gemini-2.5-flash"
}
```
**Platform**: Windows only  
**Status Codes**:
- `200 OK` - Success
- `500 INTERNAL_SERVER_ERROR` - Capture failed or not supported

---

## 🔐 Provider Key Management

### POST /api/providers/:name/key
**Description**: Save API key for a provider  
**Path Parameters**:
- `name`: Provider name (`groq`, `gemini`, `openrouter`, `custom`)

**Request**:
```json
{
  "api_key": "your-api-key-here"
}
```
**Response**:
- `200 OK` - Saved successfully
- `500 INTERNAL_SERVER_ERROR` - Keyring error

**Security**: Keys stored in OS-level secure keyring

### DELETE /api/providers/:name/key
**Description**: Delete API key for a provider  
**Path Parameters**:
- `name`: Provider name

**Response**:
- `200 OK` - Deleted successfully
- `500 INTERNAL_SERVER_ERROR` - Keyring error

### GET /api/providers/state
**Description**: Check which providers have API keys configured  
**Response**:
```json
{
  "groq": {
    "has_key": true
  },
  "gemini": {
    "has_key": true
  },
  "openrouter": {
    "has_key": false
  },
  "custom": {
    "has_key": false
  }
}
```

---

## ⚙️ Settings Management

### GET /api/settings
**Description**: Get current settings  
**Response**: Complete `settings.json` object
```json
{
  "en": { ... },
  "hi": { ... },
  "sc": { ... },
  "providers": { ... },
  "sc_providers": { ... },
  "fallback": { ... },
  ...
}
```

### POST /api/settings/en
**Description**: Update English language settings  
**Request**:
```json
{
  "provider": "groq",
  "model": "llama-3.1-8b-instant",
  "custom_model": null,
  "prompt": "You are a helpful assistant"
}
```
**Response**:
- `200 OK` - Updated successfully
- `500 INTERNAL_SERVER_ERROR` - Save failed

### POST /api/settings/hi
**Description**: Update Hindi language settings  
**Request**: Same structure as `/api/settings/en`

### POST /api/settings/sc
**Description**: Update screenshot settings  
**Request**: Same structure as `/api/settings/en`

### POST /api/settings/providers
**Description**: Update provider configurations (for text/audio)  
**Request**:
```json
{
  "groq": {
    "default_model": "llama-3.1-8b-instant",
    "extra_models": ""
  },
  "gemini": {
    "default_model": "gemini-2.5-flash",
    "extra_models": ""
  },
  "openrouter": {
    "default_model": "meta-llama/llama-3.1-70b",
    "extra_models": "qwen/qwen2-7b-instruct"
  }
}
```

### POST /api/settings/providers-sc
**Description**: Update screenshot provider configurations  
**Request**: Same structure as `/api/settings/providers`

### POST /api/settings/fallback
**Description**: Update fallback settings  
**Request**:
```json
{
  "openrouter_choice": "claude"
}
```
**Note**: `"claude"` maps to `anthropic/claude-3.5-sonnet`, others to `openai/gpt-4o-mini`

### POST /api/settings/sc-fallback
**Description**: Update screenshot fallback settings  
**Request**:
```json
{
  "or_fallback": true,
  "model": "google/gemini-2.5-flash"
}
```

---

## 🌐 Remote LLM (AnythingLLM)

### POST /api/remote/select
**Description**: Select and validate remote LLM server  
**Request**:
```json
{
  "server": "AnythingLLM Local",
  "url": "http://localhost:3001"
}
```
**Response**:
- `200 OK` + selected server object
- `400 BAD_REQUEST` - Invalid URL
- `502 BAD_GATEWAY` - Server not reachable

### DELETE /api/remote/select
**Description**: Clear remote LLM selection  
**Response**: `200 OK`

### GET /api/remote/status
**Description**: Check remote LLM status  
**Response**:
```json
{
  "selected": {
    "server": "AnythingLLM Local",
    "url": "http://localhost:3001"
  },
  "online": true
}
```

### POST /api/remote/key
**Description**: Save AnythingLLM API key  
**Request**:
```json
{
  "api_key": "your-anythingllm-key"
}
```

### DELETE /api/remote/key
**Description**: Delete AnythingLLM API key  
**Response**: `200 OK`

### POST /api/remote/test-auth
**Description**: Test AnythingLLM API key  
**Request**:
```json
{
  "url": "http://localhost:3001",
  "api_key": "your-key"
}
```
**Response**:
- `200 OK` - Valid key (auto-saved to keyring)
- `401 UNAUTHORIZED` - Invalid key
- `502 BAD_GATEWAY` - Server unreachable

### GET /api/remote/config
**Description**: Get remote LLM configuration  
**Response**:
```json
{
  "has_key": true,
  "slug": "my-workspace",
  "chat_default": true,
  "stream_default": false,
  "chat_mode": "query"
}
```

### POST /api/remote/config
**Description**: Update remote LLM configuration  
**Request**:
```json
{
  "slug": "my-workspace",
  "chat_default": true,
  "stream_default": false,
  "chat_mode": "chat"
}
```
**Note**: `chat_mode` can be `"chat"` or `"query"`

### GET /api/remote/workspace
**Description**: Get current workspace details  
**Response**:
```json
{
  "slug": "my-workspace",
  "name": "My Workspace"
}
```
**Status Codes**:
- `200 OK` - Workspace found
- `404 NOT_FOUND` - Workspace doesn't exist
- `401 UNAUTHORIZED` - Invalid API key
- `502 BAD_GATEWAY` - Server unreachable

### GET /api/remote/workspaces
**Description**: List all available workspaces  
**Response**:
```json
{
  "items": [
    {
      "slug": "workspace-1",
      "name": "Workspace One"
    },
    {
      "slug": "workspace-2",
      "name": null
    }
  ]
}
```

### POST /api/remote/ask
**Description**: Query AnythingLLM workspace  
**Request**:
```json
{
  "input": "What is in my documents?",
  "stream": false,
  "mode": "query"
}
```
**Response**:
```json
{
  "output": "Based on your documents...",
  "provider": "anythingllm",
  "info": {
    "mode": "query",
    "stream": false,
    "sources": [ ... ]
  }
}
```
**Modes**:
- `"chat"`: Standard chat (no RAG)
- `"query"`: Vector search + RAG (requires sources)

**Stream**:
- `true`: Server-Sent Events (SSE) format
- `false`: Standard JSON response

---

## 📁 Static Files

### GET /static/*
**Description**: Serve static files (CSS, JS, images)  
**Example**: `http://localhost:8080/static/style.css`

---

## 🔒 Security Notes

1. **API Keys**: All keys stored in OS keyring (not in files)
2. **CORS**: Permissive by default (configured via `CorsLayer`)
3. **No Authentication**: Service designed for local use
4. **Secrets in Logs**: Keys never logged or exposed

---

## 📊 Common Response Patterns

### Success Response
```json
{
  "output": "...",
  "provider": "...",
  "model": "..."
}
```

### Error Response
Plain text or JSON with error message
```
Status: 400 BAD_REQUEST
Body: "Error message here"
```

### Streaming Response (Remote Ask with stream=true)
```
Content-Type: text/event-stream

data: {"type":"textResponseChunk","textResponse":"Hello"}
data: {"type":"textResponseChunk","textResponse":" world"}
data: {"type":"finalizeResponseStream","response":"Hello world","sources":[...]}
```

---

## 🧪 Testing Endpoints

### Using curl

#### Start English audio
```bash
curl -X POST http://localhost:8080/api/start
```

#### AI query
```bash
curl -X POST http://localhost:8080/api/ai/en \
  -H "Content-Type: application/json" \
  -d '{"input":"What is Rust?"}'
```

#### Save API key
```bash
curl -X POST http://localhost:8080/api/providers/groq/key \
  -H "Content-Type: application/json" \
  -d '{"api_key":"gsk_..."}'
```

### Using JavaScript (fetch)

```javascript
// AI query
const response = await fetch('http://localhost:8080/api/ai/en', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ input: 'Explain closures' })
});
const data = await response.json();
console.log(data.output);

// WebSocket transcript
const ws = new WebSocket('ws://localhost:8080/ws/transcript');
ws.onmessage = (event) => {
  console.log('Transcript:', event.data);
};
```

---

**Last Updated**: 2025-10-26  
**Version**: 1.0
