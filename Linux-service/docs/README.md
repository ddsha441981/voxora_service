# Voxora Service Documentation

Welcome to the Voxora Service documentation! This folder contains comprehensive documentation for developers working with or extending the Voxora multilingual voice-to-text and AI assistant service.

---

## 📚 Documentation Index

### 🏗️ [00_ARCHITECTURE_OVERVIEW.md](./00_ARCHITECTURE_OVERVIEW.md)
**Start Here!** High-level system architecture, core components, data flow, and design principles.

**Contents**:
- System architecture diagram
- Core components (main server, router, state, config, secrets)
- Audio transcription and AI chat flows
- Project structure
- External dependencies
- Security features
- Logging strategy

**Audience**: All developers, new contributors

---

### 🇬🇧 [01_ENGLISH_PIPELINE.md](./01_ENGLISH_PIPELINE.md)
Complete guide to the English voice transcription and AI chat pipeline.

**Contents**:
- Audio capture flow (microphone → VAD → STT → WebSocket)
- AI chat flow with provider selection
- Configuration settings
- Fallback mechanisms (Groq/Gemini → OpenRouter)
- API endpoints
- State management
- Code references
- Troubleshooting

**Audience**: Developers working on English features, debugging audio issues

---

### 🇮🇳 [02_HINDI_PIPELINE.md](./02_HINDI_PIPELINE.md)
Complete guide to the Hindi voice transcription and AI chat pipeline.

**Contents**:
- Hindi-specific audio flow
- AI chat with Gemini/OpenRouter (Groq not supported)
- Multilingual considerations
- Fallback mechanisms
- API endpoints
- Devanagari script handling
- Key differences from English pipeline
- Troubleshooting Hindi-specific issues

**Audience**: Developers working on Hindi features, multilingual support

---

### 📸 [03_SCREENSHOT_PIPELINE.md](./03_SCREENSHOT_PIPELINE.md)
Complete guide to the screenshot capture and vision AI analysis pipeline.

**Contents**:
- Screen capture implementation (Windows helper pattern)
- Helper application architecture (Session 0/1)
- Vision AI flow with multimodal models
- BGRA→RGBA→PNG→Base64 conversion
- Conditional fallback logic
- Specialized ScreenCapture-AI prompt
- Use cases (code review, debugging, learning)
- Performance metrics
- Platform limitations

**Audience**: Developers working on screenshot features, vision AI integration

**Related**: [SCREEN_CAPTURE_HELPER.md](../SCREEN_CAPTURE_HELPER.md) - Helper implementation details

---

### 🔌 [04_API_ENDPOINTS.md](./04_API_ENDPOINTS.md)
Complete API reference for all HTTP and WebSocket endpoints.

**Contents**:
- UI pages (/, /home, /app, /mobile, /startup)
- Audio endpoints (start/stop, status, VAD, WebSocket)
- AI endpoints (English, Hindi, Screenshot)
- Provider key management
- Settings management
- Remote LLM (AnythingLLM) endpoints
- Request/response examples
- Testing with curl and JavaScript

**Audience**: API consumers, frontend developers, integration developers

---

### ⚙️ [05_CONFIGURATION_GUIDE.md](./05_CONFIGURATION_GUIDE.md)
Complete guide to configuring `data/settings.json` and environment variables.

**Contents**:
- Complete settings.json structure
- Language configurations (en, hi, sc)
- Provider configurations (groq, gemini, openrouter)
- Fallback configuration
- API key management (keyring)
- Environment variables
- Model selection priority
- Provider-specific model lists
- Configuration templates
- Best practices
- Troubleshooting

**Audience**: System administrators, deployment engineers, configuration managers

---

### 🔄 [06_FALLBACK_PIPELINE.md](./06_FALLBACK_PIPELINE.md)
Detailed fallback mechanism documentation for all pipelines.

**Contents**:
- Fallback pipeline for English
- Fallback pipeline for Hindi
- Fallback pipeline for Screenshot (conditional)
- Fallback functions (primary/alternative)
- Configuration details
- Key differences between pipelines
- Flow summary

**Audience**: Developers debugging fallback issues, reliability engineers

---

### 🌐 [07_ANYTHINGLLM_PIPELINE.md](./07_ANYTHINGLLM_PIPELINE.md)
Complete guide to the AnythingLLM remote pipeline with RAG capabilities.

**Contents**:
- Connection and setup flow
- Query modes (chat vs query/RAG)
- Vector search integration
- Streaming (Server-Sent Events)
- Workspace management
- API endpoints (11 endpoints)
- RAG implementation details
- Use cases and examples

**Audience**: Developers working on RAG features, document-based AI queries

---

## 🎯 Quick Navigation by Role

### New Developer
1. Start with **00_ARCHITECTURE_OVERVIEW.md**
2. Read relevant pipeline docs (01, 02, or 03)
3. Reference **04_API_ENDPOINTS.md** for API usage

### Frontend Developer
1. **04_API_ENDPOINTS.md** - API reference
2. **05_CONFIGURATION_GUIDE.md** - Understanding settings
3. **00_ARCHITECTURE_OVERVIEW.md** - System overview

### DevOps/Deployment
1. **05_CONFIGURATION_GUIDE.md** - Configuration management
2. **00_ARCHITECTURE_OVERVIEW.md** - System dependencies
3. **04_API_ENDPOINTS.md** - Health checks and status endpoints

### Feature Developer
1. Relevant pipeline doc (01, 02, or 03)
2. **06_FALLBACK_PIPELINE.md** - Reliability mechanisms
3. **00_ARCHITECTURE_OVERVIEW.md** - Integration points

### Bug Fixer/Troubleshooter
1. **06_FALLBACK_PIPELINE.md** - Fallback debugging
2. Relevant pipeline doc (01, 02, or 03)
3. **04_API_ENDPOINTS.md** - Testing endpoints

---

## 🔍 Quick Reference

### File Structure
```
docs/
├── README.md                      ← You are here
├── 00_ARCHITECTURE_OVERVIEW.md    ← System overview
├── 01_ENGLISH_PIPELINE.md         ← English voice/AI
├── 02_HINDI_PIPELINE.md           ← Hindi voice/AI
├── 03_SCREENSHOT_PIPELINE.md      ← Screenshot vision AI
├── 04_API_ENDPOINTS.md            ← API reference
├── 05_CONFIGURATION_GUIDE.md      ← Settings guide
├── 06_FALLBACK_PIPELINE.md        ← Fallback logic
├── 07_ANYTHINGLLM_PIPELINE.md     ← RAG/Remote LLM
└── ../SCREEN_CAPTURE_HELPER.md    ← Helper binary implementation
```

### Key Concepts

- **Pipeline**: End-to-end flow from input (audio/screen) to AI response
- **Provider**: AI service (Groq, Gemini, OpenRouter)
- **Fallback**: Automatic retry with different provider on failure
- **VAD**: Voice Activity Detection (filters silence)
- **Session**: Active audio capture state
- **Keyring**: OS-level secure storage for API keys
- **Settings**: Configuration in `data/settings.json`

### Common Patterns

```rust
// Get API key
secrets::get_key("groq")

// Choose model
choose_model("en", "groq", &settings)

// Fallback
fallback_to_openrouter_primary(state, "en", input, prompt)
```

---

## 📝 Documentation Standards

All documentation follows these standards:
- **Markdown**: CommonMark-compliant
- **Code Blocks**: Language-tagged with file paths when relevant
- **Diagrams**: ASCII art for architecture/flow
- **Examples**: Real curl/JavaScript examples
- **References**: Line numbers for code references
- **Version**: Dated with last update timestamp

---

## 🚀 Getting Started

### 1. Read Architecture Overview
```bash
# Open in your editor
code docs/00_ARCHITECTURE_OVERVIEW.md
```

### 2. Set Up Configuration
```bash
# Copy default settings
cp data/settings.json.example data/settings.json

# Edit settings
code data/settings.json
```

### 3. Add API Keys
```bash
curl -X POST http://localhost:8080/api/providers/groq/key \
  -H "Content-Type: application/json" \
  -d '{"api_key":"your-key-here"}'
```

### 4. Test Endpoints
```bash
# Check status
curl http://localhost:8080/api/status

# Test AI
curl -X POST http://localhost:8080/api/ai/en \
  -H "Content-Type: application/json" \
  -d '{"input":"Hello!"}'
```

---

## 🔗 External Resources

- **Groq API**: https://console.groq.com/
- **Gemini API**: https://ai.google.dev/
- **OpenRouter**: https://openrouter.ai/
- **AnythingLLM**: https://anythingllm.com/
- **Rust Docs**: https://doc.rust-lang.org/

---

## 📊 Documentation Maintenance

### When to Update Docs

- **New Feature**: Add to relevant pipeline doc + API endpoints
- **Breaking Change**: Update all affected docs + version number
- **Bug Fix**: Update troubleshooting sections
- **Configuration Change**: Update configuration guide

### Version History

|| Version | Date | Changes |
||---------|------|---------|
|| 1.1 | 2025-10-27 | Added helper binary docs for screen capture |
|| 1.0 | 2025-10-26 | Initial complete documentation |

---

## 🤝 Contributing to Documentation

When updating documentation:
1. Maintain consistent formatting
2. Include code examples
3. Update line number references if code changes
4. Test all curl examples
5. Update "Last Updated" timestamp
6. Bump version if major changes

---

## ❓ Need Help?

Can't find what you're looking for?

1. **Search**: Use `grep` or your editor's search across all docs
2. **Code**: Check source code directly (`src/`)
3. **API**: Test endpoints with curl/Postman
4. **Logs**: Check `logs/` directory for runtime info

---

**Documentation Version**: 1.1  
**Last Updated**: 2025-10-27  
**Maintained By**: Voxora Development Team
