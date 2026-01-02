# Auto-Save Transcripts & AI Responses

## Overview

The session system **saves transcripts and AI responses ONLY when you click "Ask AI" button**. 

- ❌ Continuous transcripts are NOT saved
- ✅ Only the transcript you send to AI is saved
- ✅ AI's response is saved

## How It Works

### 1. **Ask AI Saves Transcript** 🎤
```
User speaks → Transcript appears → User clicks "Ask AI" → Question saved to session
```

**What gets saved:**
- ✅ English transcriptions (`language: "en"`)
- ✅ Hindi transcriptions (`language: "hi"`)  
- ✅ Timestamp
- ✅ Role: `user`

**Example flow:**
1. You speak: "Hello, how do I fix this bug?"
2. Transcript appears on screen (NOT saved yet)
3. You click "Ask AI" button
4. NOW it's saved to session as:
```json
{
  "role": "user",
  "content": "Hello, how do I fix this bug?",
  "language": "en",
  "timestamp": 1730000000
}
```

### 2. **AI Response Auto-Save** 🤖
```
AI generates response → Auto-saved to session
```

**What gets saved:**
- ✅ AI's answer
- ✅ Provider used (groq/gemini/openrouter)
- ✅ Model used (llama-3.1-8b/gemini-2.5-flash/etc)
- ✅ Language context
- ✅ Timestamp
- ✅ Role: `assistant`

**Example:**
```json
{
  "role": "assistant",
  "content": "To fix that bug, you need to check the memory allocation...",
  "language": "en",
  "provider": "groq",
  "model": "llama-3.1-8b-instant",
  "timestamp": 1730000001
}
```

## Complete Conversation Example

**Scenario**: You ask the AI a question in English

```
1. 🎤 You speak: "What is Rust ownership?"
   └─ Saved as: { role: "user", content: "What is Rust ownership?", language: "en" }

2. 🤖 AI responds: "Rust ownership is a memory management system..."
   └─ Saved as: { role: "assistant", content: "Rust ownership is...", language: "en", provider: "groq" }

3. 🎤 You speak: "Give me an example"
   └─ Saved as: { role: "user", content: "Give me an example", language: "en" }

4. 🤖 AI responds with code example
   └─ Saved as: { role: "assistant", content: "Here's an example...", language: "en", provider: "groq" }
```

All automatically saved to `data/sessions.db` ✅

## Session Management

### Current Session
- If NO session exists → Auto-creates "New Session"
- All transcripts/responses go to the "current" session
- You can switch sessions anytime via UI

### Workflow
```
1. Start service
2. Open /app UI
3. Start speaking (EN or HI) - transcript appears
4. Click "Ask AI" button - transcript + response saved to session
5. View history anytime via Sessions button 🗂️
```

## Architecture

### On-Demand Saving
No background listeners!

Saving happens ONLY when:
- User clicks "Ask AI" button
- AI endpoint is called

### AI Response Hook
Every AI endpoint now includes:
```rust
AI response → save to session with provider/model metadata
```

## What Gets Saved

| Event | Role | Content | Language | Provider | Model |
|-------|------|---------|----------|----------|-------|
| Voice transcript (EN) | user | "Hello AI" | en | - | - |
| Voice transcript (HI) | user | "नमस्ते" | hi | - | - |
| AI response (EN) | assistant | "Hello! How can I help?" | en | groq | llama-3.1-8b |
| AI response (HI) | assistant | "नमस्ते! मैं कैसे मदद कर सकता हूं?" | hi | gemini | gemini-2.5-flash |
| Screen capture query | user | "What's on my screen?" | en | - | - |
| SC AI response | assistant | "I see code editor..." | en | gemini | gemini-2.5-flash-vision |

## Benefits

✅ **Never lose conversations**: Everything persists across restarts  
✅ **Full context**: Review exactly what you said and AI responded  
✅ **Track AI usage**: See which providers/models answered what  
✅ **Bilingual history**: EN and HI in same database  
✅ **Zero effort**: No button clicks required  
✅ **Privacy**: All stored locally in SQLite  

## Viewing Your History

### Via UI
1. Click 🗂️ Sessions button
2. Select any session
3. Click "View"
4. See all transcripts + AI responses

### Via API
```bash
# Get current session
curl http://localhost:8080/api/sessions/current

# Get session with messages
curl http://localhost:8080/api/sessions/{session_id}?limit=100

# Search
curl "http://localhost:8080/api/sessions/search?q=ownership"
```

## Database Structure

**sessions** table:
- Tracks conversation metadata (title, timestamps, message count)

**messages** table:
- Stores each transcript and AI response
- Links to parent session via `session_id`
- Includes all metadata (language, provider, model)

## Configuration

No configuration needed! It's always on.

To disable:
- Comment out the listener startup in `main.rs`
- Comment out the save calls in AI endpoints

## Performance

- ✅ **Async**: Saving happens in background (non-blocking)
- ✅ **Fast**: SQLite writes are microseconds
- ✅ **Efficient**: Only saves non-empty content
- ✅ **Reliable**: Uses transactions for data integrity

## Troubleshooting

**Not saving transcripts?**
- Check that go-server is running and connected
- Verify WebSocket connection in UI (green dot)
- Check logs for transcript broadcasts

**Not saving AI responses?**
- Verify AI endpoint returns successfully
- Check session_manager is set in AppState
- Look for errors in console

**Database errors?**
- Check `data/sessions.db` exists and is writable
- Verify no permission issues
- Restart service if database locked

## Privacy & Data

- **Local only**: Nothing leaves your machine
- **Encrypted**: Can encrypt DB file if needed
- **Deletable**: Delete sessions anytime
- **Portable**: Copy DB file to backup/move data

---

**In Summary**: Speak freely! Only when you click "Ask AI" will that specific question and answer be saved to session history. 💾🎉
