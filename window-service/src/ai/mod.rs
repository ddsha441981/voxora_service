pub mod groq;
pub mod gemini;
pub mod openrouter;

use crate::{state::AppState, config::Settings, secrets};

#[derive(Debug, Clone, serde::Serialize)]
pub struct AiResult {
    pub output: String,
    pub provider: String,
    pub model: String,
}

fn choose_model(lang: &str, provider: &str, s: &Settings) -> Option<String> {
    match lang {
        "en" => {
            // Prefer language-selected model if provider matches; else provider default
            if s.en.provider == provider { s.en.model.clone().or_else(|| s.providers_for(provider)) } else { s.providers_for(provider) }
        }
        "hi" => {
            if s.hi.provider == provider { s.hi.model.clone().or_else(|| s.providers_for(provider)) } else { s.providers_for(provider) }
        }
        "sc" => {
            if s.sc.provider == provider { s.sc.model.clone().or_else(|| s.sc_providers_for(provider)) } else { s.sc_providers_for(provider) }
        }
        _ => None,
    }
}

trait ProviderDefaults {
    fn providers_for(&self, provider: &str) -> Option<String>;
    fn sc_providers_for(&self, provider: &str) -> Option<String>;
}
impl ProviderDefaults for Settings {
    fn providers_for(&self, provider: &str) -> Option<String> {
        match provider {
            "groq" => self.providers.groq.default_model.clone(),
            "gemini" => self.providers.gemini.default_model.clone(),
            "openrouter" => self.providers.openrouter.default_model.clone(),
            _ => None,
        }
    }
    fn sc_providers_for(&self, provider: &str) -> Option<String> {
        match provider {
            "groq" => self.sc_providers.groq.default_model.clone(),
            "gemini" => self.sc_providers.gemini.default_model.clone(),
            "openrouter" => self.sc_providers.openrouter.default_model.clone(),
            _ => None,
        }
    }
}

fn choose_prompt(lang: &str, s: &Settings) -> String {
    match lang { "en" => s.en.prompt.clone(), "hi" => s.hi.prompt.clone(), _ => String::new() }
}

// Improved prompt management: returns current prompt and whether it should be marked as sent
async fn get_prompt_for_request(state: &AppState, lang: &str) -> (String, bool) {
    // Check if prompt was already sent in this session
    let sent = match lang {
        "en" => state.prompt_sent_en.lock().await.clone(),
        "hi" => state.prompt_sent_hi.lock().await.clone(),
        "sc" => state.prompt_sent_sc.lock().await.clone(),
        _ => return (String::new(), false),
    };
    
    let s = state.settings.lock().await.clone();
    let current_prompt = choose_prompt(lang, &s);
    
    // Send prompt only on first request (send-once behavior)
    // OpenRouter now handles empty prompts correctly by omitting system message
    if sent && !current_prompt.is_empty() {
        // Already sent - don't resend
        (String::new(), false)
    } else if !current_prompt.is_empty() {
        // First time - send it
        (current_prompt, true)
    } else {
        // No prompt configured
        (String::new(), false)
    }
}

async fn mark_prompt_sent(state: &AppState, lang: &str) {
    match lang {
        "en" => { let mut g = state.prompt_sent_en.lock().await; *g = true; }
        "hi" => { let mut g = state.prompt_sent_hi.lock().await; *g = true; }
        "sc" => { let mut g = state.prompt_sent_sc.lock().await; *g = true; }
        _ => {}
    }
}

// Reset prompt state (for config changes or errors requiring fallback)
pub async fn reset_prompt_state(state: &AppState, lang: &str) {
    match lang {
        "en" => { let mut g = state.prompt_sent_en.lock().await; *g = false; }
        "hi" => { let mut g = state.prompt_sent_hi.lock().await; *g = false; }
        "sc" => { let mut g = state.prompt_sent_sc.lock().await; *g = false; }
        _ => {}
    }
}

fn sanitize_provider_en(p: &str) -> String {
    match p {
        "" | "default" => "groq".to_string(),
        "groq" | "gemini" | "openrouter" => p.to_string(),
        _ => "groq".to_string(),
    }
}
fn sanitize_provider_hi(p: &str) -> String {
    match p {
        "" | "default" => "gemini".to_string(),
        "gemini" | "openrouter" => p.to_string(),
        _ => "gemini".to_string(),
    }
}
fn sanitize_provider_sc(p: &str) -> String {
    match p {
        "" | "default" => "gemini".to_string(),
        "gemini" | "groq" | "openrouter" => p.to_string(),
        _ => "gemini".to_string(),
    }
}

pub async fn chat_en_auto(state: &AppState, input: String) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let prov = sanitize_provider_en(&s.en.provider);
    drop(s);
    let (prompt_once, should_mark) = get_prompt_for_request(state, "en").await;
    match prov.as_str() {
        "groq" => {
            match chat_en_groq_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "en").await; } Ok(ok) }
                Err(_) => { let res = fallback_to_openrouter_primary(state, "en", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "en").await; } res },
            }
        }
        "gemini" => {
            match chat_en_gemini_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "en").await; } Ok(ok) }
                Err(_) => { let res = fallback_to_openrouter_primary(state, "en", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "en").await; } res },
            }
        }
        "openrouter" => {
            match chat_en_openrouter_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "en").await; } Ok(ok) }
                Err(_) => { let res = fallback_to_openrouter_alt(state, "en", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "en").await; } res },
            }
        }
        other => Err(anyhow::anyhow!("Unsupported EN provider: {}", other)),
    }
}

pub async fn chat_hi_auto(state: &AppState, input: String) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let prov = sanitize_provider_hi(&s.hi.provider);
    drop(s);
    let (prompt_once, should_mark) = get_prompt_for_request(state, "hi").await;
    match prov.as_str() {
        "gemini" => {
            match chat_hi_gemini_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "hi").await; } Ok(ok) }
                Err(_) => { let res = fallback_to_openrouter_primary(state, "hi", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "hi").await; } res },
            }
        }
        "openrouter" => {
            match chat_hi_openrouter_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "hi").await; } Ok(ok) }
                Err(_) => { let res = fallback_to_openrouter_alt(state, "hi", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "hi").await; } res },
            }
        }
        other => Err(anyhow::anyhow!("Unsupported HI provider: {}", other)),
    }
}

pub async fn chat_sc_auto(state: &AppState, input: String) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let prov = sanitize_provider_sc(&s.sc.provider);
    drop(s);
    let (prompt_once, should_mark) = get_prompt_for_request(state, "sc").await;
    match prov.as_str() {
        "gemini" => {
            match chat_en_gemini_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "sc").await; } Ok(AiResult { output: ok.output, provider: ok.provider, model: ok.model }) }
                Err(_) => { let res = fallback_to_openrouter_primary(state, "sc", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "sc").await; } res },
            }
        }
        "groq" => {
            match chat_en_groq_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "sc").await; } Ok(AiResult { output: ok.output, provider: ok.provider, model: ok.model }) }
                Err(_) => { let res = fallback_to_openrouter_primary(state, "sc", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "sc").await; } res },
            }
        }
        "openrouter" => {
            match chat_en_openrouter_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "sc").await; } Ok(AiResult { output: ok.output, provider: ok.provider, model: ok.model }) }
                Err(_) => { let res = fallback_to_openrouter_alt(state, "sc", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "sc").await; } res },
            }
        }
        other => Err(anyhow::anyhow!("Unsupported SC provider: {}", other)),
    }
}

pub async fn chat_hi_openrouter(state: &AppState, input: String) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let prov = sanitize_provider_hi(&s.hi.provider);
    drop(s);
    let (prompt_once, should_mark) = get_prompt_for_request(state, "hi").await;
    match prov.as_str() {
        "gemini" => {
            match chat_hi_gemini_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "hi").await; } Ok(ok) }
                Err(_) => { let res = fallback_to_openrouter_primary(state, "hi", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "hi").await; } res },
            }
        }
        "openrouter" => {
            match chat_hi_openrouter_with_prompt(state, input.clone(), &prompt_once).await {
                Ok(ok) => { if should_mark { mark_prompt_sent(state, "hi").await; } Ok(ok) }
                Err(_) => { let res = fallback_to_openrouter_alt(state, "hi", input, &prompt_once).await; if res.is_ok() && should_mark { mark_prompt_sent(state, "hi").await; } res },
            }
        }
        other => Err(anyhow::anyhow!("Unsupported HI provider: {}", other)),
    }
}

pub async fn sc_analyze_image(state: &AppState, base64_png: String) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let prov = sanitize_provider_sc(&s.sc.provider);
    let sc_models = s.sc_providers.clone();
    let sc_fallback = s.sc_fallback_or;
    let sc_fallback_model = s.sc_fallback_or_model.clone().or_else(|| sc_models.openrouter.default_model.clone());
    drop(s);

    // Prompt-once for Screen Capture
    let (prompt_once, should_mark) = get_prompt_for_request(state, "sc").await;

    match prov.as_str() {
        "gemini" => {
            // Primary: Gemini Vision
            let res = async {
                let key = secrets::get_key("gemini");
                let key = key.ok_or_else(|| anyhow::anyhow!("Missing Gemini API key"))?;
                let model = sc_models.gemini.default_model.clone().ok_or_else(|| anyhow::anyhow!("No SC Gemini model"))?;
                let out = gemini::generate_with_image(&key, &model, &prompt_once, &base64_png).await?;
                Ok::<AiResult, anyhow::Error>(AiResult { output: out, provider: "gemini".into(), model })
            }.await;
            if res.is_ok() { if should_mark { mark_prompt_sent(state, "sc").await; } return res; }
            // Fallback: OpenRouter Vision (only if marked)
            if sc_fallback {
                let key = secrets::get_key("openrouter").ok_or_else(|| anyhow::anyhow!("Missing OpenRouter API key"))?;
                let model = sc_fallback_model.ok_or_else(|| anyhow::anyhow!("No SC OpenRouter fallback model"))?;
                let out = openrouter::chat_with_image(&key, &model, &prompt_once, &base64_png).await?;
                if should_mark { mark_prompt_sent(state, "sc").await; }
                return Ok(AiResult { output: out, provider: "openrouter".into(), model });
            }
            // Return original error
            res
        }
        "groq" => {
            // Groq now natively supports vision with llama-4-scout-17b-16e-instruct
            let res = async {
                let key = secrets::get_key("groq");
                let key = key.ok_or_else(|| anyhow::anyhow!("Missing Groq API key"))?;
                let model = sc_models.groq.default_model.clone().ok_or_else(|| anyhow::anyhow!("No SC Groq model configured"))?;
                let out = groq::chat_with_image(&key, &model, &prompt_once, &base64_png).await?;
                Ok::<AiResult, anyhow::Error>(AiResult { output: out, provider: "groq".into(), model })
            }.await;
            if res.is_ok() { if should_mark { mark_prompt_sent(state, "sc").await; } return res; }
            // Fallback: OpenRouter Vision (only if sc_fallback_or is true)
            if sc_fallback {
                let key = secrets::get_key("openrouter").ok_or_else(|| anyhow::anyhow!("Missing OpenRouter API key"))?;
                let model = sc_fallback_model.ok_or_else(|| anyhow::anyhow!("No SC OpenRouter fallback model"))?;
                let out = openrouter::chat_with_image(&key, &model, &prompt_once, &base64_png).await?;
                if should_mark { mark_prompt_sent(state, "sc").await; }
                return Ok(AiResult { output: out, provider: "openrouter".into(), model });
            }
            res
        }
        "openrouter" => {
            let key = secrets::get_key("openrouter").ok_or_else(|| anyhow::anyhow!("Missing OpenRouter API key"))?;
            let model = sc_models.openrouter.default_model.clone().ok_or_else(|| anyhow::anyhow!("No SC OpenRouter model"))?;
            let out = openrouter::chat_with_image(&key, &model, &prompt_once, &base64_png).await?;
            if should_mark { mark_prompt_sent(state, "sc").await; }
            Ok(AiResult { output: out, provider: "openrouter".into(), model })
        }
        other => Err(anyhow::anyhow!("Unsupported SC provider: {}", other)),
    }
}

async fn fallback_to_openrouter_primary(state: &AppState, lang: &str, input: String, prompt: &str) -> anyhow::Result<AiResult> {
    // Use OpenRouter default_model from settings
    let s = state.settings.lock().await.clone();
    let model = if lang=="sc" { s.sc_providers.openrouter.default_model.clone() } else { s.providers.openrouter.default_model.clone() }.ok_or_else(|| anyhow::anyhow!("No OpenRouter default model"))?;
    let key = secrets::get_key("openrouter").ok_or_else(|| anyhow::anyhow!("Missing OpenRouter API key"))?;
    let out = openrouter::chat(&key, &model, prompt, &input).await?;
    Ok(AiResult { output: out, provider: "openrouter".into(), model })
}

fn choose_openrouter_fallback_model(choice: &str) -> &'static str {
    match choice {
        // These are common OpenRouter aliases for OpenAI and Claude; can be changed later via settings if needed
        "claude" => "anthropic/claude-3.5-sonnet",
        _ => "openai/gpt-4o-mini",
    }
}

async fn fallback_to_openrouter_alt(state: &AppState, lang: &str, input: String, prompt: &str) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let choice = if lang=="sc" { s.fallback.openrouter_choice.to_lowercase() } else { s.fallback.openrouter_choice.to_lowercase() };
    let model = if lang=="sc" { s.sc_fallback_or_model.clone().unwrap_or_else(|| choose_openrouter_fallback_model(&choice).to_string()) } else { choose_openrouter_fallback_model(&choice).to_string() };
    let key = secrets::get_key("openrouter").ok_or_else(|| anyhow::anyhow!("Missing OpenRouter API key"))?;
    let out = openrouter::chat(&key, &model, prompt, &input).await?;
    Ok(AiResult { output: out, provider: "openrouter".into(), model })
}

async fn chat_en_groq_with_prompt(state: &AppState, input: String, prompt: &str) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let model = choose_model("en", "groq", &s).ok_or_else(|| anyhow::anyhow!("No model for Groq"))?;
    let use_stream = s.groq_streaming;
    let key = secrets::get_key("groq").ok_or_else(|| anyhow::anyhow!("Missing Groq API key"))?;
    let out = if use_stream { groq::chat_stream(&key, &model, prompt, &input).await? } else { groq::chat(&key, &model, prompt, &input).await? };
   /// let out = groq::chat(&key, &model, prompt, &input).await?;
    Ok(AiResult { output: out, provider: "groq".into(), model })
}

async fn chat_en_gemini_with_prompt(state: &AppState, input: String, prompt: &str) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let model = choose_model("en", "gemini", &s).ok_or_else(|| anyhow::anyhow!("No model for Gemini"))?;
    let key = secrets::get_key("gemini").ok_or_else(|| anyhow::anyhow!("Missing Gemini API key"))?;
    let out = gemini::generate(&key, &model, prompt, &input).await?;
    Ok(AiResult { output: out, provider: "gemini".into(), model })
}

async fn chat_en_openrouter_with_prompt(state: &AppState, input: String, prompt: &str) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let model = choose_model("en", "openrouter", &s).ok_or_else(|| anyhow::anyhow!("No model for OpenRouter"))?;
    let key = secrets::get_key("openrouter").ok_or_else(|| anyhow::anyhow!("Missing OpenRouter API key"))?;
    let out = openrouter::chat(&key, &model, prompt, &input).await?;
    Ok(AiResult { output: out, provider: "openrouter".into(), model })
}

async fn chat_hi_gemini_with_prompt(state: &AppState, input: String, prompt: &str) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let model = choose_model("hi", "gemini", &s).ok_or_else(|| anyhow::anyhow!("No model for Gemini (hi)"))?;
    let key = secrets::get_key("gemini").ok_or_else(|| anyhow::anyhow!("Missing Gemini API key"))?;
    let out = gemini::generate(&key, &model, prompt, &input).await?;
    Ok(AiResult { output: out, provider: "gemini".into(), model })
}

async fn chat_hi_openrouter_with_prompt(state: &AppState, input: String, prompt: &str) -> anyhow::Result<AiResult> {
    let s = state.settings.lock().await.clone();
    let model = choose_model("hi", "openrouter", &s).ok_or_else(|| anyhow::anyhow!("No model for OpenRouter (hi)"))?;
    let key = secrets::get_key("openrouter").ok_or_else(|| anyhow::anyhow!("Missing OpenRouter API key"))?;
    let out = openrouter::chat(&key, &model, prompt, &input).await?;
    Ok(AiResult { output: out, provider: "openrouter".into(), model })
}

// Direct provider calls (no auto-routing or fallback)
pub async fn chat_en_groq_direct(state: &AppState, input: String) -> anyhow::Result<AiResult> {
    let (prompt_once, should_mark) = get_prompt_for_request(state, "en").await;
    let res = chat_en_groq_with_prompt(state, input, &prompt_once).await;
    if res.is_ok() && should_mark { mark_prompt_sent(state, "en").await; }
    res
}

pub async fn chat_en_gemini_direct(state: &AppState, input: String) -> anyhow::Result<AiResult> {
    let (prompt_once, should_mark) = get_prompt_for_request(state, "en").await;
    let res = chat_en_gemini_with_prompt(state, input, &prompt_once).await;
    if res.is_ok() && should_mark { mark_prompt_sent(state, "en").await; }
    res
}

pub async fn chat_en_openrouter_direct(state: &AppState, input: String) -> anyhow::Result<AiResult> {
    let (prompt_once, should_mark) = get_prompt_for_request(state, "en").await;
    let res = chat_en_openrouter_with_prompt(state, input, &prompt_once).await;
    if res.is_ok() && should_mark { mark_prompt_sent(state, "en").await; }
    res
}

pub async fn chat_hi_gemini_direct(state: &AppState, input: String) -> anyhow::Result<AiResult> {
    let (prompt_once, should_mark) = get_prompt_for_request(state, "hi").await;
    let res = chat_hi_gemini_with_prompt(state, input, &prompt_once).await;
    if res.is_ok() && should_mark { mark_prompt_sent(state, "hi").await; }
    res
}

pub async fn chat_hi_openrouter_direct(state: &AppState, input: String) -> anyhow::Result<AiResult> {
    let (prompt_once, should_mark) = get_prompt_for_request(state, "hi").await;
    let res = chat_hi_openrouter_with_prompt(state, input, &prompt_once).await;
    if res.is_ok() && should_mark { mark_prompt_sent(state, "hi").await; }
    res
}
