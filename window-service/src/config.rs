use serde::{Deserialize, Serialize};
use std::{fs, path::Path, io};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCfg {
    pub default_model: Option<String>,
    pub extra_models: Option<String>, // comma-separated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Providers {
    pub groq: ProviderCfg,
    pub gemini: ProviderCfg,
    pub openrouter: ProviderCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSettings {
    pub provider: String,                 // "default" | provider key | "custom"
    pub model: Option<String>,            // when provider != custom
    pub custom_model: Option<String>,     // when provider == custom
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackCfg {
    // Common fallback selection for OpenRouter different-model fallback.
    // Allowed: "openai" | "claude"
    pub openrouter_choice: String,
}

impl Default for FallbackCfg {
    fn default() -> Self { Self { openrouter_choice: "openai".into() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutsCfg {
    #[serde(default = "default_health_check_timeout")]
    pub health_check_secs: u64,
    #[serde(default = "default_workspace_timeout")]
    pub workspace_secs: u64,
    #[serde(default = "default_chat_timeout")]
    pub chat_secs: u64,
}

fn default_health_check_timeout() -> u64 { 3 }
fn default_workspace_timeout() -> u64 { 4 }
fn default_chat_timeout() -> u64 { 15 }

impl Default for TimeoutsCfg {
    fn default() -> Self {
        Self {
            health_check_secs: default_health_check_timeout(),
            workspace_secs: default_workspace_timeout(),
            chat_secs: default_chat_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteUiCfg {
    #[serde(default)]
    pub chat_default: bool,
    #[serde(default)]
    pub stream_default: bool,
    #[serde(default = "default_chat_mode")]
    pub chat_mode: String, // "chat" | "query"
}

fn default_chat_mode() -> String { "chat".into() }

impl Default for RemoteUiCfg {
    fn default() -> Self {
        Self { chat_default: false, stream_default: false, chat_mode: default_chat_mode() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub en: LangSettings,
    pub hi: LangSettings,
    // Screen-capture settings
    #[serde(default = "default_sc_settings")]
    pub sc: LangSettings,
    pub providers: Providers,
    // Separate provider defaults for screen-capture
    #[serde(default = "default_sc_providers")]
    pub sc_providers: Providers,
    #[serde(default)]
    pub fallback: FallbackCfg,
    // Screen-capture OpenRouter fallback toggle/model
    #[serde(default)]
    pub sc_fallback_or: bool,
    #[serde(default)]
    pub sc_fallback_or_model: Option<String>,
    // Remote UI preferences
    #[serde(default)]
    pub remote: RemoteUiCfg,
    // HTTP timeouts
    #[serde(default)]
    pub timeouts: TimeoutsCfg,
    // Provider feature toggles
    #[serde(default)]
    pub groq_streaming: bool,
}

fn default_sc_settings() -> LangSettings {
    LangSettings { provider: "gemini".into(), model: Some("gemini-2.5-flash".into()), custom_model: None, prompt: String::new() }
}
fn default_sc_providers() -> Providers {
    Providers {
        groq: ProviderCfg { default_model: Some("meta-llama/llama-4-scout-17b-16e-instruct".into()), extra_models: None },
        gemini: ProviderCfg { default_model: Some("gemini-2.5-flash".into()), extra_models: None },
        openrouter: ProviderCfg { default_model: Some("google/gemini-2.5-flash".into()), extra_models: None },
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            en: LangSettings { provider: "groq".into(), model: Some("llama-3.1-8b-instant".into()), custom_model: None, prompt: String::new() },
            hi: LangSettings { provider: "gemini".into(), model: Some("gemini-2.5-flash".into()), custom_model: None, prompt: String::new() },
            sc: default_sc_settings(),
            providers: Providers {
                groq: ProviderCfg { default_model: Some("llama-3.1-8b-instant".into()), extra_models: None },
                gemini: ProviderCfg { default_model: Some("gemini-2.5-flash".into()), extra_models: None },
                openrouter: ProviderCfg { default_model: Some("meta-llama/llama-3.1-70b".into()), extra_models: Some("qwen/qwen2-7b-instruct,nousresearch/hermes-3-llama-3.1-8b".into()) },
            },
            sc_providers: default_sc_providers(),
            fallback: FallbackCfg::default(),
            sc_fallback_or: true,
            sc_fallback_or_model: Some("google/gemini-2.5-flash".into()),
            remote: RemoteUiCfg::default(),
            timeouts: TimeoutsCfg::default(),
            groq_streaming: false,
        }
    }
}

pub fn load_from(path: &Path) -> io::Result<Settings> {
    if !path.exists() {
        return Ok(Settings::default());
    }
    let data = fs::read_to_string(path)?;
    let s: Settings = serde_json::from_str(&data).unwrap_or_default();
    Ok(s)
}

pub fn save_to(path: &Path, s: &Settings) -> io::Result<()> {
    if let Some(dir) = path.parent() { if !dir.exists() { fs::create_dir_all(dir)?; } }
    let data = serde_json::to_string_pretty(s).unwrap_or_else(|_| "{}".into());
    
    // Atomic write: write to temp file, then rename
    // This prevents corruption if process crashes mid-write
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, data)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}
