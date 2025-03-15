pub(crate) mod bsc;
pub(crate) mod openai;
// pub(crate) mod solana;

use std::str::FromStr;
use primitive_types::H256;

use serde::de::DeserializeOwned;
use vemodel::*;
use vrs_core_sdk::{
    callback, codec::*, error::RuntimeError, http::*, set_timer, storage, timer, CallResult,
};
use crate::agent::bsc::{check_gas_price, TransactionDetails, untrace_issue_tx};

pub const OPENAI: [u8; 4] = *b"opai";
pub const DEEPSEEK: [u8; 4] = *b"dpsk";
pub const GASPRICE_STORAGE_KEY: &str = "gas_price";
pub const PENDING_ISSUE_KEY: &str = "pending_issue";

pub const DEEPSEEK_API_HOST: &'static str = "https://api.deepseek.ai";

pub(crate) fn set_sys_key(vendor: [u8; 4], key: String) -> Result<(), String> {
    let ty = crate::trie::llm_key(vendor);
    storage::put(&ty, key.into_bytes()).map_err(|e| e.to_string())
}

pub(crate) fn get_sys_key(vendor: [u8; 4]) -> Result<String, String> {
    let ty = crate::trie::llm_key(vendor);
    storage::get(&ty)
        .map_err(|e| e.to_string())?
        .map(|b| String::from_utf8(b))
        .transpose()
        .map_err(|_| "Invalid LLM key".to_string())?
        .ok_or("LLM key not found".to_string())
}

fn decorate_prompt(community: &str, account: &AccountId, prompt: &str) -> String {
    format!(
        "你是一名论坛{}版块的管理员，论坛程序将会把每篇帖子或者at你的评论以json格式发送给你。其中，author和mention的数据类型为BSC链地址，以0x开头，表示用户id，你自己的user_id={}。你需要阅读这些内容，并且根据本版块的规则进行响应，本版块的规则如下：\n{}",
        community,
        account,
        prompt
    )
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum HttpCallType {
    CreatingAgent(CommunityId),
    AppendingMessage(ContentId),
    InvokingLLM(ContentId),
    CheckInvocationStatus(ContentId),
    PullingMessage(ContentId),
    SubmittingToolCall(ContentId),
    CheckingTx(CommunityId),
    SendIssueTx(CommunityId),
    QueryBscGasPrice,
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

pub(crate) fn trace(id: u64, call_type: HttpCallType) -> Result<(), RuntimeError> {
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
        HttpCallType::CheckingTx(community_id) => {
            let key = crate::trie::to_community_key(community_id);
            let mut community =
                crate::find::<Community>(&key)?.ok_or("Community not found".to_string())?;
            let agent_addr = community.agent_pubkey.to_string();
            match bsc::on_checking_bnb_transfer(&agent_addr, response)
                .map_err(|e| e.to_string())
                .inspect_err(|e| eprintln!("failed to resolve solana RPC response, {:?}", e))
            {
                Ok(Some(tx)) => {
                    if tx.amount_received >= crate::MIN_ACTIVATE_FEE {
                        // TODO move this to after token issued
                        //token issue

                        storage::put(
                            &crate::trie::to_balance_key(community_id, community.agent_pubkey),
                            community.token_info.total_issuance.encode(),
                        )
                        .map_err(|e| e.to_string())?;
                        crate::agent::init_agent(&community)?;
                        community.status = CommunityStatus::Active;

                    } else {
                        community.status = CommunityStatus::CreateFailed(
                            "The received amount is not enough".to_string(),
                        );
                    }
                }
                Ok(None) => {
                    community.status =
                        CommunityStatus::CreateFailed("No transfer found".to_string());
                }
                Err(_) => {
                    community.status = CommunityStatus::CreateFailed(
                        "Failed to resolve the tx from BSC RPC".to_string(),
                    );
                }
            }
            crate::save_event(Event::CommunityUpdated(community.id()))?;
            crate::save(&key, &community)?;
        }
        HttpCallType::CreatingAgent(community_id) => {
            let assistant_id = openai::resolve_assistant_id(response)?;
            let key = crate::trie::to_community_key(community_id);
            let mut community =
                crate::find::<Community>(&key)?.ok_or("Community not found".to_string())?;
            community.llm_assistant_id = assistant_id;
            crate::save(&key, &community)?;
        }
        HttpCallType::AppendingMessage(content_id) => {
            run(content_id)?;
        }
        HttpCallType::InvokingLLM(content_id)
        | HttpCallType::CheckInvocationStatus(content_id)
        | HttpCallType::SubmittingToolCall(content_id) => {
            // TODO define the invocation object to replace the openai::RunObject
            let run = parse_response::<openai::RunObject>(response)?;
            let status = match run.status.as_str() {
                "queued" | "in_progress" => InvocationStatus::Running,
                "completed" => InvocationStatus::Completed,
                "requires_action" => InvocationStatus::WaitingFunctionCall,
                _ => InvocationStatus::Failed,
            };
            if vemodel::is_thread(content_id) {
                let mut thread = crate::find::<Thread>(&crate::trie::to_content_key(content_id))?
                    .ok_or("Thread not found".to_string())?;
                if thread.llm_session_id.is_empty() {
                    thread.llm_session_id = run.thread_id.clone();
                    crate::save(&crate::trie::to_content_key(content_id), &thread)?;
                }
            }
            let community_id = vemodel::get_belongs_to(content_id);
            let community_key = crate::trie::to_community_key(community_id);
            let community = crate::find::<Community>(&community_key)?
                .ok_or("Community not found".to_string())?;
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
                    if let Some(actions) = run.required_action {
                        let call_result = actions
                            .submit_tool_outputs
                            .tool_calls
                            .into_iter()
                            .map(|call| {
                                (
                                    call.id.clone(),
                                    match call_tool(
                                        &community,
                                        &call.function.name,
                                        &call.function.arguments,
                                    ) {
                                        Ok(result) => result,
                                        Err(e) => e,
                                    },
                                )
                            })
                            .collect::<Vec<(String, String)>>();
                        submit_tool_call(
                            community.llm_vendor.key(),
                            content_id,
                            &run.thread_id,
                            &run.id,
                            call_result,
                        )?;
                    }
                }
                InvocationStatus::Completed => {
                    pull_messages(
                        community.llm_vendor.key(),
                        content_id,
                        &run.thread_id,
                        &run.id,
                    )?;
                }
                InvocationStatus::Failed => {
                    vrs_core_sdk::eprintln!("{:?}", serde_json::to_string(&run))
                }
            }
        }
        HttpCallType::PullingMessage(content_id) => {
            // TODO define the message object to replace the openai::MessageObject
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
                let key = crate::trie::to_content_key(id);
                let reply_to =
                    crate::trie::is_comment(content_id).then(|| hex::encode(content_id.encode()));
                let comment = Comment {
                    id: hex::encode(id.encode()),
                    content: crate::compress(content.as_ref())?,
                    images: vec![],
                    author: community.agent_pubkey,
                    mention: vec![],
                    reply_to,
                    created_time: timer::now() as i64,
                };
                crate::save(&key, &comment)?;
                crate::save_event(Event::CommentPosted(id))?;
            }
        }
        HttpCallType::QueryBscGasPrice => {
            if let Ok(Some(u)) = check_gas_price(response) {
                crate::save(GASPRICE_STORAGE_KEY.as_bytes(), &u)?;
            }
        }
        HttpCallType::SendIssueTx(community) => {
            match untrace_issue_tx(response) {
                Ok(tx) => {
                    match tx {
                        None => {}
                        Some(_) => {}
                    }
                }
                Err(e) => {
                    vrs_core_sdk::eprintln!("untrace issue tx error: {}", e.to_string());
                }
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
    let community_key = crate::trie::to_community_key(vemodel::get_belongs_to(content_id));
    let community =
        crate::find::<Community>(&community_key)?.ok_or("Community not found".to_string())?;
    let id = openai::retrieve_run(community.llm_vendor.key(), &session_id, &invoke_id)?;
    trace(id, HttpCallType::CheckInvocationStatus(content_id)).map_err(|e| e.to_string())
}

pub(crate) fn init_agent(community: &Community) -> Result<(), String> {
    let prompt = decorate_prompt(&community.name, &community.agent_pubkey, &community.prompt);
    let id = openai::create_assistant(community.llm_vendor.key(), &community.name, &prompt)?;
    trace(id, HttpCallType::CreatingAgent(community.id())).map_err(|e| e.to_string())
}

pub(crate) fn create_session_and_run(
    community: &Community,
    thread: &Thread,
    text: &str,
) -> Result<(), String> {
    let id = openai::create_thread_and_run(
        community.llm_vendor.key(),
        &community.llm_assistant_id,
        thread,
        text,
    )?;
    trace(id, HttpCallType::InvokingLLM(thread.id())).map_err(|e| e.to_string())
}

fn run(content_id: ContentId) -> Result<(), String> {
    if vemodel::is_thread(content_id) {
        return Ok(());
    }
    let comment = crate::find::<Comment>(&crate::trie::to_content_key(content_id))?
        .ok_or("Comment not found".to_string())?;
    let community_id = comment.community_id();
    let community = crate::find::<Community>(&crate::trie::to_community_key(community_id))?
        .ok_or("Community not found".to_string())?;
    let thread_id = comment.thread_id();
    let thread = crate::find::<Thread>(&crate::trie::to_content_key(thread_id))?
        .ok_or("Thread not found".to_string())?;
    let id = openai::create_run(
        community.llm_vendor.key(),
        &community.llm_assistant_id,
        &thread.llm_session_id,
    )?;
    trace(id, HttpCallType::InvokingLLM(content_id)).map_err(|e| e.to_string())
}

pub(crate) fn append_message_then_run(
    community: &Community,
    thread: &Thread,
    comment: &Comment,
    text: &str,
) -> Result<(), String> {
    let id = openai::append_message(
        community.llm_vendor.key(),
        &thread.llm_session_id,
        comment,
        text,
    )?;
    trace(id, HttpCallType::AppendingMessage(comment.id())).map_err(|e| e.to_string())
}

fn pull_messages(
    key: &str,
    content_id: ContentId,
    session_id: &str,
    invoke_id: &str,
) -> Result<(), String> {
    let id = openai::list_messages(key, session_id, invoke_id)?;
    trace(id, HttpCallType::PullingMessage(content_id)).map_err(|e| e.to_string())
}

fn submit_tool_call(
    key: &str,
    content_id: ContentId,
    session_id: &str,
    invoke_id: &str,
    call_result: Vec<(String, String)>,
) -> Result<(), String> {
    let id = openai::submit_tool_outputs(key, session_id, invoke_id, call_result)?;
    trace(id, HttpCallType::SubmittingToolCall(content_id)).map_err(|e| e.to_string())
}

fn call_tool(on: &Community, func: &str, params: &str) -> Result<String, String> {
    let json: serde_json::Value =
        serde_json::from_str(params).map_err(|_| "Invalid parameters".to_string())?;
    match func {
        "agent_balance" => crate::balance_of(on.id(), on.agent_pubkey).map(|v| v.to_string()),
        "transfer" => {
            let recipient = json["recipient"].as_str().ok_or("Invalid recipient")?;
            let recipient =
                AccountId::from_str(recipient).map_err(|_| "Invalid param: recipient")?;
            let amount = json["amount"].as_u64().ok_or("Invalid amount")?;
            crate::transfer(on.id(), on.agent_pubkey, recipient, amount).map(|_| "Ok".to_string())
        }
        "balance_of" => {
            let account = json["account"].as_str().ok_or("Invalid account")?;
            let account = AccountId::from_str(account).map_err(|_| "Invalid param: account")?;
            crate::balance_of(on.id(), account).map(|v| v.to_string())
        }
        _ => Err("Invalid tool".to_string()),
    }
}

pub(crate) fn check_transfering(community: &Community, tx: String) -> Result<(), String> {
    match community.status {
        CommunityStatus::PendingCreation | CommunityStatus::Active => Ok(()),
        CommunityStatus::WaitingTx(_)
        | CommunityStatus::Frozen(_)
        | CommunityStatus::CreateFailed(_) => {
            let id = bsc::initiate_checking_bnb_transfer(&tx)?;
            trace(id, HttpCallType::CheckingTx(community.id())).map_err(|e| e.to_string())
        }
    }
}