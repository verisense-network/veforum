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
    PendingCreation,
    WaitingTx(u128),
    CreateFailed(String),
    Active,
    Frozen(u64),
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct Community {
    pub id: String,
    pub private: bool,
    pub logo: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub token_info: TokenMetadata,
    pub prompt: String,
    pub creator: AccountId,
    pub agent_pubkey: AccountId,
    pub llm_vendor: LlmVendor,
    pub llm_assistant_id: String,
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

    pub fn mask(&mut self) {
        self.prompt = Default::default();
        match &self.llm_vendor {
            LlmVendor::OpenAI { .. } => {
                self.llm_vendor = LlmVendor::OpenAI {
                    key: Default::default(),
                };
            }
            LlmVendor::DeepSeek { key: _key, host } => {
                self.llm_vendor = LlmVendor::DeepSeek {
                    key: Default::default(),
                    host: host.clone(),
                };
            }
        }
    }
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct Thread {
    pub id: String,
    pub community_name: String,
    pub title: String,
    pub content: Vec<u8>,
    pub images: Vec<String>,
    pub author: AccountId,
    pub mention: Vec<AccountId>,
    pub llm_session_id: String,
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
    pub content: Vec<u8>,
    pub images: Vec<String>,
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

    pub fn thread_id(&self) -> ContentId {
        let id = self.id();
        id & (u128::MAX - u32::MAX as u128)
    }

    pub fn community_id(&self) -> CommunityId {
        (self.id() >> 64) as CommunityId
    }
}

pub type AccountId = H160;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Decode, Encode)]
pub struct H160(pub [u8; 20]);

impl H160 {
    pub fn from_slice(s: &[u8]) -> Result<Self, String> {
        (s.len() == 20)
            .then(|| ())
            .ok_or("invalid account id".to_string())?;
        let mut res = [0u8; 20];
        res.copy_from_slice(&s[..20]);
        Ok(Self(res))
    }

    #[cfg(feature = "crypto")]
    pub fn from_compressed(raw: &[u8; 33]) -> Result<Self, String> {
        use tiny_keccak::{Hasher, Keccak};
        let pubkey =
            secp256k1::PublicKey::from_byte_array_compressed(raw).map_err(|e| e.to_string())?;
        let mut hasher = Keccak::v256();
        let mut digest = [0u8; 32];
        hasher.update(&pubkey.serialize_uncompressed()[1..]);
        hasher.finalize(&mut digest);
        let mut res = [0u8; 20];
        res.copy_from_slice(&digest[12..]);
        Ok(Self(res))
    }

    #[cfg(feature = "crypto")]
    pub fn from_arbitrary(v: &[u8]) -> Self {
        use tiny_keccak::{Hasher, Keccak};
        let mut hasher = Keccak::v256();
        let mut digest = [0u8; 32];
        hasher.update(v);
        hasher.finalize(&mut digest);
        let mut res = [0u8; 20];
        res.copy_from_slice(&digest[12..]);
        Self(res)
    }
}

impl std::str::FromStr for H160 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Option<[u8; 20]> = hex::decode(s.to_lowercase().trim_start_matches("0x"))
            .map(|v| v.try_into().ok())
            .map_err(|e| e.to_string())?;
        s.map(|v| Self(v)).ok_or("invalid account id".to_string())
    }
}

impl std::fmt::Display for H160 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        format!("0x{}", hex::encode(self.0)).fmt(f)
    }
}

struct H160Visitor;

impl<'de> serde::de::Visitor<'de> for H160Visitor {
    type Value = H160;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "h160 with 0x prefix")
    }

    fn visit_str<E>(self, value: &str) -> Result<AccountId, E>
    where
        E: serde::de::Error,
    {
        <H160 as std::str::FromStr>::from_str(value)
            .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(value), &self))
    }
}

impl<'de> serde::Deserialize<'de> for H160 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(H160Visitor)
    }
}

impl serde::Serialize for H160 {
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
    pub address: H160,
    pub alias: Option<String>,
    pub last_post_at: i64,
}

#[derive(Debug, Clone, Decode, Encode, Deserialize, Serialize)]
pub enum AccountData {
    Pubkey(Account),
    AliasOf(AccountId),
}

const POST_COOLING_DOWN: i64 = 180;

impl Account {
    pub fn new(address: H160) -> Self {
        Self {
            nonce: 0,
            address,
            alias: None,
            last_post_at: 0,
        }
    }

    pub fn name(&self) -> String {
        self.alias
            .clone()
            .unwrap_or_else(|| self.address.to_string())
    }

    pub fn allow_post(&self, now: i64) -> bool {
        self.last_post_at + POST_COOLING_DOWN < now
    }
}

#[derive(Debug, Clone, Decode, Encode, Deserialize, Serialize)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub total_issuance: u64,
    pub decimals: u8,
    pub contract: AccountId,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub enum LlmVendor {
    OpenAI { key: String },
    DeepSeek { key: String, host: String },
}

impl LlmVendor {
    pub fn key<'a>(&'a self) -> &'a str {
        match self {
            Self::OpenAI { key } => key,
            Self::DeepSeek { key, .. } => key,
        }
    }
}

#[cfg(feature = "crypto")]
pub mod crypto {
    use parity_scale_codec::{Decode, Encode};
    use secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        Message, Secp256k1,
    };
    use tiny_keccak::{Hasher, Keccak};

    /// SECP256k1 ECDSA signature in RSV format, V should be either `0/1` or `27/28`.
    #[derive(Debug, Clone, Copy, Decode, Encode)]
    pub struct EcdsaSignature(pub [u8; 65]);

    impl EcdsaSignature {
        pub fn recover(&self, msg: [u8; 32]) -> Result<[u8; 64], String> {
            let secp = Secp256k1::verification_only();
            let rid = if self.0[64] == 27u8 || self.0[64] == 0u8 {
                RecoveryId::Zero
            } else if self.0[64] == 28u8 || self.0[64] == 1u8 {
                RecoveryId::One
            } else {
                return Err("Bad V in signature".to_string());
            };
            let signature = RecoverableSignature::from_compact(&self.0[..64], rid)
                .map_err(|_| "Bad RS in signature".to_string())?;
            let msg = Message::from_digest(msg);
            let pubkey = secp
                .recover_ecdsa(&msg, &signature)
                .map_err(|_| "Bad signature".to_string())?;
            let mut res = [0u8; 64];
            res.copy_from_slice(&pubkey.serialize_uncompressed()[1..]);
            Ok(res)
        }
    }

    impl std::fmt::Display for EcdsaSignature {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            hex::encode(&self.encode()).fmt(f)
        }
    }

    pub trait EcdsaVerifiable<T: Encode> {
        fn ensure_signed(&self, nonce: u64) -> Result<(), String>;

        fn to_be_signed(&self) -> Vec<u8>;
    }

    impl<T: Encode> EcdsaVerifiable<T> for crate::args::Args<T, EcdsaSignature> {
        fn ensure_signed(&self, nonce: u64) -> Result<(), String> {
            (self.nonce == nonce)
                .then(|| ())
                .ok_or("invalid nonce".to_string())?;

            let message = self.to_be_signed();

            let mut keccak = Keccak::v256();
            keccak.update(&message);
            let mut message_hash = [0u8; 32];
            keccak.finalize(&mut message_hash);

            let raw_pubkey = self.signature.recover(message_hash)?;

            let pubkey_to_hash = if raw_pubkey.len() == 65 && raw_pubkey[0] == 0x04 {
                &raw_pubkey[1..]
            } else {
                &raw_pubkey
            };

            let mut keccak = Keccak::v256();
            keccak.update(pubkey_to_hash);
            let mut pubkey_hash = [0u8; 32];
            keccak.finalize(&mut pubkey_hash);

            (pubkey_hash[12..] == self.signer.0)
                .then(|| ())
                .ok_or("Invalid signature".to_string())?;
            Ok(())
        }

        fn to_be_signed(&self) -> Vec<u8> {
            let nonce_encoded = self.nonce.encode();
            let payload_encoded = self.payload.encode();

            let mut message_buf = Vec::with_capacity(nonce_encoded.len() + payload_encoded.len());
            message_buf.extend_from_slice(&nonce_encoded);
            message_buf.extend_from_slice(&payload_encoded);

            let hex_message = hex::encode(&message_buf);
            let prefixed_message = format!(
                "\x19Ethereum Signed Message:\n{}{}",
                hex_message.len(),
                hex_message
            );
            let prefixed_message_bytes = prefixed_message.as_bytes().to_vec();

            prefixed_message_bytes
        }
    }
}

pub mod args {
    use super::*;
    use parity_scale_codec::{Decode, Encode};
    use serde::{Deserialize, Serialize};

    const COMMUNITY_REGEX: &'static str = r"^[a-zA-Z0-9_-]{3,24}$";
    const TOKEN_REGEX: &'static str = r"^[a-zA-Z0-9]{3,8}$";
    const NAME_REGEX: &'static str = r"^[\p{L}\p{N}_-]{3,30}$";

    #[derive(Debug, Clone, Decode, Encode)]
    pub struct Args<T, S> {
        pub signature: S,
        pub signer: AccountId,
        pub nonce: u64,
        pub payload: T,
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct CreateCommunityArg {
        pub name: String,
        pub private: bool,
        pub logo: String,
        pub token: TokenMetadataArg,
        pub slug: String,
        pub description: String,
        pub prompt: String,
        pub llm_name: String,
        pub llm_api_host: Option<String>,
        pub llm_key: Option<String>,
    }

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
                .ok_or("Invalid community name".to_string())?;
            self.token.validate()
        }
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct ActivateCommunityArg {
        pub community: String,
        pub tx: String,
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct PostThreadArg {
        pub community: String,
        pub title: String,
        pub content: Vec<u8>,
        pub images: Vec<String>,
        pub mention: Vec<AccountId>,
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct PostCommentArg {
        pub thread: ContentId,
        pub content: Vec<u8>,
        pub images: Vec<String>,
        pub mention: Vec<AccountId>,
        pub reply_to: Option<ContentId>,
    }

    #[derive(Debug, Decode, Encode, Deserialize, Serialize)]
    pub struct SetAliasArg {
        pub alias: String,
    }

    impl SetAliasArg {
        pub fn validate(&self) -> Result<(), String> {
            let re = regex::Regex::new(NAME_REGEX).unwrap();
            re.captures(&self.alias)
                .ok_or("Invalid alias".to_string())?;
            Ok(())
        }
    }
}
