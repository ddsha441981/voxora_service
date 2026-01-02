use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

// Message content can be either plain text (String) or structured array (for images)
#[derive(Serialize)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Structured(Vec<GroqContent>),
}

#[derive(Serialize)]
struct ChatReqMsg {
    role: String,
    content: MessageContent,
}

#[derive(Serialize)]
struct ChatReqBody {
    model: String,
    messages: Vec<ChatReqMsg>,
    stream: bool,
}

#[derive(Serialize)]
#[serde(untagged)]
enum GroqContent {
    Text { r#type: String, text: String },
    Image { r#type: String, image_url: GroqImageUrl },
}

#[derive(Serialize)]
struct GroqImageUrl { url: String }

#[derive(Deserialize)]
struct ChatResChoiceMsg { content: Option<String> }
#[derive(Deserialize)]
struct ChatResChoice { message: ChatResChoiceMsg }
#[derive(Deserialize)]
struct ChatRes { choices: Vec<ChatResChoice> }

pub async fn chat(api_key: &str, model: &str, system_prompt: &str, user_input: &str) -> anyhow::Result<String> {
    let url = "https://api.groq.com/openai/v1/chat/completions";
    let body = ChatReqBody {
        model: model.to_string(),
        messages: vec![
            ChatReqMsg { role: "system".into(), content: MessageContent::Text(system_prompt.to_string()) },
            ChatReqMsg { role: "user".into(), content: MessageContent::Text(user_input.to_string()) },
        ],
        stream: false,
    };
    let auth = format!("Bearer {}", api_key);
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, auth)
        .json(&body)
        .send()
        .await?;
    if !res.status().is_success() { 
        let status = res.status();
        let body = res.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
        return Err(anyhow::anyhow!("groq http {}: {}", status, body)); 
    }
    let json: ChatRes = res.json().await?;
    let out = json.choices.get(0).and_then(|c| c.message.content.clone()).unwrap_or_default();
    Ok(out)
}

/// Native Groq vision support for llama-4-scout-17b-16e-instruct and other multimodal models
pub async fn chat_with_image(api_key: &str, model: &str, system_prompt: &str, base64_png: &str) -> anyhow::Result<String> {
    let url = "https://api.groq.com/openai/v1/chat/completions";
    let messages = vec![
        ChatReqMsg { role: "system".into(), content: MessageContent::Text(system_prompt.to_string()) },
        ChatReqMsg { 
            role: "user".into(), 
            content: MessageContent::Structured(vec![
                GroqContent::Text { r#type: "text".into(), text: "Analyze this screenshot".into() },
                GroqContent::Image { r#type: "image_url".into(), image_url: GroqImageUrl { url: format!("data:image/png;base64,{}", base64_png) } },
            ])
        },
    ];
    let body = ChatReqBody { model: model.to_string(), messages, stream: false };
    let auth = format!("Bearer {}", api_key);
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, auth)
        .json(&body)
        .send()
        .await?;
    if !res.status().is_success() { 
        let status = res.status();
        let body = res.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
        return Err(anyhow::anyhow!("groq vision http {}: {}", status, body)); 
    }
    let json: ChatRes = res.json().await?;
    let out = json.choices.get(0).and_then(|c| c.message.content.clone()).unwrap_or_default();
    Ok(out)
}


/// Groq Streaming
#[derive(Deserialize)]
struct StreamDelta { content: Option<String> }
#[derive(Deserialize)]
struct StreamChoice { delta: Option<StreamDelta>, finish_reason: Option<String> }
#[derive(Deserialize)]
struct StreamEvent { choices: Option<Vec<StreamChoice>> }

/// Streaming chat: accumulate delta.content from SSE events
pub async fn chat_stream(api_key: &str, model: &str, system_prompt: &str, user_input: &str) -> anyhow::Result<String> {
    let url = "https://api.groq.com/openai/v1/chat/completions";
    let body = ChatReqBody {
        model: model.to_string(),
        messages: vec![
            ChatReqMsg { role: "system".into(), content: MessageContent::Text(system_prompt.to_string()) },
            ChatReqMsg { role: "user".into(), content: MessageContent::Text(user_input.to_string()) },
        ],
        stream: true,
    };
    let auth = format!("Bearer {}", api_key);
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, auth)
        .json(&body)
        .send()
        .await?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
        return Err(anyhow::anyhow!("groq stream http {}: {}", status, body));
    }
    let text = res.text().await.unwrap_or_default();
    let mut out = String::new();
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("data:") {
            let js = rest.trim();
            if js.is_empty() || js == "[DONE]" { continue; }
            if let Ok(ev) = serde_json::from_str::<StreamEvent>(js) {
                if let Some(choices) = ev.choices {
                    for ch in choices {
                        if let Some(delta) = ch.delta { if let Some(s) = delta.content { out.push_str(&s); } }
                    }
                }
            }
        }
    }
    Ok(out)
}

