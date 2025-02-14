pub(crate) mod openai;

use serde::de::DeserializeOwned;
use vemodel::*;
use vrs_core_sdk::{
    callback, codec::*, error::RuntimeError, http::*, set_timer, storage, timer, CallResult,
};

pub const OPENAI: [u8; 4] = *b"opai";
// pub const DEEPSEEK: [u8; 4] = *b"dpsk";

pub(crate) fn set_llm_key(vendor: [u8; 4], key: String) -> Result<(), String> {
    let ty = crate::trie::llm_key(vendor);
    storage::put(&ty, key.into_bytes()).map_err(|e| e.to_string())
}

pub(crate) fn get_llm_key(vendor: [u8; 4]) -> Result<String, String> {
    let ty = crate::trie::llm_key(vendor);
    storage::get(&ty)
        .map_err(|e| e.to_string())?
        .map(|b| String::from_utf8(b))
        .transpose()
        .map_err(|_| "Invalid LLM key".to_string())?
        .ok_or("LLM key not found".to_string())
}

pub(crate) fn get_agent_id(community_id: CommunityId) -> Result<String, String> {
    let assistant_key = crate::trie::agent_key(community_id);
    storage::get(&assistant_key)
        .map_err(|e| e.to_string())?
        .map(|b| String::from_utf8(b))
        .transpose()
        .map_err(|_| "Invalid assistant id".to_string())?
        .ok_or("Assistant id not found".to_string())
}

pub(crate) fn set_agent_id(community_id: CommunityId, assistant_id: String) -> Result<(), String> {
    let assistant_key = crate::trie::agent_key(community_id);
    storage::put(assistant_key, assistant_id.into_bytes()).map_err(|e| e.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum HttpCallType {
    CreatingAgent(CommunityId),
    AppendingMessage(ContentId),
    InvokingLLM(ContentId),
    CheckInvocationStatus(ContentId),
    PullingMessage(ContentId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvocationStatus {
    Running,
    WaitingFunctionCall,
    Failed,
    Completed,
}

#[callback]
pub fn on_response(id: u64, response: CallResult<HttpResponse>) {
    let key = crate::trie::http_trace_key(id);
    match storage::get(&key) {
        Ok(Some(v)) => {
            if let Ok(call_type) = HttpCallType::decode(&mut &v[..]) {
                if let Err(e) = untrace(&key, call_type, response) {
                    vrs_core_sdk::println!("{}", e);
                }
            }
        }
        Ok(None) => {}
        Err(_e) => {}
    }
}

fn trace(id: u64, call_type: HttpCallType) -> Result<(), RuntimeError> {
    let key = crate::trie::http_trace_key(id);
    storage::put(&key, &call_type.encode())
}

fn parse_response<T: DeserializeOwned>(response: CallResult<HttpResponse>) -> Result<T, String> {
    let response = response.map_err(|e| e.to_string())?;
    let data = serde_json::from_slice::<T>(&response.body)
        .map_err(|e| format!("unable to deserialize body from llm: {:?}", e))?;
    Ok(data)
}

fn untrace(
    key: &[u8],
    call_type: HttpCallType,
    response: CallResult<HttpResponse>,
) -> Result<(), String> {
    storage::del(key).map_err(|e| e.to_string())?;
    match call_type {
        HttpCallType::CreatingAgent(community_id) => {
            let assistant_id = openai::resolve_assistant_id(response)?;
            set_agent_id(community_id, assistant_id)?;
        }
        HttpCallType::AppendingMessage(_content_id) => {}
        HttpCallType::InvokingLLM(content_id) | HttpCallType::CheckInvocationStatus(content_id) => {
            // TODO define the invocation object to replace the openai::RunObject
            let run = parse_response::<openai::RunObject>(response)?;
            let status = match run.status.as_str() {
                "queued" | "in_progress" => InvocationStatus::Running,
                "completed" => InvocationStatus::Completed,
                "requires_action" => InvocationStatus::WaitingFunctionCall,
                _ => InvocationStatus::Failed,
            };
            match status {
                InvocationStatus::Running => {
                    let _ = set_timer!(
                        std::time::Duration::from_secs(5),
                        check_invocation_status,
                        content_id,
                        run.thread_id.clone(),
                        run.id.clone(),
                    );
                }
                InvocationStatus::WaitingFunctionCall => {
                    // TODO submit function call
                }
                InvocationStatus::Completed => {
                    pull_messages(content_id, &run.thread_id, &run.id)?;
                }
                InvocationStatus::Failed => {
                    vrs_core_sdk::eprintln!("{:?}", serde_json::to_string(&run))
                }
            }
        }
        HttpCallType::PullingMessage(content_id) => {
            // TODO
            let messages = openai::resolve_messages(response)?;
            let reply = messages
                .data
                .into_iter()
                .find(|m| m.role == openai::MessageRole::assistant);
            if let Some(reply) = reply {
                let content = reply
                    .content
                    .into_iter()
                    .filter(|c| c.content_type == "text")
                    .map(|c| c.text.value)
                    .collect::<Vec<String>>()
                    .join("\n");
                let id = crate::allocate_comment_id(content_id)?;
                let community_id = (id >> 64) as CommunityId;
                let community_key = crate::trie::to_community_key(community_id);
                let community: Community =
                    crate::find(&community_key)?.ok_or("Community not found".to_string())?;
                let agent_account = community.agent_account();
                let key = crate::trie::to_content_key(id);
                let reply_to =
                    crate::trie::is_comment(content_id).then(|| hex::encode(content_id.encode()));
                let comment = Comment {
                    id: hex::encode(id.encode()),
                    content,
                    image: None,
                    author: agent_account,
                    mention: vec![],
                    reply_to,
                    created_time: timer::now() as i64,
                };
                storage::put(&key, comment.encode()).map_err(|e| e.to_string())?;
                crate::save_event(Event::CommentPosted(id))?;
            }
        }
    }
    Ok(())
}

#[timer]
pub(crate) fn check_invocation_status(
    content_id: ContentId,
    session_id: String,
    invoke_id: String,
) -> Result<(), String> {
    let key = get_llm_key(OPENAI)?;
    let id = openai::retrieve_run(key, &session_id, &invoke_id)?;
    trace(id, HttpCallType::CheckInvocationStatus(content_id)).map_err(|e| e.to_string())
}

pub(crate) fn init_agent(community: &str, prompt: &str) -> Result<(), String> {
    let community_id = crate::name_to_community_id(community).expect("caller check;");
    let key = get_llm_key(OPENAI)?;
    let id = openai::create_assistant(community, prompt, key)?;
    trace(id, HttpCallType::CreatingAgent(community_id)).map_err(|e| e.to_string())
}

pub(crate) fn create_session_and_run(thread: &Thread) -> Result<(), String> {
    let community_id = thread.community_id();
    let assistant_id = get_agent_id(community_id)?;
    let key = get_llm_key(OPENAI)?;
    let id = openai::create_thread_and_run(&assistant_id, key, thread)?;
    trace(id, HttpCallType::InvokingLLM(thread.id())).map_err(|e| e.to_string())
}

pub(crate) fn pull_messages(
    content_id: ContentId,
    session_id: &str,
    invoke_id: &str,
) -> Result<(), String> {
    let key = get_llm_key(OPENAI)?;
    let id = openai::list_messages(key, session_id, invoke_id)?;
    trace(id, HttpCallType::PullingMessage(content_id)).map_err(|e| e.to_string())
}
