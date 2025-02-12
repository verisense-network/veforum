use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use vemodel::Thread;
use vrs_core_sdk::http::{self, HttpMethod, HttpRequest, HttpResponse, RequestHead};
use vrs_core_sdk::CallResult;

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

pub(crate) fn resolve_assistant_id(response: CallResult<HttpResponse>) -> Result<String, String> {
    let assistant: AssistantObject = super::parse_response(response)?;
    Ok(assistant.id)
}

pub(crate) fn create_thread_and_run(
    assistant_id: &str,
    key: String,
    thread: &Thread,
) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let mut body = serde_json::json!({
        "assistant_id": assistant_id,
        "thread": {
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string(&thread).expect("json;qed")
                }],
            }]
        }
    });
    if let Some(ref img) = thread.image {
        body["thread"]["messages"][0]["content"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
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

pub(crate) fn retrieve_run(key: String, session_id: &str, invoke_id: &str) -> Result<u64, String> {
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

// TODO
pub(crate) fn resolve_run_id_and_status(
    response: CallResult<HttpResponse>,
) -> Result<(String, super::InvocationStatus), String> {
    let run: RunObject = super::parse_response(response)?;
    let status = match run.status.as_str() {
        "queued" | "in_progress" => super::InvocationStatus::Running,
        "completed" => super::InvocationStatus::Completed,
        "requires_action" => super::InvocationStatus::WaitingFunctionCall,
        _ => super::InvocationStatus::Failed,
    };
    Ok((run.id, status))
}

pub(crate) fn list_messages(key: String, session_id: &str) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("OpenAI-Beta".to_string(), "assistants=v2".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Get,
            uri: format!("https://api.openai.com/v1/threads/{}/messages", session_id),
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
    pub required_action: Option<BTreeMap<String, String>>,
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
