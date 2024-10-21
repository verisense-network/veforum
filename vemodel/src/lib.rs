use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub enum Method {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct VeSubspace {
    pub id: u64,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub banner: String,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct VeArticle {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author_id: u64,
    pub author_nickname: String,
    pub subspace_id: u64,
    pub ext_link: String,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
    pub updated_time: i64,
}

#[derive(Debug, Decode, Encode, Deserialize, Serialize)]
pub struct VeComment {
    pub id: u64,
    pub content: String,
    pub author_id: u64,
    pub author_nickname: String,
    pub post_id: u64,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
}

// const PREFIX_USER_KEY: &[u8; 5] = b"veus:";
pub const PREFIX_SUBSPACE_KEY: &[u8; 5] = b"vesb:";
pub const PREFIX_ARTICLE_KEY: &[u8; 5] = b"vear:";
pub const PREFIX_COMMENT_KEY: &[u8; 5] = b"veco:";

pub const REQNUM_KEY: &[u8; 7] = b"_reqnum";
pub const COMMON_KEY: &[u8; 7] = b"_common";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
