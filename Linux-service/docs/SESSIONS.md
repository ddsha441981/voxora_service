# Session Management

## Overview

The session management system allows you to organize and persist your AI conversations and transcripts. All sessions are stored in a local SQLite database (`data/sessions.db`) for fast access and easy backup.

## Features

- **Create & Manage Sessions**: Organize conversations into separate sessions
- **Persistent Storage**: All messages are automatically saved to SQLite
- **Search**: Full-text search across session titles and message content
- **View History**: Review past conversations with full context
- **Delete Sessions**: Remove unwanted sessions (cannot be undone)
- **Current Session Tracking**: Set an active session for new messages

## API Endpoints

### Sessions

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/sessions` | GET | List all sessions (supports `?limit=N&offset=N`) |
| `/api/sessions` | POST | Create new session (`{"title": "My Session"}`) |
| `/api/sessions/:id` | GET | Get session with messages |
| `/api/sessions/:id` | PUT | Update session title |
| `/api/sessions/:id` | DELETE | Delete session |
| `/api/sessions/:id/messages` | GET | Get messages for session |
| `/api/sessions/:id/messages` | POST | Add message to session |
| `/api/sessions/current` | GET | Get current active session |
| `/api/sessions/current` | POST | Set current session |
| `/api/sessions/current` | DELETE | Clear current session |
| `/api/sessions/search` | GET | Search sessions (`?q=query&limit=N`) |

### Request/Response Examples

#### Create Session
```bash
curl -X POST http://localhost:8080/api/sessions \
  -H "Content-Type: application/json" \
  -d '{"title": "AI Debugging Session"}'
```

Response:
```json
{
  "id": "uuid-here",
  "title": "AI Debugging Session",
  "created_at": 1730000000,
  "updated_at": 1730000000,
  "message_count": 0
}
```

#### Add Message
```bash
curl -X POST http://localhost:8080/api/sessions/{session_id}/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "How do I fix this bug?",
    "role": "user",
    "language": "en",
    "provider": null,
    "model": null
  }'
```

#### Search Sessions
```bash
curl "http://localhost:8080/api/sessions/search?q=debugging&limit=10"
```

## UI Usage

1. **Open Sessions**: Click the 🗂️ Sessions button in the toolbar
2. **Create New**: Click "+ New Session" and enter a title
3. **View Session**: Click "View" on any session to load its messages
4. **Set Active**: Click on a session item to set it as current
5. **Search**: Type in the search box to find sessions
6. **Delete**: Click "Delete" button (confirmation required)

## Database Schema

### Tables

**sessions**
- `id` TEXT PRIMARY KEY
- `title` TEXT NOT NULL
- `created_at` INTEGER NOT NULL (Unix timestamp)
- `updated_at` INTEGER NOT NULL (Unix timestamp)

**messages**
- `id` TEXT PRIMARY KEY
- `session_id` TEXT NOT NULL (Foreign key)
- `timestamp` INTEGER NOT NULL (Unix timestamp)
- `role` TEXT NOT NULL ("user" | "assistant")
- `content` TEXT NOT NULL
- `language` TEXT (e.g., "en", "hi")
- `provider` TEXT (e.g., "groq", "gemini")
- `model` TEXT (e.g., "llama-3.1-8b")

## Architecture

The session management system is **completely isolated** from the main application:

```
src/sessions/
  ├── mod.rs       # Public API
  ├── models.rs    # Data structures
  ├── storage.rs   # SQLite operations
  ├── manager.rs   # Business logic
  └── routes.rs    # HTTP endpoints
```

### Key Design Decisions

- **Separate State**: Uses `SessionState` instead of `AppState`
- **Independent Database**: `data/sessions.db` separate from main app data
- **No Coupling**: Zero dependencies on transcription/capture logic
- **Pluggable**: Can be removed by deleting the `sessions/` folder

## Backup & Export

To backup your sessions:
```bash
# Copy the database file
cp data/sessions.db data/sessions_backup_$(date +%Y%m%d).db
```

The database is a single file that can be easily:
- Backed up
- Copied between machines
- Opened with SQLite tools for analysis
- Exported to JSON/CSV

## Future Enhancements

Potential improvements:
- [ ] Export sessions to JSON/Markdown
- [ ] Import sessions from files
- [ ] Session tags/categories
- [ ] Automatic session naming from first message
- [ ] Message editing
- [ ] Session sharing/collaboration
- [ ] Encryption for sensitive conversations

## Troubleshooting

**Sessions not loading?**
- Check that `data/sessions.db` exists
- Verify file permissions
- Check logs for SQLite errors

**Search not working?**
- SQLite FTS is not enabled (uses basic LIKE search)
- For better search, consider upgrading to FTS5

**Database locked?**
- Only one process should access the database
- Check for hanging connections
- Restart the service
