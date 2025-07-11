extern crate core;

mod agent;
pub mod eth_types;
mod nucleus;
mod trie;

use crate::agent::rewards::generate_rewards;
use crate::eth_types::Address;
use crate::trie::{to_account_key, to_reward_payload_key, PERMISSION_KEY_PREFIX};
use sha2::{Digest, Sha256};
use vemodel::{
    Account, AccountData, AccountId, Community, CommunityId, ContentId, Event, EventId, LlmVendor,
    RewardPayload,
};
use vrs_core_sdk::{
    codec::{Decode, Encode},
    storage,
};

pub const MIN_ACTIVATE_FEE: u128 = 2_000_000_000_000_000;
pub const MIN_INVITE_FEE: u128 = 2_000_000_000_000_000;

pub(crate) fn from_llm_settings(
    llm_name: String,
    llm_api_host: Option<String>,
    llm_key: Option<String>,
) -> Result<LlmVendor, String> {
    match llm_name.as_ref() {
        "OpenAI" => {
            if llm_key.is_none() {
                let default_key = crate::agent::get_sys_key(crate::agent::OPENAI)?;
                Ok(LlmVendor::OpenAI { key: default_key })
            } else {
                Ok(LlmVendor::OpenAI {
                    key: llm_key.unwrap(),
                })
            }
        }
        "DeepSeek" => {
            false
                .then(|| ())
                .ok_or("DeepSeek is not supported".to_string())?;
            if llm_key.is_none() {
                let default_key = crate::agent::get_sys_key(crate::agent::DEEPSEEK)?;
                Ok(LlmVendor::DeepSeek {
                    key: default_key,
                    host: crate::agent::DEEPSEEK_API_HOST.to_string(),
                })
            } else {
                Ok(LlmVendor::DeepSeek {
                    key: llm_key.unwrap(),
                    host: llm_api_host.unwrap_or(crate::agent::DEEPSEEK_API_HOST.to_string()),
                })
            }
        }
        _ => Err("unsupported LLM vendor".to_string()),
    }
}

pub(crate) fn try_find_community(community_id: CommunityId) -> Result<Community, String> {
    let key = crate::trie::to_community_key(community_id);
    crate::find::<Community>(&key)?.ok_or("Community not found".to_string())
}

pub(crate) fn find<T: Decode>(key: &[u8]) -> Result<Option<T>, String> {
    let r = storage::get(key).map_err(|e| e.to_string())?;
    r.map(|d| T::decode(&mut &d[..]))
        .transpose()
        .map_err(|e| e.to_string())
}

pub(crate) fn save<T: Encode>(key: &[u8], value: &T) -> Result<(), String> {
    storage::put(key, value.encode()).map_err(|e| e.to_string())
}

pub(crate) fn name_to_community_id(name: &str) -> Option<CommunityId> {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let v = hasher.finalize();
    Some(CommunityId::from_be_bytes(v[..4].try_into().unwrap()))
}

pub(crate) fn save_event(event: Event) -> Result<(), String> {
    let event_id = allocate_event_id()?;
    let key = trie::to_event_key(event_id);
    storage::put(&key, event.encode()).map_err(|e| e.to_string())
}

pub(crate) fn allocate_event_id() -> Result<EventId, String> {
    let max = trie::to_event_key(EventId::MAX);
    match storage::search(&max, storage::Direction::Reverse).map_err(|e| e.to_string())? {
        Some((id, _)) => trie::to_event_id(&id).map(|v| v + 1),
        None => Ok(1),
    }
}

pub(crate) fn allocate_thread_id(community_id: CommunityId) -> Result<ContentId, String> {
    let start_key = trie::MIN_CONTENT_KEY | ((community_id as u128) << 64);
    let end_key = start_key | u64::MAX as u128;
    let r = storage::search(&end_key.to_be_bytes()[..], storage::Direction::Reverse)
        .map_err(|e| e.to_string())?
        .filter(|(k, _)| k.starts_with(&start_key.to_be_bytes()[..8]))
        .map(|(k, _)| trie::to_content_id(&k))
        .transpose()?
        .unwrap_or((community_id as u128) << 64);
    (r & 0xffffffff_00000000 < 0xffffffff_00000000)
        .then(|| ())
        .ok_or("We don't expect more than 4b threads in a community :)")?;
    Ok((r + 0x1_00000000) & (u128::MAX - 0xffffffff))
}

pub(crate) fn allocate_comment_id(thread_id: ContentId) -> Result<ContentId, String> {
    let start_key = trie::MIN_CONTENT_KEY | thread_id;
    let end_key = start_key | u32::MAX as u128;
    let r = storage::search(&end_key.to_be_bytes()[..], storage::Direction::Reverse)
        .map_err(|e| e.to_string())?
        .filter(|(k, _)| k.starts_with(&start_key.to_be_bytes()[..12]))
        .map(|(k, _)| trie::to_content_id(&k))
        .transpose()?
        .unwrap_or(thread_id);
    (r & 0xffffffff < 0xffffffff)
        .then(|| ())
        .ok_or("We don't expect more than 4b comments in a thread :)")?;
    Ok(r + 1)
}

pub(crate) fn get_account_info(account_id: AccountId) -> Result<Account, String> {
    let key = trie::to_account_key(account_id);
    match crate::find::<AccountData>(&key)? {
        Some(AccountData::Pubkey(data)) => Ok(data),
        Some(AccountData::AliasOf(id)) => {
            let key = trie::to_account_key(id);
            if let Some(AccountData::Pubkey(data)) = crate::find::<AccountData>(&key)? {
                Ok(data)
            } else {
                Err("account not found".to_string())
            }
        }
        None => Ok(Account::new(account_id)),
    }
}

pub(crate) fn get_account_count() -> Result<u128, String> {
    let prefix_vec = PERMISSION_KEY_PREFIX.to_be_bytes().to_vec();
    let mut count: u128 = 0;
    let mut last_key = prefix_vec.clone();

    loop {
        let batch = storage::get_range(&last_key, storage::Direction::Forward, 1000)
            .map_err(|e| e.to_string())?;

        if batch.is_empty() {
            break;
        }

        let filtered_count = batch
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix_vec))
            .count() as u128;

        count += filtered_count;

        if filtered_count < 1000 {
            break;
        }

        last_key = batch.last().unwrap().0.clone();
    }

    Ok(count)
}

#[allow(dead_code)]
pub(crate) fn save_account(
    account_id: AccountId,
    account_data: &AccountData,
) -> Result<(), String> {
    let key = to_account_key(account_id);
    crate::save(&key, account_data)
}

pub(crate) fn get_rewards(community_id: CommunityId, account_id: AccountId) -> Vec<RewardPayload> {
    let key = to_reward_payload_key(community_id, account_id);
    let v: Vec<RewardPayload> = crate::find(key.as_ref())
        .unwrap_or_default()
        .unwrap_or_default();
    v
}

pub(crate) fn get_nonce(account_id: AccountId) -> Result<u64, String> {
    let key = trie::to_account_key(account_id);
    match crate::find::<AccountData>(&key)? {
        Some(AccountData::Pubkey(data)) => Ok(data.nonce),
        Some(AccountData::AliasOf(_)) => Err("alias account could not be used to sign".to_string()),
        None => Ok(0),
    }
}

pub(crate) fn incr_nonce(account_id: AccountId, update_time: Option<u64>) -> Result<(), String> {
    let key = trie::to_account_key(account_id);
    let mut account = match crate::find::<AccountData>(&key)? {
        Some(AccountData::Pubkey(data)) => Ok(data),
        Some(AccountData::AliasOf(_)) => Err("alias account could not be used to sign".to_string()),
        None => Ok(Account::new(account_id)),
    }?;
    account.nonce += 1;
    if let Some(t) = update_time {
        account.last_post_at = t;
    }
    storage::put(&key, AccountData::Pubkey(account).encode()).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn transfer(
    community_id: CommunityId,
    from: AccountId,
    to: AccountId,
    amount: u128,
) -> Result<(), String> {
    let community = crate::try_find_community(community_id)?;
    let amount = amount * 10u128.pow(community.token_info.decimals as u32);
    let from_key = trie::to_balance_key(community_id.clone(), from);
    let from_balance = storage::get(&from_key)
        .map_err(|e| e.to_string())?
        .map(|d| u128::decode(&mut &d[..]).map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(0);
    let to_key = trie::to_balance_key(community_id.clone(), to);
    let to_balance = storage::get(&to_key)
        .map_err(|e| e.to_string())?
        .map(|d| u128::decode(&mut &d[..]).map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(0);
    if from_balance < amount {
        return Err("insufficient balance".to_string());
    }
    // TODO we need transaction
    storage::put(&from_key, (from_balance - amount).encode()).map_err(|e| e.to_string())?;
    storage::put(&to_key, (to_balance + amount).encode()).map_err(|e| e.to_string())?;
    let community = crate::try_find_community(community_id)?;
    if let Some(reward) = generate_rewards(Address::from(to.0.clone()), amount as u128, &community)
    {
        let key = to_reward_payload_key(community_id, to.clone());
        let mut v: Vec<RewardPayload> = crate::find(key.as_ref())?.unwrap_or_default();
        v.push(reward);
        crate::save(key.as_slice(), &v)?;
    }
    Ok(())
}

pub(crate) fn balance_of(
    community_id: CommunityId,
    account_id: AccountId,
) -> Result<String, String> {
    let community = crate::try_find_community(community_id)?;
    let key = trie::to_balance_key(community_id, account_id);
    storage::get(&key)
        .map_err(|e| e.to_string())?
        .map(|d| u128::decode(&mut &d[..]).map_err(|e| e.to_string()))
        .transpose()
        .map(|v| v.unwrap_or(0))
        .map(|v| {
            rust_decimal::Decimal::new(v as i64, community.token_info.decimals as u32).to_string()
        })
}

pub(crate) fn into_account_id(alias: &str) -> AccountId {
    AccountId::from_arbitrary(alias.as_bytes())
}

pub(crate) fn decompress(data: &[u8]) -> Result<String, String> {
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut s = String::new();
    decoder
        .read_to_string(&mut s)
        .map_err(|e| format!("Invalid compressed data: {:?}", e))?;
    Ok(s)
}

pub(crate) fn compress(data: &str) -> Result<Vec<u8>, String> {
    use std::io::Write;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(data.as_bytes())
        .map_err(|e| format!("Invalid data: {:?}", e))?;
    encoder
        .finish()
        .map_err(|e| format!("Invalid compressed data: {:?}", e))
}

pub(crate) fn validate_write_permission(
    community_id: CommunityId,
    account_id: AccountId,
) -> Result<(), String> {
    let key = trie::to_permission_key(community_id, account_id);
    let permission: u32 = find(key.as_ref())?.unwrap_or(0);
    (permission != 0)
        .then(|| ())
        .ok_or("You don't have permission to post in this community".to_string())
}
