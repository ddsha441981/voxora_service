use reqwest::header::CONTENT_TYPE;
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct Part { text: String }
#[derive(Serialize)]
struct Content { parts: Vec<Part> }
#[derive(Serialize)]
struct GenReq {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<Content>,
}

// Vision (inline base64 image)
#[derive(Serialize)]
struct InlineData { mime_type: String, data: String }

#[derive(Serialize)]
#[serde(untagged)]
enum PartOrImage { Text { text: String }, InlineData { inline_data: InlineData } }

#[derive(Serialize)]
struct VisionContent { parts: Vec<PartOrImage> }
#[derive(Serialize)]
struct VisionReq { contents: Vec<VisionContent>, #[serde(skip_serializing_if="Option::is_none")] system_instruction: Option<Content> }

pub async fn generate_with_image(api_key: &str, model: &str, system_prompt: &str, base64_png: &str) -> anyhow::Result<String> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model);
    let body = VisionReq { contents: vec![ VisionContent { parts: vec![ PartOrImage::InlineData { inline_data: InlineData { mime_type: "image/png".into(), data: base64_png.to_string() } } ] } ], system_instruction: if system_prompt.trim().is_empty() { None } else { Some(Content { parts: vec![ Part { text: system_prompt.to_string() } ] }) } };
    let client = reqwest::Client::new();
    let res = client.post(&url).header(CONTENT_TYPE, "application/json").header("x-goog-api-key", api_key).json(&body).send().await?;
    if !res.status().is_success() { 
        let status = res.status();
        let body = res.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
        return Err(anyhow::anyhow!("gemini vision http {}: {}", status, body)); 
    }
    let json: GenRes = res.json().await?;
    let text = json.candidates.get(0).and_then(|c| c.content.parts.get(0)).and_then(|p| p.text.clone()).unwrap_or_default();
    Ok(text)
}

#[derive(Deserialize)]
struct GenResCandidateContentPart { text: Option<String> }
#[derive(Deserialize)]
struct GenResCandidateContent { parts: Vec<GenResCandidateContentPart> }
#[derive(Deserialize)]
struct GenResCandidate { content: GenResCandidateContent }
#[derive(Deserialize)]
struct GenRes { candidates: Vec<GenResCandidate> }

pub async fn generate(api_key: &str, model: &str, system_prompt: &str, user_input: &str) -> anyhow::Result<String> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model);
    let body = GenReq {
        contents: vec![ Content { parts: vec![ Part { text: user_input.to_string() } ] } ],
        system_instruction: if system_prompt.trim().is_empty() { None } else { Some(Content { parts: vec![ Part { text: system_prompt.to_string() } ] }) },
    };
    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .header(CONTENT_TYPE, "application/json")
        .header("x-goog-api-key", api_key)
        .json(&body)
        .send()
        .await?;
    if !res.status().is_success() { 
        let status = res.status();
        let body = res.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
        return Err(anyhow::anyhow!("gemini http {}: {}", status, body)); 
    }
    let json: GenRes = res.json().await?;
    let text = json
        .candidates
        .get(0)
        .and_then(|c| c.content.parts.get(0))
        .and_then(|p| p.text.clone())
        .unwrap_or_default();
    Ok(text)
}
