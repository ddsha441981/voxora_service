# Configuration Guide

## 📋 Overview

Complete guide to configuring Voxora service via `data/settings.json` and environment variables.

---

## 📁 Configuration File Location

**Path**: `data/settings.json` (relative to executable)

- **Dev mode**: `voxora-service/data/settings.json`
- **Release mode**: `{exe_dir}/data/settings.json`

**Auto-loading**: Settings loaded at startup and can be modified via API

---

## 📝 Complete settings.json Structure

```json
{
  "en": {
    "provider": "groq",
    "model": "gemini-2.5-flash",
    "custom_model": null,
    "prompt": "You are helpful assistant and give answers with coding example "
  },
  "hi": {
    "provider": "gemini",
    "model": "gemini-2.5-flash",
    "custom_model": null,
    "prompt": "आप सहायक सहायक हैं"
  },
  "sc": {
    "provider": "groq",
    "model": "meta-llama/llama-4-scout-17b-16e-instruct",
    "custom_model": null,
    "prompt": "You are ScreenCapture‑AI. You see only a screenshot..."
  },
  "providers": {
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
      "extra_models": "qwen/qwen2-7b-instruct,nousresearch/hermes-3-llama-3.1-8b"
    }
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
  "fallback": {
    "openrouter_choice": "claude"
  },
  "sc_fallback_or": true,
  "sc_fallback_or_model": "google/gemini-2.5-flash",
  "remote": {
    "chat_default": true,
    "stream_default": false,
    "chat_mode": "query"
  }
}
```

---

## ⚙️ Language Configurations

### English (`en`)

```json
{
  "en": {
    "provider": "groq",           // Primary AI provider
    "model": "...",                // Override model (optional)
    "custom_model": null,          // Custom model name (future use)
    "prompt": "System prompt..."   // First-time system prompt
  }
}
```

**Fields**:
- `provider`: `"groq"` | `"gemini"` | `"openrouter"`
- `model`: Override the provider's default model (optional)
- `custom_model`: Reserved for future custom provider support
- `prompt`: System prompt sent only once per app session

**Default Provider**: `groq`

### Hindi (`hi`)

```json
{
  "hi": {
    "provider": "gemini",
    "model": "gemini-2.5-flash",
    "custom_model": null,
    "prompt": "आप सहायक सहायक हैं"
  }
}
```

**Supported Providers**: `"gemini"` | `"openrouter"`  
**Note**: Groq NOT supported for Hindi

**Default Provider**: `gemini`

### Screenshot (`sc`)

```json
{
  "sc": {
    "provider": "groq",
    "model": "meta-llama/llama-4-scout-17b-16e-instruct",
    "custom_model": null,
    "prompt": "You are ScreenCapture‑AI..."
  }
}
```

**Supported Providers**: `"gemini"` | `"groq"` | `"openrouter"`

**Default Provider**: `gemini`

**Recommended Prompt**: See full prompt in `03_SCREENSHOT_PIPELINE.md`

---

## 🔌 Provider Configurations

### Chat/Audio Providers (`providers`)

```json
{
  "providers": {
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
      "extra_models": "qwen/qwen2-7b-instruct,nousresearch/hermes-3-llama-3.1-8b"
    }
  }
}
```

**Fields**:
- `default_model`: Model used when language doesn't specify one
- `extra_models`: Comma-separated list (UI/future use)

### Screenshot Providers (`sc_providers`)

```json
{
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
  }
}
```

**Why Separate?**: Screenshot requires vision-capable models

---

## 🔄 Fallback Configuration

### Chat/Audio Fallback (`fallback`)

```json
{
  "fallback": {
    "openrouter_choice": "claude"
  }
}
```

**Mapping**:
- `"claude"` → `"anthropic/claude-3.5-sonnet"`
- Any other value → `"openai/gpt-4o-mini"`

**When Used**: When primary OpenRouter request fails

### Screenshot Fallback

```json
{
  "sc_fallback_or": true,
  "sc_fallback_or_model": "google/gemini-2.5-flash"
}
```

**Fields**:
- `sc_fallback_or`: Enable/disable OpenRouter fallback (boolean)
- `sc_fallback_or_model`: Specific model for fallback

**Key Difference**: Screenshot fallback is **conditional** (can be disabled)

---

## 🌐 Remote LLM Configuration (`remote`)

```json
{
  "remote": {
    "chat_default": true,
    "stream_default": false,
    "chat_mode": "query"
  }
}
```

**Fields**:
- `chat_default`: Use remote LLM by default (boolean)
- `stream_default`: Use streaming responses (boolean)
- `chat_mode`: `"chat"` (no RAG) | `"query"` (with RAG)

**Additional Config** (in keyring):
- `anythingllm`: API key
- `anythingllm_workspace`: Workspace slug

---

## 🔑 API Keys (Keyring)

API keys are **NOT** stored in `settings.json`. They're stored securely in the OS keyring.

### Supported Keys

| Key Name | Service | Required For |
|----------|---------|--------------|
| `groq` | Groq API | English pipeline, Screenshot (via OpenRouter) |
| `gemini` | Google Gemini | English/Hindi/Screenshot |
| `openrouter` | OpenRouter | All pipelines (fallback), Groq vision |
| `custom` | Custom provider | Future use |
| `anythingllm` | AnythingLLM | Remote LLM feature |
| `anythingllm_workspace` | AnythingLLM | Remote LLM workspace |

### Managing Keys

Via API:
```bash
# Save
curl -X POST http://localhost:8080/api/providers/groq/key \
  -H "Content-Type: application/json" \
  -d '{"api_key":"gsk_..."}'

# Delete
curl -X DELETE http://localhost:8080/api/providers/groq/key

# Check status
curl http://localhost:8080/api/providers/state
```

Via Code:
```rust
secrets::save_key("groq", "gsk_...");
secrets::get_key("groq");        // Option<String>
secrets::delete_key("groq");
secrets::has_key("groq");         // bool
```

---

## 🌍 Environment Variables

### `ENGLISH_GO_SERVER_URL`
- **Purpose**: WebSocket URL for English speech-to-text server
- **Default**: `ws://127.0.0.1:8085/ws`
- **Example**: `ENGLISH_GO_SERVER_URL=ws://192.168.1.100:8085/ws`

**Usage**:
```bash
# Windows PowerShell
$env:ENGLISH_GO_SERVER_URL="ws://custom-server:8085/ws"
cargo run

# Windows CMD
set ENGLISH_GO_SERVER_URL=ws://custom-server:8085/ws
cargo run

# Linux/macOS
export ENGLISH_GO_SERVER_URL=ws://custom-server:8085/ws
cargo run
```

---

## 📊 Model Selection Priority

### Algorithm

1. **Language-specific model** (if provider matches):
   - `en.provider == "groq"` AND `en.model != null` → Use `en.model`

2. **Provider default model**:
   - Use `providers.groq.default_model`

3. **Hardcoded fallback**:
   - Last resort (varies by provider)

### Example Scenarios

#### Scenario 1: Language Override
```json
{
  "en": {"provider": "groq", "model": "custom-model"},
  "providers": {"groq": {"default_model": "llama-3.1-8b-instant"}}
}
```
**Result**: Uses `"custom-model"`

#### Scenario 2: Provider Default
```json
{
  "en": {"provider": "groq", "model": null},
  "providers": {"groq": {"default_model": "llama-3.1-8b-instant"}}
}
```
**Result**: Uses `"llama-3.1-8b-instant"`

#### Scenario 3: Cross-Provider
```json
{
  "en": {"provider": "gemini", "model": "custom-model"},
  "providers": {"groq": {"default_model": "llama-3.1-8b-instant"}}
}
```
**Result**: Ignores `en.model` (provider doesn't match), uses Gemini's default

---

## 🎛️ Provider-Specific Model Lists

### Groq Models

**Popular Options**:
- `llama-3.1-8b-instant` (fastest)
- `llama-3.1-70b-versatile` (most capable)
- `mixtral-8x7b-32768` (long context)

**Vision** (via OpenRouter):
- `meta-llama/llama-4-scout-17b-16e-instruct`

### Gemini Models

**Popular Options**:
- `gemini-2.5-flash` (fastest, multimodal)
- `gemini-2.5-pro` (most capable)
- `gemini-1.5-flash` (older, still good)

**Vision**: All models support vision

### OpenRouter Models

**Popular Options**:
- `meta-llama/llama-3.1-70b` (general purpose)
- `anthropic/claude-3.5-sonnet` (strong reasoning)
- `openai/gpt-4o-mini` (fast, cheap)
- `google/gemini-2.5-flash` (multimodal)
- `qwen/qwen2-7b-instruct` (multilingual)

**Full List**: https://openrouter.ai/models

---

## 🔧 Configuration Best Practices

### 1. **Set Fallback Models**
Always configure OpenRouter models for reliability:
```json
{
  "providers": {
    "openrouter": {"default_model": "meta-llama/llama-3.1-70b"}
  },
  "fallback": {"openrouter_choice": "claude"}
}
```

### 2. **Use Vision-Capable Models for Screenshot**
```json
{
  "sc_providers": {
    "gemini": {"default_model": "gemini-2.5-flash"}
  },
  "sc_fallback_or": true,
  "sc_fallback_or_model": "google/gemini-2.5-flash"
}
```

### 3. **Customize System Prompts**
Tailor prompts to your use case:
```json
{
  "en": {
    "prompt": "You are a Rust expert. Answer with code examples."
  }
}
```

### 4. **Enable Remote LLM for RAG**
If using AnythingLLM for document-based queries:
```json
{
  "remote": {
    "chat_default": false,
    "stream_default": true,
    "chat_mode": "query"
  }
}
```

### 5. **Test Configuration**
After editing, verify:
```bash
curl http://localhost:8080/api/settings | jq .
```

---

## 🐛 Troubleshooting Configuration

### Issue: Settings not persisting
- **Check**: File permissions on `data/settings.json`
- **Verify**: Using API endpoints (POST /api/settings/*)
- **Test**: Manual edit → restart service

### Issue: Wrong model being used
- **Debug**: Check `choose_model()` logic in `ai/mod.rs:14-28`
- **Verify**: Provider matches language config
- **Test**: Force provider via explicit endpoint

### Issue: Fallback not working
- **Check**: OpenRouter API key exists
- **Verify**: `providers.openrouter.default_model` is set
- **Check**: `sc_fallback_or` is `true` (for screenshot)

### Issue: Invalid JSON
- **Validate**: Use JSON linter (jsonlint.com)
- **Check**: Comma placement, quote matching
- **Backup**: Keep known-good `settings.json`

---

## 📝 Configuration Templates

### Minimal (Gemini only)
```json
{
  "en": {"provider": "gemini", "model": null, "prompt": ""},
  "hi": {"provider": "gemini", "model": null, "prompt": ""},
  "sc": {"provider": "gemini", "model": null, "prompt": ""},
  "providers": {
    "gemini": {"default_model": "gemini-2.5-flash"}
  },
  "sc_providers": {
    "gemini": {"default_model": "gemini-2.5-flash"}
  },
  "fallback": {"openrouter_choice": "claude"},
  "sc_fallback_or": false
}
```

### Multi-Provider
```json
{
  "en": {"provider": "groq"},
  "hi": {"provider": "gemini"},
  "sc": {"provider": "openrouter"},
  "providers": {
    "groq": {"default_model": "llama-3.1-8b-instant"},
    "gemini": {"default_model": "gemini-2.5-flash"},
    "openrouter": {"default_model": "meta-llama/llama-3.1-70b"}
  },
  "sc_providers": {
    "openrouter": {"default_model": "google/gemini-2.5-flash"}
  },
  "fallback": {"openrouter_choice": "claude"},
  "sc_fallback_or": true,
  "sc_fallback_or_model": "google/gemini-2.5-flash"
}
```

---

## 🔄 Configuration Updates

### Via API (Recommended)
- **Automatic persistence**: Changes saved to `data/settings.json`
- **Validation**: Server validates JSON structure
- **Atomic**: Changes applied immediately

### Manual Edit
1. Stop service
2. Edit `data/settings.json`
3. Validate JSON
4. Restart service

**Warning**: Manual edits while service running will be overwritten

---

## 📚 Related Documentation

- **API Endpoints**: `04_API_ENDPOINTS.md` (settings endpoints)
- **English Pipeline**: `01_ENGLISH_PIPELINE.md` (model selection)
- **Hindi Pipeline**: `02_HINDI_PIPELINE.md` (provider restrictions)
- **Screenshot Pipeline**: `03_SCREENSHOT_PIPELINE.md` (vision models)
- **Fallback Logic**: `06_FALLBACK_PIPELINE.md` (detailed fallback flow)

---

**Last Updated**: 2025-10-26  
**Version**: 1.0
