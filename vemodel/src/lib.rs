use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use vrs_core_sdk::AccountId;

pub type CommunityId = u32;
pub type EventId = u64;
pub type ContentId = u128;

#[derive(Debug, Decode, Encode, Deserialize, Serialize, Clone, Copy)]
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
    pub name: String,
    pub slug: String,
    pub description: String,
    pub creator: AccountId,
    pub ed25519_pubkey: [u8; 32],
    pub status: CommunityStatus,
    pub created_time: i64,
}

impl Community {
    pub fn id(&self) -> CommunityId {
        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());
        let v = hasher.finalize();
        CommunityId::from_be_bytes(v[..4].try_into().unwrap())
    }

    pub fn agent_account(&self) -> AccountId {
        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());
        let v: [u8; 32] = hasher.finalize().into();
        AccountId::from(v)
    }
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct Thread {
    pub id: String,
    pub title: String,
    pub content: String,
    pub image: Option<String>,
    pub author: AccountId,
    pub mention: Vec<AccountId>,
    pub created_time: i64,
}

impl Thread {
    pub fn id(&self) -> ContentId {
        let id = hex::decode(&self.id).expect("invalid thread id");
        ContentId::decode(&mut &id[..]).expect("invalid thread id")
    }

    pub fn community_id(&self) -> CommunityId {
        (self.id() >> 64) as CommunityId
    }
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct Comment {
    pub id: String,
    pub content: String,
    pub image: Option<String>,
    pub author: AccountId,
    pub mention: Vec<AccountId>,
    pub reply_to: Option<ContentId>,
    pub created_time: i64,
}

impl Comment {
    pub fn id(&self) -> ContentId {
        let id = hex::decode(&self.id).expect("invalid thread id");
        ContentId::decode(&mut &id[..]).expect("invalid thread id")
    }

    pub fn community_id(&self) -> CommunityId {
        (self.id() >> 64) as CommunityId
    }
}

pub mod trie {
    use super::*;
    pub const MIN_COMMUNITIE_KEY: u64 = 0x00000001_00000000;
    pub const MAX_COMMUNITY_KEY: u64 = 0x00000001_ffffffff;
    pub const MAX_COMMUNITY_ID: u32 = 0xffffffff;

    pub const MIN_CONTENT_KEY: u128 = 0x00000002_00000000_00000000_00000000;
    pub const MAX_CONTENT_KEY: u128 = 0x00000002_ffffffff_ffffffff_ffffffff;
    pub const MAX_CONTENT_ID: u128 = 0x00000000_ffffffff_ffffffff_ffffffff;

    pub const MAX_EVENT_KEY: u128 = 0xffffffff_ffffffff;
    pub const MAX_EVENT_ID: u64 = 0xffffffff_ffffffff;

    /// check if the content id is a thread
    pub fn is_thread(content_id: ContentId) -> bool {
        content_id & 0xffffffff == 0
    }

    pub fn to_community_key(community_id: CommunityId) -> Vec<u8> {
        let key = MIN_COMMUNITIE_KEY | community_id as u64;
        key.to_be_bytes().to_vec()
    }

    pub fn to_community_id(key: &[u8]) -> Result<CommunityId, String> {
        let key = key
            .try_into()
            .map_err(|_| "invalid community id".to_string())?;
        let id = u64::from_be_bytes(key);
        Ok(id as CommunityId)
    }

    pub fn to_content_key(content_id: ContentId) -> Vec<u8> {
        let key = MIN_CONTENT_KEY | content_id;
        key.to_be_bytes().to_vec()
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

    pub fn to_event_key(event_id: EventId) -> Vec<u8> {
        let key = event_id as u128;
        key.to_be_bytes().to_vec()
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
}

pub mod args {
    use super::*;
    use parity_scale_codec::{Decode, Encode};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct CreateCommunityArg {
        pub name: String,
        pub slug: String,
        pub description: String,
        pub prompt: String,
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct PostThreadArg {
        pub community: String,
        pub title: String,
        pub content: String,
        pub image: Option<String>,
        pub mention: Vec<AccountId>,
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct PostCommentArg {
        pub thread: ContentId,
        pub content: String,
        pub image: Option<String>,
        pub mention: Vec<AccountId>,
        pub reply_to: Option<ContentId>,
    }
}
