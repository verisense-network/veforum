use std::collections::BTreeMap;
use vrs_core_sdk::http::{self, HttpMethod, HttpRequest, RequestHead};

pub(crate) fn create_assistant(name: &str, prompt: String, key: String) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let body = serde_json::json!({
        "instructions": prompt,
        "model": "gpt-4o",
        "name": name,
        "tools": []
    });
    let id = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Post,
            uri: "https://api.openai.com/v1/assistants".to_string(),
            headers,
        },
        body: serde_json::to_vec(&body).expect("json;qed"),
    })
    .map_err(|e| e.to_string())?;
    Ok(id)
}
