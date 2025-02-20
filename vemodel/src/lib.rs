use borsh::{BorshDeserialize, BorshSerialize};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type CommunityId = u32;
pub type EventId = u64;
pub type ContentId = u128;

pub fn is_comment(content_id: ContentId) -> bool {
    content_id & 0xffffffff != 0
}

pub fn is_thread(content_id: ContentId) -> bool {
    content_id & 0xffffffff == 0
}

pub fn get_belongs_to(content_id: ContentId) -> CommunityId {
    (content_id >> 64) as CommunityId
}

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
    pub id: String,
    pub logo: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub token_info: TokenMetadata,
    pub prompt: String,
    pub creator: AccountId,
    pub agent_pubkey: AccountId,
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
        AccountId(v)
    }
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct Thread {
    pub id: String,
    pub community_name: String,
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
    pub reply_to: Option<String>,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Decode, Encode, BorshSerialize, BorshDeserialize)]
pub struct AccountId(pub [u8; 32]);

pub type Pubkey = AccountId;

const MAX_BASE58_LEN: usize = 44;

impl std::str::FromStr for AccountId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        (s.len() <= MAX_BASE58_LEN)
            .then(|| ())
            .ok_or("invalid account id".to_string())?;
        bs58::decode(s.as_bytes())
            .into_array_const::<32>()
            .map(|a| Self(a))
            .map_err(|_| "invalid account id".to_string())
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        bs58::encode(&self.encode()).into_string().fmt(f)
    }
}

struct AccountIdVisitor;

impl<'de> serde::de::Visitor<'de> for AccountIdVisitor {
    type Value = Pubkey;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "bs58 account")
    }

    fn visit_str<E>(self, value: &str) -> Result<AccountId, E>
    where
        E: serde::de::Error,
    {
        <AccountId as std::str::FromStr>::from_str(value)
            .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(value), &self))
    }
}

impl<'de> serde::Deserialize<'de> for AccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(AccountIdVisitor)
    }
}

impl serde::Serialize for AccountId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Decode, Encode, Deserialize, Serialize)]
pub struct Account {
    pub nonce: u64,
    pub pubkey: Pubkey,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Decode, Encode, Deserialize, Serialize)]
pub enum AccountData {
    Pubkey(Account),
    AliasOf(AccountId),
}

impl Account {
    pub fn name(&self) -> String {
        self.alias
            .clone()
            .unwrap_or_else(|| self.pubkey.to_string())
    }
}

#[derive(Debug, Clone, Decode, Encode, Deserialize, Serialize)]
pub struct TokenMetadata {
    pub symbol: String,
    pub total_issuance: u64,
    pub decimals: u8,
    pub contract: AccountId,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Copy, Decode, Encode)]
pub struct Signature(pub [u8; 64]);

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        hex::encode(&self.encode()).fmt(f)
    }
}

pub mod args {
    use super::*;
    use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
    use parity_scale_codec::{Decode, Encode};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Decode, Encode)]
    pub struct Args<T> {
        pub signature: Signature,
        pub signer: Pubkey,
        pub nonce: u64,
        pub payload: T,
    }

    pub trait Verifiable<T: Encode> {
        fn ensure_signed(&self) -> Result<(), String>;

        fn prehash(&self) -> [u8; 32];
    }

    impl<T: Encode> Verifiable<T> for Args<T> {
        fn ensure_signed(&self) -> Result<(), String> {
            // let prehash = self.prehash();
            // let pubkey = VerifyingKey::from_bytes(&self.signer.0).map_err(|_| "invalid pubkey")?;
            // let signature = Ed25519Signature::from_bytes(&self.signature.0);
            // pubkey
            //     .verify(&prehash, &signature)
            //     .map_err(|_| "invalid signature")?;
            Ok(())
        }

        fn prehash(&self) -> [u8; 32] {
            let mut hasher = Sha256::new();
            hasher.update(self.nonce.encode().as_slice());
            hasher.update(self.payload.encode().as_slice());
            hasher.finalize().into()
        }
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct CreateCommunityArg {
        pub name: String,
        pub logo: String,
        pub token: TokenMetadataArg,
        pub slug: String,
        pub description: String,
        pub prompt: String,
    }

    const COMMUNITY_REGEX: &'static str = r"^[a-zA-Z0-9_-]{3,24}$";
    const TOKEN_REGEX: &'static str = r"^[a-zA-Z0-9]{3,8}$";

    #[derive(Debug, Clone, Decode, Encode, Deserialize, Serialize)]
    pub struct TokenMetadataArg {
        pub symbol: String,
        pub total_issuance: u64,
        pub decimals: u8,
        pub image: Option<String>,
    }

    impl TokenMetadataArg {
        pub fn validate(&self) -> Result<(), String> {
            let re = regex::Regex::new(TOKEN_REGEX).unwrap();
            re.captures(&self.symbol)
                .is_some()
                .then(|| ())
                .ok_or("Invalid token name".to_string())?;
            (self.total_issuance > 0 && self.total_issuance <= (1u64 << 53))
                .then(|| ())
                .ok_or("total issuance should be greater than 0".to_string())?;
            (self.decimals <= 8)
                .then(|| ())
                .ok_or("decimals should be less than or equal to 18".to_string())?;
            Ok(())
        }
    }

    impl CreateCommunityArg {
        pub fn validate(&self) -> Result<(), String> {
            let re = regex::Regex::new(COMMUNITY_REGEX).unwrap();
            re.captures(&self.name)
                .is_some()
                .then(|| ())
                .ok_or("Invalid community name".to_string())?;
            self.token.validate()
        }
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

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct SetAliasArg {
        pub alias: String,
    }
}
