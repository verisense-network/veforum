pub mod openai;

use std::collections::BTreeMap;
use vemodel::*;
use vrs_core_sdk::{
    callback,
    codec::*,
    error::RuntimeError,
    http::{self, *},
    storage, CallResult,
};

const HTTP_MASK: u128 = 0x0000000f_00000000_00000000_00000000;
const KEY_STORE: u128 = 0x00000010_00000000_00000000_00000000;

pub(crate) fn set_llm_key(key: String) -> Result<(), RuntimeError> {
    storage::put(&KEY_STORE.to_be_bytes(), key)
}

pub(crate) fn get_llm_key() -> Result<Option<Vec<u8>>, RuntimeError> {
    storage::get(&KEY_STORE.to_be_bytes())
}

fn http_event(id: u64) -> [u8; 16] {
    (HTTP_MASK | id as u128).to_be_bytes()
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum HttpEvent {
    AgentCreated(CommunityId),
    SessionCreated(ContentId),
    MessageAppended(ContentId),
    ExecutionCreated(ContentId),
}

#[callback]
pub fn on_response(id: u64, response: CallResult<HttpResponse>) {
    let key = http_event(id);
    match storage::get(&key) {
        Ok(Some(v)) => {
            if let Ok(event) = HttpEvent::decode(&mut &v[..]) {
                dispatch_event(event, response);
                let _ = storage::del(&key);
            }
        }
        Ok(None) => {}
        Err(_e) => {}
    }
}

fn save_event(id: u64, event: HttpEvent) -> Result<(), RuntimeError> {
    let key = http_event(id);
    storage::put(&key, &event.encode())
}

fn dispatch_event(event: HttpEvent, response: CallResult<HttpResponse>) {
    match event {
        HttpEvent::AgentCreated(community_id) => {}
        HttpEvent::SessionCreated(content_id) => {}
        HttpEvent::MessageAppended(content_id) => {}
        HttpEvent::ExecutionCreated(content_id) => {}
    }
}

pub fn init_agent(community: &str, prompt: String) -> Result<(), String> {
    let community_id = crate::community_id(community).expect("caller check;");
    let mut headers = BTreeMap::new();
    let key = get_llm_key()
        .map_err(|e| e.to_string())?
        .map(|b| String::from_utf8(b))
        .transpose()
        .map_err(|_| "Invalid LLM key".to_string())?
        .ok_or("LLM key not set".to_string())?;
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let body = serde_json::json!({
        "instructions": prompt,
        "model": "gpt-4o",
        "name": community,
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
    save_event(id, HttpEvent::AgentCreated(community_id)).map_err(|e| e.to_string())
}

pub fn create_session_and_run(msg: &str) {}
