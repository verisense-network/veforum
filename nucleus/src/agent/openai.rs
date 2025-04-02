use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vemodel::{Comment, Thread};
use vrs_core_sdk::http::{self, HttpMethod, HttpRequest, HttpResponse, RequestHead};
use vrs_core_sdk::CallResult;

pub(crate) fn create_assistant(key: &str, name: &str, prompt: &str) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let body = serde_json::json!({
        "instructions": prompt,
        "model": "gpt-4o",
        "name": name,
        "tools": [{
            "type": "function",
            "function": {
                "name": "transfer",
                "description": "Transfer funds to another user",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "recipient": {
                            "type": "string",
                            "description": "The recipient user_id"
                        },
                        "amount": {
                            "type": "number",
                            "description": "The amount(integer) to transfer"
                        }
                    },
                    "required": ["recipient", "amount"],
                    "additionalProperties": false
                },
                "strict": true
            }
        },{
            "type": "function",
            "function": {
                "name": "agent_balance",
                "description": "Check the agent's balance"
            }
        },{
            "type": "function",
            "function": {
                "name": "balance_of",
                "description": "Query the balance of a user",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "account_id": {
                            "type": "string",
                            "description": "The user_id to query"
                        }
                    },
                    "required": ["account_id"],
                    "additionalProperties": false
                },
                "strict": true
            }
        }]
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

pub(crate) fn resolve_assistant_id(response: CallResult<HttpResponse>) -> Result<String, String> {
    let assistant: AssistantObject = super::parse_response(response)?;
    Ok(assistant.id)
}

pub(crate) fn create_thread_and_run(
    key: &str,
    assistant_id: &str,
    thread: &Thread,
    text: &str,
) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let text_msg = serde_json::json!({
        "id": thread.id,
        "title": thread.title,
        "content": text,
        "author": thread.author,
        "mention": thread.mention,
        "created_time": thread.created_time,
    });
    let mut body = serde_json::json!({
        "assistant_id": assistant_id,
        "thread": {
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string(&text_msg).expect("json;qed")
                }],
            }]
        }
    });
    let contents = body["thread"]["messages"][0]["content"]
        .as_array_mut()
        .unwrap();
    for img in thread.images.iter() {
        contents.push(serde_json::json!({
            "type": "image_url",
            "image_url": { "url": img },
        }));
    }
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Post,
            uri: "https://api.openai.com/v1/threads/runs".to_string(),
            headers,
        },
        body: serde_json::to_vec(&body).expect("json;qed"),
    })
    .map_err(|e| e.to_string())?;
    Ok(response)
}

pub(crate) fn create_run(key: &str, assistant_id: &str, thread_id: &str) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let body = serde_json::json!({
        "assistant_id": assistant_id,
        "additional_instructions": "this is a comment",
    });
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Post,
            uri: format!("https://api.openai.com/v1/threads/{}/runs", thread_id),
            headers,
        },
        body: serde_json::to_vec(&body).expect("json;qed"),
    })
    .map_err(|e| e.to_string())?;
    Ok(response)
}

pub(crate) fn submit_tool_outputs(
    key: &str,
    session_id: &str,
    invoke_id: &str,
    call_result: Vec<(String, String)>,
) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let call_result = call_result
        .into_iter()
        .map(|(id, out)| {
            serde_json::json!({
                "tool_call_id": id,
                "output": out,
            })
        })
        .collect::<Vec<_>>();
    let body = serde_json::json!({
        "tool_outputs": call_result,
    });
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Post,
            uri: format!(
                "https://api.openai.com/v1/threads/{}/runs/{}/submit_tool_outputs",
                session_id, invoke_id
            ),
            headers,
        },
        body: serde_json::to_vec(&body).expect("json;qed"),
    })
    .map_err(|e| e.to_string())?;
    Ok(response)
}

pub(crate) fn retrieve_run(key: &str, session_id: &str, invoke_id: &str) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Get,
            uri: format!(
                "https://api.openai.com/v1/threads/{}/runs/{}",
                session_id, invoke_id
            ),
            headers,
        },
        body: vec![],
    })
    .map_err(|e| e.to_string())?;
    Ok(response)
}

pub(crate) fn append_message(
    key: &str,
    session_id: &str,
    comment: &Comment,
    text: &str,
) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let text_msg = serde_json::json!({
        "id": comment.id,
        "content": text,
        "author": comment.author,
        "mention": comment.mention,
        "created_time": comment.created_time,
    });
    let mut body = serde_json::json!({
        "role": "user",
        "content": [{
            "type": "text",
            "text": serde_json::to_string(&text_msg).expect("json;qed")
        }],
    });
    let contents = body["content"].as_array_mut().unwrap();
    for img in comment.images.iter() {
        contents.push(serde_json::json!({
            "type": "image_url",
            "image_url": { "url": img },
        }));
    }
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Post,
            uri: format!("https://api.openai.com/v1/threads/{}/messages", session_id),
            headers,
        },
        body: serde_json::to_vec(&body).expect("json;qed"),
    })
    .map_err(|e| e.to_string())?;
    Ok(response)
}

pub(crate) fn list_messages(key: &str, session_id: &str, invoke_id: &str) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Get,
            uri: format!(
                "https://api.openai.com/v1/threads/{}/messages?run_id={}",
                session_id, invoke_id
            ),
            headers,
        },
        body: vec![],
    })
    .map_err(|e| e.to_string())?;
    Ok(response)
}

pub(crate) fn resolve_messages(response: CallResult<HttpResponse>) -> Result<ListMessage, String> {
    super::parse_response(response)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssistantObject {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    pub tools: Vec<Tools>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ToolResource>,
    pub metadata: Option<BTreeMap<String, String>>,
    pub headers: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Tools {
    CodeInterpreter,
    FileSearch(ToolsFileSearch),
    Function(ToolsFunction),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolResource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_interpreter: Option<CodeInterpreter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search: Option<FileSearch>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CodeInterpreter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileSearch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_store_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_stores: Option<VectorStores>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VectorStores {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolsFileSearch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search: Option<ToolsFileSearchObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolsFunction {
    pub function: Function,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolsFileSearchObject {
    pub max_num_results: Option<u8>,
    pub ranking_options: Option<FileSearchRankingOptions>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileSearchRankingOptions {
    pub ranker: Option<FileSearchRanker>,
    pub score_threshold: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum FileSearchRanker {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "default_2024_08_21")]
    Default2024_08_21,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: FunctionParameters,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct FunctionParameters {
    #[serde(rename = "type")]
    pub schema_type: JSONSchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, Box<JSONSchemaDefine>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JSONSchemaType {
    Object,
    Number,
    String,
    Array,
    Null,
    Boolean,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct JSONSchemaDefine {
    #[serde(rename = "type")]
    pub schema_type: Option<JSONSchemaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, Box<JSONSchemaDefine>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JSONSchemaDefine>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunObject {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub thread_id: String,
    pub assistant_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_action: Option<ActionRequired>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<LastError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    pub model: String,
    pub instructions: Option<String>,
    pub tools: Vec<Tools>,
    pub metadata: BTreeMap<String, String>,
    pub headers: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LastError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageObject {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub thread_id: String,
    pub role: MessageRole,
    pub content: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    pub metadata: Option<BTreeMap<String, String>>,
    pub headers: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attachment {
    pub file_id: Option<String>,
    pub tools: Vec<Tool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum MessageRole {
    user,
    system,
    assistant,
    function,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: ContentText,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContentText {
    pub value: String,
    pub annotations: Vec<ContentTextAnnotations>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListMessage {
    pub object: String,
    pub data: Vec<MessageObject>,
    pub first_id: String,
    pub last_id: String,
    pub has_more: bool,
    pub headers: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ContentTextAnnotations {
    FileCitation(ContentTextAnnotationsFileCitationObject),
    FilePath(ContentTextAnnotationsFilePathObject),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContentTextAnnotationsFileCitationObject {
    pub text: String,
    pub file_citation: FileCitation,
    pub start_index: u32,
    pub end_index: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileCitation {
    pub file_id: String,
    pub quote: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContentTextAnnotationsFilePathObject {
    pub text: String,
    pub file_path: FilePath,
    pub start_index: u32,
    pub end_index: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilePath {
    pub file_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionRequired {
    pub submit_tool_outputs: ToolCalls,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolCalls {
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub function: FunctionCallWithParameter,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionCallWithParameter {
    pub name: String,
    pub arguments: String,
}
