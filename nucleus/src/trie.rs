use vemodel::*;

#[allow(dead_code)]
pub const MAX_COMMUNITY_ID: u32 = 0xffffffff;
pub const MAX_EVENT_ID: u64 = 0xffffffff_ffffffff;
pub const MAX_CONTENT_ID: u128 = 0x00000000_ffffffff_ffffffff_ffffffff;

pub const MAX_EVENT_KEY: u128 = 0x00000000_00000000_ffffffff_ffffffff;
pub const MIN_COMMUNITIE_KEY: u64 = 0x00000001_00000000;
#[allow(dead_code)]
pub const MAX_COMMUNITY_KEY: u64 = 0x00000001_ffffffff;
pub const MIN_CONTENT_KEY: u128 = 0x00000002_00000000_00000000_00000000;
pub const MAX_CONTENT_KEY: u128 = 0x00000002_ffffffff_ffffffff_ffffffff;

pub const ACCOUNT_KEY_PREFIX: u64 = 0x00000003_00000000;
pub const BALANCE_KEY_PREFIX: u64 = 0x00000004_00000000;
pub const PERMISSION_KEY_PREFIX: u64 = 0x00000005_00000000;
pub const REWARD_PAYLOAD_PREFIX: u64 = 0x00000006_00000000;
pub const REWARD_SEQ_PREFIX: u64  = 0x00000007_00000000;
pub const INVITE_AMT_PREFIX: u64  = 0x00000008_00000000;

pub const KEY_STORE: u64 = 0x00000010_00000000;
pub const HTTP_MASK: u128 = 0x0000000f_00000000_00000000_00000000;


pub fn is_comment(content_id: ContentId) -> bool {
    content_id & 0xffffffff != 0
}

pub fn to_community_key(community_id: CommunityId) -> [u8; 8] {
    let key = MIN_COMMUNITIE_KEY | community_id as u64;
    key.to_be_bytes()
}


pub fn to_content_key(content_id: ContentId) -> [u8; 16] {
    let key = MIN_CONTENT_KEY | content_id;
    key.to_be_bytes()
}

pub fn to_content_id(key: &[u8]) -> Result<ContentId, String> {
    let key = key
        .try_into()
        .map_err(|_| "invalid content id".to_string())?;
    let key = u128::from_be_bytes(key);
    if key <= MAX_CONTENT_KEY && key >= MIN_CONTENT_KEY {
        Ok(key & MAX_CONTENT_ID)
    } else {
        Err("invalid content id".to_string())
    }
}

pub fn to_event_key(event_id: EventId) -> [u8; 16] {
    let key = event_id as u128;
    key.to_be_bytes()
}

pub fn to_event_id(key: &[u8]) -> Result<EventId, String> {
    let key = key.try_into().map_err(|_| "invalid event id".to_string())?;
    let key = u128::from_be_bytes(key);
    if key <= MAX_EVENT_KEY {
        Ok(key as EventId)
    } else {
        Err("invalid event id".to_string())
    }
}

pub fn http_trace_key(id: u64) -> [u8; 16] {
    (HTTP_MASK | id as u128).to_be_bytes()
}

pub fn llm_key(vendor: [u8; 4]) -> [u8; 8] {
    (u32::from_be_bytes(vendor) as u64 | KEY_STORE).to_be_bytes()
}

pub fn to_account_key(account_id: AccountId) -> [u8; 28] {
    [&ACCOUNT_KEY_PREFIX.to_be_bytes()[..], &account_id.0[..]]
        .concat()
        .try_into()
        .unwrap()
}

pub fn to_balance_key(community_id: CommunityId, account_id: AccountId) -> [u8; 32] {
    [
        &BALANCE_KEY_PREFIX.to_be_bytes()[..],
        &account_id.0[..],
        &community_id.to_be_bytes()[..],
    ]
    .concat()
    .try_into()
    .unwrap()
}

pub fn to_invitecode_amt_key(community_id: CommunityId, account_id: AccountId) -> [u8; 32] {
    [
        &INVITE_AMT_PREFIX.to_be_bytes()[..],
        &account_id.0[..],
        &community_id.to_be_bytes()[..],
    ]
        .concat()
        .try_into()
        .unwrap()
}

pub fn to_reward_payload_key(community_id: CommunityId, account_id: AccountId) -> [u8;32]{
    [
        &REWARD_PAYLOAD_PREFIX.to_be_bytes()[..],
        &account_id.0[..],
        &community_id.to_be_bytes()[..],
    ]
        .concat()
        .try_into()
        .unwrap()
}

pub fn to_reward_seq_key(community_id: CommunityId) -> [u8;12]{
    [
        &REWARD_SEQ_PREFIX.to_be_bytes()[..],
        &community_id.to_be_bytes()[..],
    ]
        .concat()
        .try_into()
        .unwrap()
}



pub fn to_permission_key(community_id: CommunityId, account_id: AccountId) -> [u8; 32] {
    [
        &PERMISSION_KEY_PREFIX.to_be_bytes()[..],
        &account_id.0[..],
        &community_id.to_be_bytes()[..],
    ]
    .concat()
    .try_into()
    .unwrap()
}
