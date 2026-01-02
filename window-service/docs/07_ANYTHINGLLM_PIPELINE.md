# AnythingLLM Remote Pipeline

## 📋 Overview

The AnythingLLM pipeline provides RAG (Retrieval-Augmented Generation) capabilities by connecting to a self-hosted or remote AnythingLLM instance. It enables document-based querying with vector search and workspace management.

---

## 🏗️ Pipeline Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                ANYTHINGLLM REMOTE PIPELINE                      │
└────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                  CONNECTION FLOW                              │
└──────────────────────────────────────────────────────────────┘

    User Configuration
          │
          ▼
    POST /api/remote/select
          │
          ├──> Validate URL (http://localhost:3001)
          ├──> Health check (GET base URL)
          │
          ▼
    ┌─────────────────┐
    │  Store Remote   │
    │  Selection      │ ──> state.remote_selected
    └─────────────────┘
          │
          ▼
    POST /api/remote/key
          │
          ├──> Store API key in keyring ("anythingllm")
          │
          ▼
    POST /api/remote/test-auth
          │
          ├──> Test: GET /api/v1/workspaces
          ├──> Validate API key
          │
          ▼
    ┌─────────────────┐
    │  Get Workspaces │
    └─────────────────┘
          │
          ▼
    POST /api/remote/config
          │
          ├──> Set workspace slug
          ├──> Set chat_mode ("chat" or "query")
          ├──> Set defaults (chat_default, stream_default)
          │
          ▼
    ┌─────────────────┐
    │  Ready to Query │
    └─────────────────┘


┌──────────────────────────────────────────────────────────────┐
│                    QUERY FLOW (Chat Mode)                     │
└──────────────────────────────────────────────────────────────┘

    User Input
          │
          ▼
    POST /api/remote/ask
          │
          ├──> mode: "chat"
          ├──> stream: false/true
          │
          ▼
    ┌─────────────────────────┐
    │  AnythingLLM API        │
    │  POST /api/v1/workspace/│
    │       {slug}/chat       │
    └─────────────────────────┘
          │
          │ {"message": "user input"}
          ▼
    ┌─────────────────────────┐
    │  LLM Response           │
    │  (No RAG, direct chat)  │
    └─────────────────────────┘
          │
          ▼
    Return: {output, provider, info}


┌──────────────────────────────────────────────────────────────┐
│                   QUERY FLOW (Query Mode - RAG)               │
└──────────────────────────────────────────────────────────────┘

    User Input
          │
          ▼
    POST /api/remote/ask
          │
          ├──> mode: "query"
          ├──> stream: false/true
          │
          ▼
    ┌─────────────────────────┐
    │  Step 1: Vector Search  │
    │  POST /api/v1/workspace/│
    │       {slug}/vector-    │
    │       search            │
    └─────────────────────────┘
          │
          │ {"query": "user input"}
          ▼
    ┌─────────────────────────┐
    │  Check: Has Results?    │
    └─────────────────────────┘
          │
          ├── No Results ──> Return "No relevant sources found."
          │
          ├── Has Results
          │         │
          │         ▼
          │   ┌─────────────────────────┐
          │   │  Step 2: Chat with RAG  │
          │   │  POST /api/v1/workspace/│
          │   │       {slug}/chat       │
          │   └─────────────────────────┘
          │         │
          │         │ Uses vector search results
          │         ▼
          │   ┌─────────────────────────┐
          │   │  LLM Response + Sources │
          │   └─────────────────────────┘
          │         │
          └─────────┘
                    │
                    ▼
          Return: {output, provider, info: {sources}}


┌──────────────────────────────────────────────────────────────┐
│                    STREAMING FLOW (SSE)                       │
└──────────────────────────────────────────────────────────────┘

    POST /api/remote/ask (stream: true)
          │
          ▼
    ┌─────────────────────────┐
    │  AnythingLLM API        │
    │  POST /api/v1/workspace/│
    │       {slug}/stream-chat│
    └─────────────────────────┘
          │
          │ Server-Sent Events (SSE)
          ▼
    ┌──────────────────────────────────────────┐
    │  Event Stream:                           │
    │  data: {"type":"textResponseChunk",...}  │
    │  data: {"type":"textResponseChunk",...}  │
    │  data: {"type":"finalizeResponseStream"} │
    └──────────────────────────────────────────┘
          │
          │ Parse SSE lines
          ▼
    ┌─────────────────────────┐
    │  Accumulate Chunks      │
    │  Extract Final Response │
    │  Extract Sources        │
    └─────────────────────────┘
          │
          ▼
    Return: {output, provider, info: {stream:true, sources}}
```

---

## ⚙️ Configuration

### Settings Location
- **File**: `data/settings.json` → `remote` section
- **Keyring**: `anythingllm` (API key), `anythingllm_workspace` (workspace slug)

### Remote Settings Structure

```json
{
  "remote": {
    "chat_default": true,          // Use remote by default
    "stream_default": false,        // Use streaming responses
    "chat_mode": "query"            // "chat" or "query" (RAG)
  }
}
```

### State Storage

**In AppState** (`state.rs`):
```rust
pub struct AppState {
    pub remote_selected: Arc<Mutex<Option<RemoteSelection>>>,
    // ...
}

pub struct RemoteSelection {
    pub server: String,      // "AnythingLLM Local"
    pub url: String,         // "http://localhost:3001"
}
```

**In Keyring** (`secrets.rs`):
- `anythingllm`: API key (Bearer token)
- `anythingllm_workspace`: Workspace slug

---

## 🔌 API Endpoints

### Setup & Configuration

#### 1. Select Remote Server
```bash
POST /api/remote/select
Content-Type: application/json

{
  "server": "AnythingLLM Local",
  "url": "http://localhost:3001"
}
```

**Response**:
- `200 OK` + server object (if reachable)
- `400 BAD_REQUEST` - Invalid URL format
- `502 BAD_GATEWAY` - Server not reachable

**Implementation**: `src/routes.rs:591-612`

#### 2. Clear Remote Selection
```bash
DELETE /api/remote/select
```

**Response**: `200 OK`

#### 3. Check Remote Status
```bash
GET /api/remote/status
```

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

**Implementation**: `src/routes.rs:620-629`

### API Key Management

#### 4. Save API Key
```bash
POST /api/remote/key
Content-Type: application/json

{
  "api_key": "ANYLL-xxxxx"
}
```

**Response**: `200 OK` or `500 INTERNAL_SERVER_ERROR`

**Implementation**: `src/routes.rs:109-114`

#### 5. Delete API Key
```bash
DELETE /api/remote/key
```

**Response**: `200 OK`

**Implementation**: `src/routes.rs:115-120`

#### 6. Test Authentication
```bash
POST /api/remote/test-auth
Content-Type: application/json

{
  "url": "http://localhost:3001",
  "api_key": "ANYLL-xxxxx"
}
```

**Response**:
- `200 OK` + `{"ok": true}` - Valid key (auto-saved to keyring)
- `401 UNAUTHORIZED` - Invalid API key
- `502 BAD_GATEWAY` - Server unreachable

**Implementation**: `src/routes.rs:636-653`

### Configuration Management

#### 7. Get Remote Config
```bash
GET /api/remote/config
```

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

**Implementation**: `src/routes.rs:82-87`

#### 8. Update Remote Config
```bash
POST /api/remote/config
Content-Type: application/json

{
  "slug": "my-workspace",           // Optional: workspace slug
  "chat_default": true,              // Optional: use remote by default
  "stream_default": false,           // Optional: use streaming
  "chat_mode": "query"               // Optional: "chat" or "query"
}
```

**Response**: `200 OK` or `500 INTERNAL_SERVER_ERROR`

**Notes**:
- `slug` saved to keyring (`anythingllm_workspace`)
- Other settings saved to `data/settings.json`

**Implementation**: `src/routes.rs:89-105`

### Workspace Management

#### 9. Get Current Workspace
```bash
GET /api/remote/workspace
```

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

**Implementation**: `src/routes.rs:124-140`

#### 10. List All Workspaces
```bash
GET /api/remote/workspaces
```

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
      "name": "Workspace Two"
    }
  ]
}
```

**Implementation**: `src/routes.rs:144-165`

### Query Endpoints

#### 11. Ask Question (Chat or Query)
```bash
POST /api/remote/ask
Content-Type: application/json

{
  "input": "What is in my documents?",
  "stream": false,                    // Optional: use streaming
  "mode": "query"                     // Optional: "chat" or "query"
}
```

**Response**:
```json
{
  "output": "Based on your documents, ...",
  "provider": "anythingllm",
  "info": {
    "mode": "query",
    "stream": false,
    "sources": [
      {
        "title": "doc.pdf",
        "chunk": "relevant text..."
      }
    ]
  }
}
```

**Implementation**: `src/routes.rs:173-276`

---

## 🔄 Query Modes

### Chat Mode (`"chat"`)

**Behavior**: Direct conversation with LLM, no document retrieval

**Use Cases**:
- General questions
- Follow-up questions
- Brainstorming

**API Flow**:
```
POST /api/v1/workspace/{slug}/chat
→ Direct to LLM
→ Return response
```

**Example**:
```bash
curl -X POST http://localhost:8080/api/remote/ask \
  -H "Content-Type: application/json" \
  -d '{
    "input": "What is Rust?",
    "mode": "chat"
  }'
```

### Query Mode (`"query"` - RAG)

**Behavior**: Vector search + document-augmented response

**Use Cases**:
- Document-based questions
- Finding information in uploaded files
- Context-aware responses

**API Flow**:
```
1. POST /api/v1/workspace/{slug}/vector-search
   → Search documents
   → Get relevant chunks

2. If results found:
   POST /api/v1/workspace/{slug}/chat
   → LLM with document context
   → Return response + sources

3. If no results:
   → Return "No relevant sources found."
```

**Example**:
```bash
curl -X POST http://localhost:8080/api/remote/ask \
  -H "Content-Type: application/json" \
  -d '{
    "input": "What does the documentation say about API keys?",
    "mode": "query"
  }'
```

**Response**:
```json
{
  "output": "According to the documentation, API keys should be...",
  "provider": "anythingllm",
  "info": {
    "mode": "query",
    "stream": false,
    "sources": [
      {
        "title": "API_DOCS.md",
        "chunk": "API keys are stored in..."
      }
    ]
  }
}
```

---

## 🌊 Streaming (Server-Sent Events)

### Enable Streaming

```bash
POST /api/remote/ask
{
  "input": "Explain async/await",
  "stream": true,
  "mode": "chat"
}
```

### SSE Event Format

AnythingLLM sends events in this format:
```
data: {"type":"textResponseChunk","textResponse":"Hello"}
data: {"type":"textResponseChunk","textResponse":" world"}
data: {"type":"finalizeResponseStream","response":"Hello world","sources":[...]}
```

### Parsing Logic

**File**: `src/routes.rs:208-235`

```rust
// Parse SSE lines
for line in body.lines() {
    if let Some(rest) = line.trim_start().strip_prefix("data:") {
        let js = rest.trim();
        if let Ok(v) = serde_json::from_str::<Value>(js) {
            match v.get("type").and_then(|x| x.as_str()) {
                Some("textResponseChunk") => {
                    // Accumulate text
                    if let Some(s) = v.get("textResponse").and_then(|x| x.as_str()) {
                        output.push_str(s);
                    }
                }
                Some("finalizeResponseStream") => {
                    // Extract sources and final text
                    if let Some(s) = v.get("sources").cloned() {
                        sources = Some(s);
                    }
                    if let Some(s) = v.get("response").and_then(|x| x.as_str()) {
                        finalize_text = Some(s.to_string());
                    }
                }
                _ => {}
            }
        }
    }
}
```

### Frontend Integration

```javascript
const response = await fetch('http://localhost:8080/api/remote/ask', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    input: 'Explain Rust',
    stream: true,
    mode: 'query'
  })
});

const data = await response.json();
console.log(data.output); // Full accumulated response
console.log(data.info.sources); // Document sources
```

**Note**: Voxora accumulates the stream server-side and returns complete response

---

## 🎯 Code Reference

### Main Query Handler

**File**: `src/routes.rs`  
**Function**: `remote_ask()`  
**Lines**: 173-276

```rust
pub async fn remote_ask(State(state): State<AppState>, Json(req): Json<RemoteAskReq>) 
    -> impl IntoResponse 
{
    // 1. Validate selection
    let base = state.remote_selected.lock().await.as_ref()
        .map(|s| s.url.clone())
        .ok_or_else(|| "no remote selected")?;
    
    // 2. Get credentials
    let key = secrets::get_key("anythingllm")?;
    let slug = secrets::get_key("anythingllm_workspace")?;
    
    // 3. Query mode: vector search first
    if mode.eq_ignore_ascii_case("query") {
        let vs_url = format!("{}/api/v1/workspace/{}/vector-search", api, slug);
        let payload = json!({"query": req.input});
        let response = client.post(&vs_url)
            .header("Authorization", format!("Bearer {}", key))
            .json(&payload)
            .send().await?;
        
        // Check if we have hits
        let has_hits = response.json::<Value>().await?
            .get("results").and_then(|x| x.as_array())
            .map(|a| !a.is_empty()).unwrap_or(false);
        
        if !has_hits {
            return Json(RemoteAskResp {
                output: "No relevant sources found.".into(),
                provider: "anythingllm".into(),
                info: json!({"mode":"query","vector_hits":0})
            });
        }
    }
    
    // 4. Chat call (with or without streaming)
    let chat_path = if stream { "stream-chat" } else { "chat" };
    let chat_url = format!("{}/api/v1/workspace/{}/{}", api, slug, chat_path);
    let payload = json!({"message": req.input});
    
    let response = client.post(&chat_url)
        .header("Authorization", format!("Bearer {}", key))
        .json(&payload)
        .send().await?;
    
    // 5. Parse response (streaming or regular)
    // ... (see implementation for full parsing logic)
}
```

---

## 📊 State Management

### Remote Selection State

**Location**: `state.remote_selected` (Arc<Mutex<Option<RemoteSelection>>>)

**Structure**:
```rust
pub struct RemoteSelection {
    pub server: String,    // Display name
    pub url: String,       // Base URL
}
```

**Lifecycle**:
- Set via `POST /api/remote/select`
- Cleared via `DELETE /api/remote/select`
- Persists for app lifetime (not saved to disk)

### Configuration State

**In `settings.json`**:
```json
{
  "remote": {
    "chat_default": true,
    "stream_default": false,
    "chat_mode": "query"
  }
}
```

**In Keyring**:
- `anythingllm`: API key
- `anythingllm_workspace`: Active workspace slug

---

## 🔐 Security

### API Key Storage
- **Never** stored in `settings.json`
- Stored in OS keyring via `secrets::save_key("anythingllm", key)`
- Retrieved securely at runtime

### Authentication Flow
```
Client → Voxora → AnythingLLM
         (with Bearer token from keyring)
```

### Network Security
- HTTPS supported (configure URL with `https://`)
- Timeout: 3-15s (varies by endpoint)
- No proxy configuration (direct connection)

---

## 🚀 Usage Examples

### Complete Setup Flow

```bash
# 1. Select remote server
curl -X POST http://localhost:8080/api/remote/select \
  -H "Content-Type: application/json" \
  -d '{
    "server": "AnythingLLM Local",
    "url": "http://localhost:3001"
  }'

# 2. Test and save API key
curl -X POST http://localhost:8080/api/remote/test-auth \
  -H "Content-Type: application/json" \
  -d '{
    "url": "http://localhost:3001",
    "api_key": "ANYLL-xxxxx"
  }'

# 3. List workspaces
curl http://localhost:8080/api/remote/workspaces

# 4. Set workspace and mode
curl -X POST http://localhost:8080/api/remote/config \
  -H "Content-Type: application/json" \
  -d '{
    "slug": "my-docs",
    "chat_mode": "query",
    "stream_default": false
  }'

# 5. Query documents
curl -X POST http://localhost:8080/api/remote/ask \
  -H "Content-Type: application/json" \
  -d '{
    "input": "What is in my documentation?",
    "mode": "query"
  }'
```

### Chat Mode Example

```bash
curl -X POST http://localhost:8080/api/remote/ask \
  -H "Content-Type: application/json" \
  -d '{
    "input": "Tell me a joke",
    "mode": "chat"
  }'
```

Response:
```json
{
  "output": "Why did the developer go broke? Because he used up all his cache!",
  "provider": "anythingllm",
  "info": {
    "mode": "chat",
    "stream": false
  }
}
```

### Query Mode with Sources

```bash
curl -X POST http://localhost:8080/api/remote/ask \
  -H "Content-Type: application/json" \
  -d '{
    "input": "How do I configure the API?",
    "mode": "query"
  }'
```

Response:
```json
{
  "output": "To configure the API, you need to set up your settings.json file...",
  "provider": "anythingllm",
  "info": {
    "mode": "query",
    "stream": false,
    "sources": [
      {
        "title": "CONFIGURATION_GUIDE.md",
        "chunk": "## API Configuration\n\nSettings are stored in...",
        "score": 0.89
      }
    ]
  }
}
```

---

## 🐛 Troubleshooting

### Issue: "no remote selected"
**Cause**: Remote server not configured  
**Solution**: 
```bash
POST /api/remote/select
```

### Issue: "Missing OpenRouter API key" (wrong error message)
**Cause**: Actually missing AnythingLLM API key  
**Solution**:
```bash
POST /api/remote/key
{"api_key": "ANYLL-xxxxx"}
```

### Issue: "no slug"
**Cause**: Workspace slug not configured  
**Solution**:
```bash
POST /api/remote/config
{"slug": "my-workspace"}
```

### Issue: "workspace not found" (404)
**Cause**: Invalid workspace slug  
**Solution**: List workspaces and use valid slug
```bash
GET /api/remote/workspaces
```

### Issue: "remote unreachable" (502)
**Cause**: AnythingLLM server not running or URL incorrect  
**Solution**:
1. Verify AnythingLLM is running: `curl http://localhost:3001`
2. Check URL in remote selection
3. Check firewall/network

### Issue: "No relevant sources found."
**Cause**: Vector search returned no results  
**Solution**:
1. Check if documents are uploaded to workspace
2. Try different query phrasing
3. Use `"mode": "chat"` for general questions
4. Re-embed documents in AnythingLLM

### Issue: Streaming not working
**Cause**: AnythingLLM may return SSE in non-stream mode  
**Solution**: Voxora handles both formats automatically (lines 238-256 in routes.rs)

---

## 📈 Performance Metrics

- **Connection Test**: ~100-500ms
- **Auth Test**: ~200-1000ms
- **Vector Search**: ~500-2000ms (depends on document count)
- **Chat (no RAG)**: ~1-5s (depends on LLM)
- **Query (with RAG)**: ~2-7s (vector search + LLM)
- **Streaming**: Real-time chunks, similar total time

---

## 🔍 Key Differences from AI Pipelines

| Feature | AI Pipelines | AnythingLLM |
|---------|--------------|-------------|
| Provider | Groq/Gemini/OpenRouter | AnythingLLM (self-hosted) |
| RAG Support | No | Yes (query mode) |
| Vector Search | No | Yes |
| Document Context | No | Yes |
| Streaming | No | Yes (SSE) |
| Workspace | No | Yes |
| API Key Storage | Keyring | Keyring |
| Fallback | Yes | No |

---

## 🎨 Use Cases

### 1. **Document Q&A**
Upload documentation and query with RAG:
```json
{
  "input": "How do I deploy this service?",
  "mode": "query"
}
```

### 2. **Code Documentation Search**
Upload codebase docs and search:
```json
{
  "input": "What does the fallback_to_openrouter function do?",
  "mode": "query"
}
```

### 3. **Meeting Notes Analysis**
Upload meeting transcripts and extract insights:
```json
{
  "input": "What were the action items from last week's meeting?",
  "mode": "query"
}
```

### 4. **Knowledge Base Assistant**
Create internal knowledge base with RAG:
```json
{
  "input": "What is our policy on API rate limiting?",
  "mode": "query"
}
```

### 5. **General Chat (No RAG)**
Use as general LLM without documents:
```json
{
  "input": "Explain async/await in simple terms",
  "mode": "chat"
}
```

---

## 🔗 AnythingLLM Setup

### Prerequisites
- AnythingLLM instance running (local or remote)
- API key generated in AnythingLLM settings
- At least one workspace created
- Documents uploaded (for query mode)

### AnythingLLM Endpoints Used

| Endpoint | Purpose |
|----------|---------|
| `GET /api/v1/workspaces` | List all workspaces |
| `GET /api/v1/workspace/{slug}` | Get workspace details |
| `POST /api/v1/workspace/{slug}/vector-search` | Search documents |
| `POST /api/v1/workspace/{slug}/chat` | Chat with LLM |
| `POST /api/v1/workspace/{slug}/stream-chat` | Streaming chat |

### Recommended AnythingLLM Settings
- **Embedding Model**: `text-embedding-3-small` (OpenAI) or similar
- **LLM**: GPT-4, Claude, or local model
- **Chunk Size**: 1000-2000 tokens
- **Chunk Overlap**: 100-200 tokens

---

## 📚 Related Documentation

- **API Endpoints**: `04_API_ENDPOINTS.md` (remote endpoints)
- **Configuration**: `05_CONFIGURATION_GUIDE.md` (remote settings)
- **Architecture**: `00_ARCHITECTURE_OVERVIEW.md` (remote LLM section)

---

## 🔮 Future Enhancements

- [ ] Multi-workspace support (switch between workspaces)
- [ ] Document upload via Voxora API
- [ ] Embedding management
- [ ] Workspace creation/deletion
- [ ] Chat history persistence
- [ ] Agent mode support
- [ ] Custom prompts per workspace

---

**Last Updated**: 2025-10-26  
**Version**: 1.0
