use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

// Message content can be either plain text (String) or structured array (for images)
#[derive(Serialize)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Structured(Vec<ORContent>),
}

#[derive(Serialize)]
struct ChatReqMsg {
    role: String,
    content: MessageContent,
}

pub async fn chat_with_image(api_key: &str, model: &str, system_prompt: &str, base64_png: &str) -> anyhow::Result<String> {
    let url = "https://openrouter.ai/api/v1/chat/completions";
    let mut messages = Vec::new();
    // Only include system message if prompt is not empty
    if !system_prompt.trim().is_empty() {
        messages.push(ChatReqMsg { role: "system".into(), content: MessageContent::Text(system_prompt.to_string()) });
    }
    messages.push(ChatReqMsg { 
        role: "user".into(), 
        content: MessageContent::Structured(vec![
            ORContent::Text { r#type: "text".into(), text: "Describe the screen".into() },
            ORContent::Image { r#type: "image_url".into(), image_url: ORImageUrl { url: format!("data:image/png;base64,{}", base64_png) } },
        ])
    });
    let body = ChatReqBody { model: model.to_string(), messages, stream: false };
    let auth = format!("Bearer {}", api_key);
    let client = reqwest::Client::new();
    let res = client.post(url).header(CONTENT_TYPE, "application/json").header(AUTHORIZATION, auth).json(&body).send().await?;
    if !res.status().is_success() { 
        let status = res.status();
        let body = res.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
        return Err(anyhow::anyhow!("openrouter vision http {}: {}", status, body)); 
    }
    let json: ChatRes = res.json().await?;
    let out = json.choices.get(0).and_then(|c| c.message.content.clone()).unwrap_or_default();
    Ok(out)
}
#[derive(Serialize)]
struct ChatReqBody {
    model: String,
    messages: Vec<ChatReqMsg>,
    stream: bool,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ORContent {
    Text { r#type: String, text: String },
    Image { r#type: String, image_url: ORImageUrl },
}

#[derive(Serialize)]
struct ORImageUrl { url: String }

#[derive(Deserialize)]
struct ChatResChoiceMsg { content: Option<String> }
#[derive(Deserialize)]
struct ChatResChoice { message: ChatResChoiceMsg }
#[derive(Deserialize)]
struct ChatRes { choices: Vec<ChatResChoice> }

pub async fn chat(api_key: &str, model: &str, system_prompt: &str, user_input: &str) -> anyhow::Result<String> {
    let url = "https://openrouter.ai/api/v1/chat/completions";
    let mut messages = Vec::new();
    // Only include system message if prompt is not empty
    if !system_prompt.trim().is_empty() {
        messages.push(ChatReqMsg { role: "system".into(), content: MessageContent::Text(system_prompt.to_string()) });
    }
    messages.push(ChatReqMsg { role: "user".into(), content: MessageContent::Text(user_input.to_string()) });
    
    let body = ChatReqBody {
        model: model.to_string(),
        messages,
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
        return Err(anyhow::anyhow!("openrouter http {}: {}", status, body)); 
    }
    let json: ChatRes = res.json().await?;
    let out = json.choices.get(0).and_then(|c| c.message.content.clone()).unwrap_or_default();
    Ok(out)
}
