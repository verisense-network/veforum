use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub use vrs_core_sdk::AccountId;

pub type CommunityId = u32;
pub type EventId = u64;
pub type ContentId = u128;

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub enum Event {
    #[codec(index = 0)]
    CommunityCreated(CommunityId),
    #[codec(index = 1)]
    CommunityUpdated(CommunityId),
    #[codec(index = 2)]
    ThreadPosted(ContentId),
    #[codec(index = 3)]
    ThreadDeleted(ContentId),
    #[codec(index = 4)]
    CommentPosted(ContentId),
    #[codec(index = 5)]
    CommentDeleted(ContentId),
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize, Eq, PartialEq)]
pub enum CommunityStatus {
    PendingCreation = 0,
    WaitingTx = 1,
    Active = 2,
    Frozen = 3,
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct Community {
    pub id: CommunityId,
    pub name: String,
    pub slug: String,
    pub description: Vec<u8>,
    pub creator: AccountId,
    pub ed25519_pubkey: [u8; 32],
    pub status: CommunityStatus,
    pub created_time: i64,
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct Thread {
    pub id: ContentId,
    pub title: String,
    pub content: Vec<u8>,
    pub author: AccountId,
    pub mention: Vec<AccountId>,
    pub created_time: i64,
}

impl Thread {
    pub fn community_id(&self) -> CommunityId {
        (self.id >> 64) as CommunityId
    }
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct Comment {
    pub id: ContentId,
    pub content: Vec<u8>,
    pub author: AccountId,
    pub mention: Vec<AccountId>,
    pub reply_to: Option<ContentId>,
    pub created_time: i64,
}

impl Comment {
    pub fn community_id(&self) -> CommunityId {
        (self.id >> 64) as CommunityId
    }
}

pub mod trie {
    use super::*;
    pub const COMMUNITIES_KEY: u64 = 0x00000001_00000000;
    pub const CONTENTS_KEY: u128 = 0x00000002_00000000_00000000_00000000;
    pub const EVENTS_KEY: u128 = 0x00;

    /// check if the content id is a thread
    pub fn is_thread(content_id: ContentId) -> bool {
        content_id & 0xffffffff == 0
    }

    pub fn to_community_key(community_id: CommunityId) -> Vec<u8> {
        let key = COMMUNITIES_KEY | community_id as u64;
        key.to_be_bytes().to_vec()
    }

    pub fn to_community_id(key: &[u8]) -> Result<CommunityId, String> {
        let key = key
            .try_into()
            .map_err(|_| "invalid community id".to_string())?;
        let id = u64::from_be_bytes(key);
        Ok((id & u32::MAX as u64) as CommunityId)
    }

    pub fn to_content_key(content_id: ContentId) -> Vec<u8> {
        let key = CONTENTS_KEY | content_id;
        key.to_be_bytes().to_vec()
    }

    pub fn to_content_id(key: &[u8]) -> Result<ContentId, String> {
        let key = key
            .try_into()
            .map_err(|_| "invalid content id".to_string())?;
        let id = u128::from_be_bytes(key);
        Ok(id & 0x00000000_ffffffff_ffffffff_ffffffff)
    }

    pub fn to_event_key(event_id: EventId) -> Vec<u8> {
        let key = event_id as u128;
        key.to_be_bytes().to_vec()
    }

    pub fn to_event_id(key: &[u8]) -> Result<EventId, String> {
        let key = key.try_into().map_err(|_| "invalid event id".to_string())?;
        let id = u128::from_be_bytes(key);
        if id > u64::MAX as u128 {
            Err("invalid event id".to_string())
        } else {
            Ok((id & u64::MAX as u128) as EventId)
        }
    }
}

pub mod args {
    use super::*;
    use parity_scale_codec::{Decode, Encode};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct CreateCommunityArg {
        pub name: String,
        pub slug: String,
        pub description: Vec<u8>,
        pub prompt: String,
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct PostThreadArg {
        pub community: String,
        pub title: String,
        pub content: Vec<u8>,
        pub mention: Vec<AccountId>,
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct PostCommentArg {
        pub thread: ContentId,
        pub content: Vec<u8>,
        pub mention: Vec<AccountId>,
        pub reply_to: Option<ContentId>,
    }
}
