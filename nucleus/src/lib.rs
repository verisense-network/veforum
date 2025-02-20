mod agent;
mod nucleus;
mod trie;

use sha2::{Digest, Sha256};
use vemodel::{AccountId, CommunityId, ContentId, Event, EventId};
use vrs_core_sdk::{
    codec::{Decode, Encode},
    storage,
};

pub(crate) fn find<T: Decode>(key: &[u8]) -> Result<Option<T>, String> {
    let r = storage::get(key).map_err(|e| e.to_string())?;
    r.map(|d| T::decode(&mut &d[..]))
        .transpose()
        .map_err(|e| e.to_string())
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
        .filter(|(k, _)| k.starts_with(&start_key.to_be_bytes()[..=8]))
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
        .filter(|(k, _)| k.starts_with(&start_key.to_be_bytes()[..=12]))
        .map(|(k, _)| trie::to_content_id(&k))
        .transpose()?
        .unwrap_or(thread_id);
    (r & 0xffffffff < 0xffffffff)
        .then(|| ())
        .ok_or("We don't expect more than 4b comments in a thread :)")?;
    Ok(r + 1)
}

pub(crate) fn transfer(
    community_id: CommunityId,
    from: AccountId,
    to: AccountId,
    amount: u64,
) -> Result<(), String> {
    let from_key = trie::to_balance_key(community_id, from);
    let from_balance = storage::get(&from_key)
        .map_err(|e| e.to_string())?
        .map(|d| u64::decode(&mut &d[..]).map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(0);
    let to_key = trie::to_balance_key(community_id, to);
    let to_balance = storage::get(&to_key)
        .map_err(|e| e.to_string())?
        .map(|d| u64::decode(&mut &d[..]).map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(0);
    if from_balance < amount {
        return Err("insufficient balance".to_string());
    }
    storage::put(&from_key, (from_balance - amount).encode()).map_err(|e| e.to_string())?;
    storage::put(&to_key, (to_balance + amount).encode()).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn balance_of(community_id: CommunityId, account_id: AccountId) -> Result<u64, String> {
    let key = trie::to_balance_key(community_id, account_id);
    storage::get(&key)
        .map_err(|e| e.to_string())?
        .map(|d| u64::decode(&mut &d[..]).map_err(|e| e.to_string()))
        .transpose()
        .map(|v| v.unwrap_or(0))
}
