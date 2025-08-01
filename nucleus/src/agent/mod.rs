pub(crate) mod bsc;
pub mod contract;
pub(crate) mod openai;
pub mod rewards;
// pub(crate) mod solana;

use crate::trie::{to_community_key, to_invitecode_amt_key};
use crate::{find, get_account_info, save, trie, try_find_community, MIN_ACTIVATE_FEE};
use const_hex::ToHexExt;
use serde::de::DeserializeOwned;
use std::str::FromStr;
use vemodel::*;
use vrs_core_sdk::{
    callback, codec::*, error::RuntimeError, http::*, set_timer, storage, timer, CallResult,
};

pub const OPENAI: [u8; 4] = *b"opai";
pub const DEEPSEEK: [u8; 4] = *b"dpsk";

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

fn decorate_prompt(
    community: &str,
    account: &AccountId,
    prompt: &str,
    token_info: &TokenMetadata,
) -> String {
    format!(
        "你是{} community的管理员，程序将会把每篇帖子或者对你的评论以json格式传递给你。
其中，author和mention的数据类型为BSC链地址，以0x开头，表示用户id，你自己的user_id={}。
每个community都有一种专属的数字货币，你可以调用工具来操作属于你的数字货币或者读取用户的账户余额。
属于你的community的数字货币信息为：{}。
你的每次回复语言种类应该跟用户每次使用的语言保持一致。
{}",
        community,
        account,
        serde_json::to_string(token_info).unwrap(),
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
    CheckingActivateTx(CommunityId),
    SendIssueTx(CommunityId),
    QueryBscGasPrice,
    QueryIssueResult(CommunityId, String),
    CheckingInviteTx(CommunityId),
    CheckingPayToJoinTx(CommunityId),
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
                    vrs_core_sdk::println!("untrace error>>>>>>>>>>>>.:{}", e);
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
        HttpCallType::QueryBscGasPrice => {
            if let Ok(Some(u)) = bsc::on_checking_gas_price(response) {
                vrs_core_sdk::println!("update bsc gasprice to {}", u);
                crate::save(&trie::GASPRICE_STORAGE_KEY.to_be_bytes(), &u)?;
            }
        }
        HttpCallType::CheckingActivateTx(community_id) => {
            let mut community = try_find_community(community_id)?;
            let agent_addr = community.agent_pubkey.to_string();
            match bsc::on_checking_bnb_transfer(&agent_addr, response).map_err(|e| e.to_string()) {
                Ok(Some(tx)) => match community.status.clone() {
                    CommunityStatus::WaitingTx(min_fee) => {
                        if tx.amount_received >= min_fee {
                            bsc::issue_token(&community, &community_id)?;
                            community.status = CommunityStatus::PendingCreation;
                            crate::save(&trie::to_community_key(community_id), &community)?;
                            let mut account = get_account_info(community.creator)?;
                            account.last_transfer_block = tx.block_number;
                            crate::save(
                                &trie::to_account_key(community.creator),
                                &AccountData::Pubkey(account),
                            )?;
                        }
                    }
                    _ => {}
                },
                // we don't reply
                _ => {}
            }
        }
        HttpCallType::CheckingPayToJoinTx(community_id) => {
            let mut community = try_find_community(community_id)?;
            let addr = community.agent_pubkey.to_string();
            let tx = bsc::on_checking_bnb_transfer(&addr, response)
                .map_err(|e| e.to_string())?
                .ok_or("No tx found".to_string())?;
            if let CommunityMode::PayToJoin(fee) = community.mode {
                if tx.amount_received >= fee {
                    let account_id = H160::from_str(&tx.sender)?;
                    let mut account = get_account_info(account_id)?;
                    if account.last_transfer_block < tx.block_number {
                        account.last_transfer_block = tx.block_number;
                        crate::save(
                            &trie::to_account_key(account.address),
                            &AccountData::Pubkey(account),
                        )?;
                        let permission_key = trie::to_permission_key(community_id, account_id);
                        let _ = crate::save(permission_key.as_ref(), &1u32);
                        let amount: u64 = tx.amount_received.try_into().unwrap();
                        let creator_share = amount * 7 / 10;
                        let platform_share = amount - creator_share;
                        community.creator_bnb_benefit += creator_share;
                        community.platform_bnb_benefit += platform_share;
                        let key = trie::to_community_key(community_id);
                        crate::save(&key, &community)?;
                    }
                }
            }
        }
        HttpCallType::SendIssueTx(community_id) => match bsc::on_issuing_tx(response) {
            Ok(Some(tx)) => {
                let mut community = try_find_community(community_id)?;
                community.status = CommunityStatus::TokenIssued(tx.to_string());
                crate::save(&trie::to_community_key(community_id), &community)?;
                let _ = set_timer!(
                    std::time::Duration::from_secs(5),
                    check_issue_token_tx,
                    community_id,
                    tx.encode_hex_with_prefix()
                );
            }
            _ => {
                let mut community = crate::try_find_community(community_id)?;
                community.status = CommunityStatus::WaitingTx(MIN_ACTIVATE_FEE);
                crate::save(&to_community_key(community_id), &community)?;
            }
        },
        HttpCallType::QueryIssueResult(community_id, tx) => {
            match bsc::on_checking_issue_result(response) {
                Ok((Some(fund_contract), token_contract)) => {
                    let mut community = crate::try_find_community(community_id)?;
                    let contract_addr = AccountId::from_str(fund_contract.as_str())?;
                    community.agent_contract = Some(contract_addr);
                    if community.token_info.new_issue {
                        community.token_info.contract = token_contract
                            .map(|c| AccountId::from_str(c.as_str()).unwrap())
                            .ok_or("The tx should include a token contract".to_string())?;
                        storage::put(
                            &crate::trie::to_balance_key(community_id, community.agent_pubkey),
                            community.token_info.total_issuance.encode(),
                        )
                        .map_err(|e| e.to_string())?;
                    }
                    crate::agent::init_agent(&community)?;
                    community.status = CommunityStatus::Active;
                    let community_key = to_community_key(community.id());
                    crate::save(&community_key, &community)?;
                    crate::save_event(Event::CommunityUpdated(community.id()))?;
                }
                _ => {
                    let _ = set_timer!(
                        std::time::Duration::from_secs(5),
                        check_issue_token_tx,
                        community_id,
                        tx,
                    );
                }
            }
        }
        HttpCallType::CreatingAgent(community_id) => {
            let assistant_id = openai::resolve_assistant_id(response)?;
            let mut community = crate::try_find_community(community_id)?;
            community.llm_assistant_id = assistant_id;
            crate::save(&trie::to_community_key(community_id), &community)?;
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
            let community = crate::try_find_community(community_id)?;
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
                    vrs_core_sdk::println!("{:?}", serde_json::to_string(&run))
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
                let community = crate::try_find_community(community_id)?;
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
        HttpCallType::CheckingInviteTx(community_id) => {
            let community = crate::try_find_community(community_id)?;
            let agent_addr = community.agent_pubkey.to_string();
            match bsc::on_checking_bnb_transfer(&agent_addr, response).map_err(|e| e.to_string()) {
                Ok(Some(tx)) => {
                    let sender = AccountId::from_str(tx.sender.as_str())?;
                    if sender == community.creator {
                        let mut account = get_account_info(sender.clone())?;
                        if account.last_transfer_block < tx.block_number {
                            account.last_transfer_block = tx.block_number;
                            if let CommunityMode::InviteOnly = community.mode {
                                let key = trie::to_account_key(sender);
                                storage::put(&key, AccountData::Pubkey(account).encode())
                                    .map_err(|e| e.to_string())?;
                                let increased = (tx.amount_received / MIN_ACTIVATE_FEE) as u64;
                                let invite_amount_key = to_invitecode_amt_key(community_id, sender);
                                let tickets = find::<u64>(invite_amount_key.as_ref())?
                                    .unwrap_or_default()
                                    + increased;
                                save(invite_amount_key.as_ref(), &tickets)?;
                            }
                        };
                    }
                }
                _ => {
                    vrs_core_sdk::println!("invite tx is none");
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
    let community = crate::try_find_community(vemodel::get_belongs_to(content_id))?;
    let id = openai::retrieve_run(community.llm_vendor.key(), &session_id, &invoke_id)?;
    trace(id, HttpCallType::CheckInvocationStatus(content_id)).map_err(|e| e.to_string())
}

#[timer]
pub(crate) fn check_issue_token_tx(
    community_id: CommunityId,
    tx_hash: String,
) -> Result<(), String> {
    let id = bsc::initiate_query_bsc_transaction(&tx_hash)?;
    trace(id, HttpCallType::QueryIssueResult(community_id, tx_hash)).map_err(|e| e.to_string())
}

pub(crate) fn init_agent(community: &Community) -> Result<(), String> {
    let prompt = decorate_prompt(
        &community.name,
        &community.agent_pubkey,
        &community.prompt,
        &community.token_info,
    );
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
        "agent_balance" => crate::balance_of(on.id(), on.agent_pubkey),
        "transfer" => {
            let recipient = json["recipient"].as_str().ok_or("Invalid recipient")?;
            let recipient =
                AccountId::from_str(recipient).map_err(|_| "Invalid param: recipient")?;
            let amount = json["amount"].as_u64().ok_or("Invalid amount")? as u128;
            crate::transfer(on.id(), on.agent_pubkey, recipient, amount).map(|_| "Ok".to_string())
        }
        "balance_of" => {
            let account = json["account"].as_str().ok_or("Invalid account")?;
            let account = AccountId::from_str(account).map_err(|_| "Invalid param: account")?;
            crate::balance_of(on.id(), account)
        }
        _ => Err("Invalid tool".to_string()),
    }
}

pub(crate) fn check_transfering(community: &Community, tx: String) -> Result<(), String> {
    match community.status.clone() {
        CommunityStatus::PendingCreation | CommunityStatus::Active => Ok(()),
        CommunityStatus::TokenIssued(_) => Ok(()),
        CommunityStatus::WaitingTx(_)
        | CommunityStatus::Frozen(_)
        | CommunityStatus::CreateFailed(_) => {
            let id = bsc::initiate_query_bsc_transaction(&tx)?;
            trace(id, HttpCallType::CheckingActivateTx(community.id())).map_err(|e| e.to_string())
        }
    }
}

pub(crate) fn check_fee(community: &Community, tx: String) -> Result<(), String> {
    let id = bsc::initiate_query_bsc_transaction(&tx)?;
    trace(id, HttpCallType::CheckingPayToJoinTx(community.id())).map_err(|e| e.to_string())
}
