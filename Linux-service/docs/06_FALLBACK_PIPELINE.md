# OpenRouter Fallback Pipeline Documentation

## Overview
This document describes the fallback mechanism for each language pipeline (English, Hindi, and Screenshot) in the Voxora service.

---

## 📋 English Pipeline (`chat_en_auto`)

### Primary Provider Options:
1. **Groq** (default)
2. **Gemini** 
3. **OpenRouter**

### Fallback Logic:

```
┌─────────────────────────────────────────────────────────────┐
│                    ENGLISH PIPELINE                          │
└─────────────────────────────────────────────────────────────┘

┌──────────────┐
│ Provider:    │
│   GROQ       │──── Success ──> Return Result
└──────────────┘
      │
      │ Failure
      ▼
┌──────────────────────────────────────────────────────────────┐
│ FALLBACK: OpenRouter Primary                                 │
│ Model: settings.providers.openrouter.default_model           │
│ Current: "meta-llama/llama-3.1-70b"                          │
└──────────────────────────────────────────────────────────────┘

-----------------------------------------------------------

┌──────────────┐
│ Provider:    │
│   GEMINI     │──── Success ──> Return Result
└──────────────┘
      │
      │ Failure
      ▼
┌──────────────────────────────────────────────────────────────┐
│ FALLBACK: OpenRouter Primary                                 │
│ Model: settings.providers.openrouter.default_model           │
│ Current: "meta-llama/llama-3.1-70b"                          │
└──────────────────────────────────────────────────────────────┘

-----------------------------------------------------------

┌──────────────┐
│ Provider:    │
│ OPENROUTER   │──── Success ──> Return Result
└──────────────┘
      │
      │ Failure
      ▼
┌──────────────────────────────────────────────────────────────┐
│ FALLBACK: OpenRouter Alternative                             │
│ Model: Based on fallback.openrouter_choice                   │
│ Choice: "claude" → "anthropic/claude-3.5-sonnet"             │
│ Choice: other → "openai/gpt-4o-mini"                         │
└──────────────────────────────────────────────────────────────┘
```

**Code Reference:** Lines 101-127 in `src/ai/mod.rs`

---

## 📋 Hindi Pipeline (`chat_hi_auto`)

### Primary Provider Options:
1. **Gemini** (default)
2. **OpenRouter**

### Fallback Logic:

```
┌─────────────────────────────────────────────────────────────┐
│                    HINDI PIPELINE                            │
└─────────────────────────────────────────────────────────────┘

┌──────────────┐
│ Provider:    │
│   GEMINI     │──── Success ──> Return Result
└──────────────┘
      │
      │ Failure
      ▼
┌──────────────────────────────────────────────────────────────┐
│ FALLBACK: OpenRouter Primary                                 │
│ Model: settings.providers.openrouter.default_model           │
│ Current: "meta-llama/llama-3.1-70b"                          │
└──────────────────────────────────────────────────────────────┘

-----------------------------------------------------------

┌──────────────┐
│ Provider:    │
│ OPENROUTER   │──── Success ──> Return Result
└──────────────┘
      │
      │ Failure
      ▼
┌──────────────────────────────────────────────────────────────┐
│ FALLBACK: OpenRouter Alternative                             │
│ Model: Based on fallback.openrouter_choice                   │
│ Choice: "claude" → "anthropic/claude-3.5-sonnet"             │
│ Choice: other → "openai/gpt-4o-mini"                         │
└──────────────────────────────────────────────────────────────┘
```

**Code Reference:** Lines 129-149 in `src/ai/mod.rs`

---

## 📋 Screenshot Pipeline (`chat_sc_auto`)

### Primary Provider Options:
1. **Gemini** (default - with vision)
2. **Groq** (via OpenRouter vision)
3. **OpenRouter**

### Fallback Logic:

```
┌─────────────────────────────────────────────────────────────┐
│                 SCREENSHOT PIPELINE                          │
└─────────────────────────────────────────────────────────────┘

┌──────────────┐
│ Provider:    │
│   GEMINI     │──── Success ──> Return Result
└──────────────┘
      │
      │ Failure
      ▼
┌──────────────────────────────────────────────────────────────┐
│ FALLBACK: OpenRouter Primary (if enabled)                    │
│ Enabled: sc_fallback_or = true                               │
│ Model: settings.sc_providers.openrouter.default_model        │
│ Current: "google/gemini-2.5-flash"                           │
└──────────────────────────────────────────────────────────────┘

-----------------------------------------------------------

┌──────────────┐
│ Provider:    │
│   GROQ       │──── Success ──> Return Result
└──────────────┘
      │
      │ Failure
      ▼
┌──────────────────────────────────────────────────────────────┐
│ FALLBACK: OpenRouter Primary (if enabled)                    │
│ Enabled: sc_fallback_or = true                               │
│ Model: sc_fallback_or_model                                  │
│ Current: "google/gemini-2.5-flash"                           │
└──────────────────────────────────────────────────────────────┘

-----------------------------------------------------------

┌──────────────┐
│ Provider:    │
│ OPENROUTER   │──── Success ──> Return Result
└──────────────┘
      │
      │ Failure
      ▼
┌──────────────────────────────────────────────────────────────┐
│ FALLBACK: OpenRouter Alternative                             │
│ Model: sc_fallback_or_model or fallback.openrouter_choice    │
│ Current: "google/gemini-2.5-flash"                           │
└──────────────────────────────────────────────────────────────┘
```

**Code Reference:** Lines 151-177 in `src/ai/mod.rs`

---

## 🔧 Configuration Details

### Current Settings (from `data/settings.json`):

```json
{
  "fallback": {
    "openrouter_choice": "claude"
  },
  "sc_fallback_or": true,
  "sc_fallback_or_model": "google/gemini-2.5-flash",
  "providers": {
    "openrouter": {
      "default_model": "meta-llama/llama-3.1-70b"
    }
  },
  "sc_providers": {
    "openrouter": {
      "default_model": "google/gemini-2.5-flash"
    }
  }
}
```

---

## 📊 Fallback Functions

### 1. `fallback_to_openrouter_primary()` (Lines 265-272)
- Uses default OpenRouter model from settings
- For chat: `providers.openrouter.default_model`
- For screenshot: `sc_providers.openrouter.default_model`

### 2. `fallback_to_openrouter_alt()` (Lines 282-289)
- Uses alternative model based on `fallback.openrouter_choice`
- Maps:
  - `"claude"` → `"anthropic/claude-3.5-sonnet"`
  - Other → `"openai/gpt-4o-mini"`
- For screenshot: Uses `sc_fallback_or_model` if available

---

## 🎯 Key Differences

| Feature | English/Hindi | Screenshot |
|---------|---------------|------------|
| Fallback trigger | Always on error | Conditional (`sc_fallback_or`) |
| Primary fallback model | `providers.openrouter.default_model` | `sc_providers.openrouter.default_model` |
| Alternative fallback | Based on `openrouter_choice` | Uses `sc_fallback_or_model` |
| Vision support | N/A | Required for all providers |

---

## 🔄 Flow Summary

1. **Try primary provider** (based on language config)
2. **On failure:**
   - For Groq/Gemini → Use OpenRouter Primary (default model)
   - For OpenRouter → Use OpenRouter Alternative (claude/gpt fallback)
3. **Mark prompt as sent** (if first-time prompt was used successfully)

---

**Generated:** 2025-10-26  
**Version:** Based on current codebase snapshot
