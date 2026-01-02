# API Endpoints Comparison & Reference

**Last Updated**: 2025-10-27 (Post-Fix)  
**Version**: 2.0 (After Issue Fixes)

This document provides a comprehensive comparison of all API endpoints across English, Hindi, and Screenshot pipelines after the recent fixes.

---

## 📊 Quick Reference Table

| Feature | English | Hindi | Screen Capture |
|---------|---------|-------|----------------|
| **Auto Endpoint** | ✅ `/api/ai/en` | ✅ `/api/ai/hi` | ✅ `/api/ai/sc` (text)<br>✅ `/api/capture` (image) |
| **Direct Endpoints** | ✅ Yes (3) | ✅ Yes (2) | ❌ No |
| **Fallback (Auto)** | ✅ Always | ✅ Always | ✅ Always (text)<br>⚠️ Conditional (image) |
| **Fallback (Direct)** | ❌ Never | ❌ Never | N/A |
| **Default Provider** | Groq | Gemini | Gemini |
| **Groq Support** | ✅ Yes | ❌ No | ✅ Yes (native vision) |
| **Gemini Support** | ✅ Yes | ✅ Yes | ✅ Yes (vision) |
| **OpenRouter Support** | ✅ Yes | ✅ Yes | ✅ Yes (vision) |
| **Vision Support** | ❌ No | ❌ No | ✅ Yes (required) |
| **Status Code Fix** | ✅ Fixed | ✅ Fixed | ✅ Fixed |
| **Prompt Management** | ✅ Fixed | ✅ Fixed | ✅ Fixed |

---

## 🔄 Complete Endpoint Matrix

### English Pipeline Endpoints

```
┌─────────────────────────────────────────────────────────────────┐
│                    ENGLISH ENDPOINTS                             │
└─────────────────────────────────────────────────────────────────┘

1. POST /api/ai/en
   ├─ Type: Auto (with fallback)
   ├─ Function: chat_en_auto()
   ├─ Providers: groq | gemini | openrouter
   ├─ Fallback: Yes → OpenRouter
   ├─ Use: Production, reliability
   └─ Status: ✅ Fixed 2025-10-27

2. POST /api/ai/en/groq
   ├─ Type: Direct (NO fallback)
   ├─ Function: chat_en_groq_direct()
   ├─ Provider: Groq ONLY
   ├─ Fallback: No
   ├─ Use: Testing, debugging
   └─ Status: ✅ NEW 2025-10-27

3. POST /api/ai/en/gemini
   ├─ Type: Direct (NO fallback)
   ├─ Function: chat_en_gemini_direct()
   ├─ Provider: Gemini ONLY
   ├─ Fallback: No
   ├─ Use: Testing, debugging
   └─ Status: ✅ NEW 2025-10-27

4. POST /api/ai/en/openrouter
   ├─ Type: Direct (NO fallback)
   ├─ Function: chat_en_openrouter_direct()
   ├─ Provider: OpenRouter ONLY
   ├─ Fallback: No
   ├─ Use: Testing, debugging
   └─ Status: ✅ NEW 2025-10-27
```

### Hindi Pipeline Endpoints

```
┌─────────────────────────────────────────────────────────────────┐
│                    HINDI ENDPOINTS                               │
└─────────────────────────────────────────────────────────────────┘

1. POST /api/ai/hi
   ├─ Type: Auto (with fallback)
   ├─ Function: chat_hi_auto()
   ├─ Providers: gemini | openrouter
   ├─ Fallback: Yes → OpenRouter
   ├─ Use: Production, reliability
   └─ Status: ✅ Fixed 2025-10-27

2. POST /api/ai/hi/gemini
   ├─ Type: Direct (NO fallback)
   ├─ Function: chat_hi_gemini_direct()
   ├─ Provider: Gemini ONLY
   ├─ Fallback: No
   ├─ Use: Testing, debugging
   └─ Status: ✅ NEW 2025-10-27

3. POST /api/ai/hi/openrouter
   ├─ Type: Direct (NO fallback)
   ├─ Function: chat_hi_openrouter_direct()
   ├─ Provider: OpenRouter ONLY
   ├─ Fallback: No
   ├─ Use: Testing, debugging
   └─ Status: ✅ NEW 2025-10-27

Note: Groq NOT supported for Hindi (code restriction)
```

### Screen Capture Pipeline Endpoints

```
┌─────────────────────────────────────────────────────────────────┐
│                SCREEN CAPTURE ENDPOINTS                          │
└─────────────────────────────────────────────────────────────────┘

1. POST /api/ai/sc
   ├─ Type: Auto (with fallback)
   ├─ Function: chat_sc_auto()
   ├─ Providers: gemini | groq | openrouter
   ├─ Fallback: Yes → OpenRouter
   ├─ Use: Text-based screen context
   └─ Status: ✅ Fixed 2025-10-27

2. POST /api/capture
   ├─ Type: Auto (conditional fallback)
   ├─ Function: sc_analyze_image()
   ├─ Providers: gemini (vision) | groq (native vision) | openrouter (vision)
   ├─ Fallback: Conditional (sc_fallback_or setting)
   ├─ Use: Screen capture + vision analysis
   └─ Status: ✅ Fixed 2025-10-27

*Note: Groq now supports native vision with llama-4-scout-17b-16e-instruct
```

---

## 🎯 Fallback Behavior Comparison

### English Auto Fallback

```
Provider Setting → Primary → Fallback
─────────────────────────────────────────────────────────
en.provider = "groq"
    ↓
    Try: Groq (llama-3.1-8b-instant)
    ↓ Fail
    Fallback: OpenRouter (meta-llama/llama-3.1-70b)
    ✅ Return result

en.provider = "gemini"
    ↓
    Try: Gemini (gemini-2.5-flash)
    ↓ Fail
    Fallback: OpenRouter (meta-llama/llama-3.1-70b)
    ✅ Return result

en.provider = "openrouter"
    ↓
    Try: OpenRouter (llama-3.1-70b)
    ↓ Fail
    Fallback: OpenRouter Alt (GPT-4o-mini or Claude-3.5)
    ✅ Return result
```

### Hindi Auto Fallback

```
Provider Setting → Primary → Fallback
─────────────────────────────────────────────────────────
hi.provider = "gemini"
    ↓
    Try: Gemini (gemini-2.5-flash)
    ↓ Fail
    Fallback: OpenRouter (meta-llama/llama-3.1-70b)
    ✅ Return result

hi.provider = "openrouter"
    ↓
    Try: OpenRouter (llama-3.1-70b)
    ↓ Fail
    Fallback: OpenRouter Alt (GPT-4o-mini or Claude-3.5)
    ✅ Return result
```

### Screen Capture Fallback (Image)

```
Provider Setting → Primary → Fallback (Conditional)
─────────────────────────────────────────────────────────
sc.provider = "gemini"
    ↓
    Try: Gemini Vision (gemini-2.5-flash)
    ↓ Fail
    Check: sc_fallback_or == true?
    ├─ Yes → Fallback: OpenRouter Vision (sc_fallback_or_model)
    │        ✅ Return result
    └─ No → ❌ Return original error

sc.provider = "groq"
    ↓
    Try: OpenRouter Vision (llama-4-scout-17b-16e)
    ↓ Fail
    Check: sc_fallback_or == true?
    ├─ Yes → Fallback: OpenRouter Vision (different model)
    │        ✅ Return result
    └─ No → ❌ Return original error

sc.provider = "openrouter"
    ↓
    Try: OpenRouter Vision (google/gemini-2.5-flash)
    ↓ Fail → ❌ Return error (no fallback)
```

---

## 📋 Configuration Examples

### English Configuration

```json
{
  "en": {
    "provider": "groq",
    "model": "llama-3.1-8b-instant",
    "custom_model": null,
    "prompt": "You are a helpful assistant"
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
  "fallback": {
    "openrouter_choice": "openai"
  }
}
```

### Hindi Configuration

```json
{
  "hi": {
    "provider": "gemini",
    "model": "gemini-2.5-flash",
    "custom_model": null,
    "prompt": "आप सहायक सहायक हैं"
  },
  "providers": {
    "gemini": {
      "default_model": "gemini-2.5-flash",
      "extra_models": ""
    },
    "openrouter": {
      "default_model": "meta-llama/llama-3.1-70b",
      "extra_models": ""
    }
  },
  "fallback": {
    "openrouter_choice": "openai"
  }
}
```

### Screen Capture Configuration

```json
{
  "sc": {
    "provider": "gemini",
    "model": "gemini-2.5-flash",
    "custom_model": null,
    "prompt": "You are ScreenCapture-AI..."
  },
  "sc_providers": {
    "gemini": {
      "default_model": "gemini-2.5-flash",
      "extra_models": ""
    },
    "groq": {
      "default_model": "meta-llama/llama-4-scout-17b-16e-instruct",
      "extra_models": ""
    },
    "openrouter": {
      "default_model": "google/gemini-2.5-flash",
      "extra_models": ""
    }
  },
  "sc_fallback_or": true,
  "sc_fallback_or_model": "google/gemini-2.5-flash"
}
```

---

## 🔑 API Key Requirements

### All Endpoints

| Provider | Key Name | Required For | Stored In |
|----------|----------|--------------|-----------|
| **Groq** | `groq` | EN (all), SC (all - native vision) | System Keyring |
| **Gemini** | `gemini` | EN, HI, SC (all) | System Keyring |
| **OpenRouter** | `openrouter` | EN, HI, SC (all fallbacks + Groq vision) | System Keyring |

### Key Management Endpoints

```
POST   /api/providers/{name}/key      # Save API key
DELETE /api/providers/{name}/key      # Delete API key
GET    /api/providers/state           # Check which keys exist
```

**Example:**
```bash
# Save Groq key
curl -X POST http://localhost:8080/api/providers/groq/key \
  -H "Content-Type: application/json" \
  -d '{"api_key": "gsk_..."}'

# Check key status
curl http://localhost:8080/api/providers/state
# Response: {"groq":{"has_key":true},"gemini":{"has_key":false},...}
```

---

## 🎯 Use Case Recommendations

### When to Use Auto Endpoints

```
✅ Production deployments
✅ User-facing features
✅ When reliability is critical
✅ When you want automatic failover
✅ When provider choice doesn't matter

Examples:
- Chatbot in production
- Voice assistant responses
- Real-time transcription analysis
```

### When to Use Direct Endpoints

```
✅ Testing specific providers
✅ Debugging provider-specific issues
✅ Cost optimization (force cheaper provider)
✅ Feature testing (provider-specific models)
✅ Performance comparison

Examples:
- Testing if Groq key works
- Comparing Gemini vs OpenRouter quality
- Debugging why Gemini is slow
- Cost analysis per provider
```

### When to Use Screen Capture

```
✅ Code review assistance
✅ Error message interpretation
✅ UI/UX analysis
✅ Documentation screenshot analysis
✅ Visual debugging

Examples:
- "What's wrong with my terminal error?"
- "Explain this code snippet"
- "Review this UI design"
```

---

## 📊 Error Code Reference

### Standardized HTTP Status Codes (NEW 2025-10-27)

| Status Code | Meaning | When It Occurs | Example Message |
|-------------|---------|----------------|-----------------|
| **200 OK** | Success | API call successful | `{"output":"...","provider":"groq","model":"..."}` |
| **400 BAD_REQUEST** | Bad configuration | No model set, unsupported provider | `"No model for Groq"` |
| **401 UNAUTHORIZED** | Missing/invalid API key | API key not in keyring or invalid | `"Missing Groq API key"` |
| **502 BAD_GATEWAY** | Provider unreachable | Network error, API down | `"groq http 503"` |
| **500 INTERNAL_SERVER_ERROR** | Unexpected error | Unknown failure | `"Internal error"` |

### Before vs After Fix

```
BEFORE (Inconsistent):
POST /api/ai/en → Missing key → 400 BAD_REQUEST
POST /api/ai/en/groq → Missing key → 500 INTERNAL_SERVER_ERROR

AFTER (Consistent):
POST /api/ai/en → Missing key → 401 UNAUTHORIZED
POST /api/ai/en/groq → Missing key → 401 UNAUTHORIZED
```

---

## 🔧 Troubleshooting Guide

### "Missing API key" Error

```
Problem: 401 "Missing Groq API key"

Solutions:
1. Save the API key:
   POST /api/providers/groq/key
   Body: {"api_key": "your-key"}

2. Verify key exists:
   GET /api/providers/state

3. If using auto endpoint, ensure fallback key exists:
   POST /api/providers/openrouter/key
```

### "No fallback triggered" Issue

```
Problem: Direct endpoint fails, no fallback

Expected Behavior:
- Direct endpoints DON'T have fallback (by design)

Solution:
- Use auto endpoint instead: /api/ai/en (not /api/ai/en/groq)
- Or set up the specific provider's API key
```

### "Missing Groq API key" Error (Screen Capture)

```
Problem: 401 "Missing Groq API key"

Explanation:
- Groq now supports native vision with llama-4-scout-17b-16e-instruct
- Screen capture with Groq requires Groq API key (not OpenRouter)

Solution:
1. Add Groq API key:
   POST /api/providers/groq/key
   Body: {"api_key": "gsk_..."}

2. Verify model is set:
   {"sc": {"model": "meta-llama/llama-4-scout-17b-16e-instruct"}}

3. Or switch to Gemini (alternative):
   {"sc": {"provider": "gemini"}}
```

---

## 📈 Performance Comparison

| Endpoint Type | Latency (Typical) | Reliability | Use Case |
|---------------|-------------------|-------------|----------|
| **EN Auto** | 200-500ms (Groq)<br>500-1s (Gemini)<br>1-2s (OpenRouter) | High (with fallback) | Production |
| **EN Direct** | Same as auto | Medium (no fallback) | Testing |
| **HI Auto** | 500-1s (Gemini)<br>1-2s (OpenRouter) | High (with fallback) | Production |
| **HI Direct** | Same as auto | Medium (no fallback) | Testing |
| **SC Text** | Same as EN/HI | High (with fallback) | Context queries |
| **SC Image** | 1-3s (Gemini Vision)<br>2-5s (OpenRouter Vision) | High (conditional fallback) | Screen analysis |

---

## 🎨 Visual Summary

```
┌─────────────────────────────────────────────────────────────────┐
│            ENDPOINT DECISION TREE                                │
└─────────────────────────────────────────────────────────────────┘

   Need English AI?
        │
        ├─ Want reliability? → /api/ai/en (auto)
        └─ Want specific provider? → /api/ai/en/{provider} (direct)

   Need Hindi AI?
        │
        ├─ Want reliability? → /api/ai/hi (auto)
        └─ Want specific provider? → /api/ai/hi/{provider} (direct)

   Need Screen Analysis?
        │
        ├─ With image? → POST /api/capture
        └─ Text only? → POST /api/ai/sc
```

---

**Related Documents:**
- [01_ENGLISH_PIPELINE.md](./01_ENGLISH_PIPELINE.md) - English pipeline details
- [02_HINDI_PIPELINE.md](./02_HINDI_PIPELINE.md) - Hindi pipeline details
- [03_SCREENSHOT_PIPELINE.md](./03_SCREENSHOT_PIPELINE.md) - Screenshot pipeline details
- [FIXED-ISSUES.md](../FIXED-ISSUES.md) - Complete list of fixes applied
- [04_API_ENDPOINTS.md](./04_API_ENDPOINTS.md) - All API endpoints reference

---

**Last Updated**: 2025-10-27  
**Status**: ✅ All endpoints documented and tested
