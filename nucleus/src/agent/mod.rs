pub(crate) mod openai;

use vemodel::*;
use vrs_core_sdk::{callback, codec::*, error::RuntimeError, http::*, storage, CallResult};

const HTTP_MASK: u128 = 0x0000000f_00000000_00000000_00000000;
const KEY_STORE: u128 = 0x00000010_00000000_00000000_00000000;

pub const OPENAI: [u8; 4] = *b"opai";
pub const DEEPSEEK: [u8; 4] = *b"dpsk";

pub(crate) fn set_llm_key(vendor: [u8; 4], key: String) -> Result<(), RuntimeError> {
    let ty = (u32::from_be_bytes(vendor) as u128) << 64 | KEY_STORE;
    storage::put(&ty.to_be_bytes(), key)
}

pub(crate) fn get_llm_key(vendor: [u8; 4]) -> Result<Option<Vec<u8>>, RuntimeError> {
    let ty = (u32::from_be_bytes(vendor) as u128) << 64 | KEY_STORE;
    storage::get(&ty.to_be_bytes())
}

fn http_trace_key(id: u64) -> [u8; 16] {
    (HTTP_MASK | id as u128).to_be_bytes()
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum HttpCallType {
    CreatingAgent(CommunityId),
    CreatingSession(ContentId),
    AppendingMessage(ContentId),
    InvokingLLM(ContentId),
}

#[callback]
pub fn on_response(id: u64, response: CallResult<HttpResponse>) {
    let key = http_trace_key(id);
    match storage::get(&key) {
        Ok(Some(v)) => {
            if let Ok(call_type) = HttpCallType::decode(&mut &v[..]) {
                untrace(&key, call_type, response);
            }
        }
        Ok(None) => {}
        Err(_e) => {}
    }
}

fn trace(id: u64, call_type: HttpCallType) -> Result<(), RuntimeError> {
    let key = http_trace_key(id);
    storage::put(&key, &call_type.encode())
}

fn untrace(key: &[u8], call_type: HttpCallType, response: CallResult<HttpResponse>) {
    match call_type {
        HttpCallType::CreatingAgent(community_id) => {
            vrs_core_sdk::println!("{:?}", response);
        }
        HttpCallType::CreatingSession(content_id) => {}
        HttpCallType::AppendingMessage(content_id) => {}
        HttpCallType::InvokingLLM(content_id) => {}
    }
    let _ = storage::del(key);
}

pub fn init_agent(community: &str, prompt: String) -> Result<(), String> {
    let community_id = crate::community_id(community).expect("caller check;");
    // TODO
    let key = get_llm_key(OPENAI)
        .map_err(|e| e.to_string())?
        .map(|b| String::from_utf8(b))
        .transpose()
        .map_err(|_| "Invalid OpenAI key".to_string())?
        .ok_or("OpenAI key not set".to_string())?;
    let id = openai::create_assistant(community, prompt, key).map_err(|e| e.to_string())?;
    trace(id, HttpCallType::CreatingAgent(community_id)).map_err(|e| e.to_string())
}

pub fn create_session_and_run(msg: &str) {}
