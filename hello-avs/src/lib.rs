use parity_scale_codec::{Decode, Encode};
use vrs_core_sdk::{get, post, storage};

#[derive(Debug, Decode, Encode)]
pub struct User {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Decode, Encode)]
pub enum Method {
    Create,
    Update,
    Delete,
}

// #[derive(Debug, Clone, Default, Encode, Decode)]
// pub struct VeUser {
//     pub id: u64,
//     pub account: String,
//     pub nickname: String,
//     pub avatar: String,
//     pub role: i16,
//     pub status: i16,
//     pub created_time: i64,
// }

#[derive(Debug, Clone, Default, Encode, Decode)]
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

#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct VeArticle {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author_id: u64,
    pub author_nickname: String,
    pub subspace_id: u64,
    pub extlink: String,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
    pub updated_time: i64,
}

#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct VeComment {
    pub id: u64,
    pub content: String,
    pub author_id: u64,
    pub author_nickname: u64,
    pub post_id: u64,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
}

// const PREFIX_USER_KEY: &[u8; 5] = b"veus:";
const PREFIX_SUBSPACE_KEY: &[u8; 5] = b"vesb:";
const PREFIX_ARTICLE_KEY: &[u8; 5] = b"vear:";
const PREFIX_COMMENT_KEY: &[u8; 5] = b"veco:";

const REQNUM_KEY: &[u8; 7] = b"_reqnum";
const COMMON_KEY: &[u8; 7] = b"_common";

// #[post]
// pub fn add_user(user: User) -> Result<(), String> {
//     let max_id_key = [&b"user:"[..], &u64::MAX.to_be_bytes()[..]].concat();
//     let max_id = match storage::search(&max_id_key, storage::Direction::Reverse)
//         .map_err(|e| e.to_string())?
//     {
//         Some((id, _)) => u64::from_be_bytes(id[5..].try_into().unwrap()) + 1,
//         None => 1u64,
//     };
//     let key = [&b"user:"[..], &max_id.to_be_bytes()[..]].concat();
//     storage::put(&key, user.encode()).map_err(|e| e.to_string())
// }

// #[get]
// pub fn get_user(id: u64) -> Result<Option<User>, String> {
//     let key = [&b"user:"[..], &id.to_be_bytes()[..]].concat();
//     let r = storage::get(&key).map_err(|e| e.to_string())?;
//     let user = r.map(|d| User::decode(&mut &d[..]).unwrap());
//     Ok(user)
// }

// subspace
#[post]
pub fn add_subspace(mut sb: VeSubspace) -> Result<(), String> {
    let max_id = get_max_id(PREFIX_SUBSPACE_KEY);
    // update the id field from the avs
    sb.id = max_id;
    let key = build_key(PREFIX_SUBSPACE_KEY, max_id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;

    add_to_common_key(Method::Create, key)?;

    Ok(())
}

#[post]
pub fn update_subspace(sb: VeSubspace) -> Result<(), String> {
    let id = sb.id;
    let key = build_key(PREFIX_SUBSPACE_KEY, id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;

    add_to_common_key(Method::Update, key)?;

    Ok(())
}

#[post]
pub fn delete_subspace(id: u64) -> Result<(), String> {
    let key = build_key(PREFIX_SUBSPACE_KEY, id);
    storage::del(&key).map_err(|e| e.to_string())?;

    add_to_common_key(Method::Delete, key)?;

    Ok(())
}

#[get]
pub fn get_subspace(id: u64) -> Result<Option<VeSubspace>, String> {
    let key = build_key(PREFIX_SUBSPACE_KEY, id);
    let r = storage::get(&key).map_err(|e| e.to_string())?;
    let instance = r.map(|d| VeSubspace::decode(&mut &d[..]).unwrap());
    Ok(instance)
}

// article
#[post]
pub fn add_article(mut sb: VeArticle) -> Result<(), String> {
    let max_id = get_max_id(PREFIX_ARTICLE_KEY);
    // update the id field from the avs
    sb.id = max_id;
    let key = build_key(PREFIX_ARTICLE_KEY, max_id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Create, key)?;

    Ok(())
}

#[post]
pub fn update_article(sb: VeArticle) -> Result<(), String> {
    let id = sb.id;
    let key = build_key(PREFIX_ARTICLE_KEY, id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Update, key)?;

    Ok(())
}

#[post]
pub fn delete_article(id: u64) -> Result<(), String> {
    let key = build_key(PREFIX_ARTICLE_KEY, id);
    storage::del(&key).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Delete, key)?;

    Ok(())
}

#[get]
pub fn get_article(id: u64) -> Result<Option<VeArticle>, String> {
    let key = build_key(PREFIX_ARTICLE_KEY, id);
    let r = storage::get(&key).map_err(|e| e.to_string())?;
    let instance = r.map(|d| VeArticle::decode(&mut &d[..]).unwrap());
    Ok(instance)
}

// comment
#[post]
pub fn add_comment(mut sb: VeComment) -> Result<(), String> {
    let max_id = get_max_id(PREFIX_COMMENT_KEY);
    // update the id field from the avs
    sb.id = max_id;
    let key = build_key(PREFIX_COMMENT_KEY, max_id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Create, key)?;

    Ok(())
}

#[post]
pub fn update_comment(sb: VeComment) -> Result<(), String> {
    let id = sb.id;
    let key = build_key(PREFIX_COMMENT_KEY, id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Update, key)?;

    Ok(())
}

#[post]
pub fn delete_comment(id: u64) -> Result<(), String> {
    let key = build_key(PREFIX_COMMENT_KEY, id);
    storage::del(&key).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Delete, key)?;

    Ok(())
}

#[get]
pub fn get_comment(id: u64) -> Result<Option<VeComment>, String> {
    let key = build_key(PREFIX_COMMENT_KEY, id);
    let r = storage::get(&key).map_err(|e| e.to_string())?;
    let instance = r.map(|d| VeComment::decode(&mut &d[..]).unwrap());
    Ok(instance)
}

//
//
fn add_to_common_key(method: Method, model_ins: Vec<u8>) -> Result<(), String> {
    let reqnum = get_reqnum();

    let res = storage::get(COMMON_KEY).map_err(|e| e.to_string())?;
    if let Some(res) = res {
        let mut avec = Vec::<(u64, Method, Vec<u8>)>::decode(&mut &res[..]).unwrap();
        // insert new tuple item
        avec.push((reqnum, method, model_ins));
        // write back
        _ = storage::put(COMMON_KEY, avec.encode()).map_err(|e| e.to_string());
    } else {
        let avec = vec![(reqnum, method, model_ins)];
        _ = storage::put(COMMON_KEY, avec.encode()).map_err(|e| e.to_string());
    }

    Ok(())
}

#[post]
pub fn get_from_common_key(sentinel: u64) -> Result<Vec<(u64, Method, Vec<u8>)>, String> {
    let res = storage::get(COMMON_KEY).map_err(|e| e.to_string())?;
    if let Some(res) = res {
        let mut avec = Vec::<(u64, Method, Vec<u8>)>::decode(&mut &res[..]).unwrap();
        let mut index = 0;
        for (i, &(reqnum, _, _)) in avec.iter().enumerate() {
            if reqnum > sentinel {
                index = i;
                break;
            }
        }

        let last_part = avec.split_off(index);
        _ = storage::put(COMMON_KEY, last_part.encode()).map_err(|e| e.to_string());
        return Ok(last_part);
    }

    Ok(vec![])
}

fn get_max_id(prefix: &[u8; 5]) -> u64 {
    let max_id_key = [prefix, &u64::MAX.to_be_bytes()[..]].concat();
    let max_id = match storage::search(&max_id_key, storage::Direction::Reverse)
        .map_err(|e| e.to_string())
        .expect("error in storage search.")
    {
        Some((id, _)) => u64::from_be_bytes(id[5..].try_into().unwrap()) + 1,
        None => 1u64,
    };

    max_id
}

fn build_key(prefix: &[u8; 5], id: u64) -> Vec<u8> {
    [prefix, &id.to_be_bytes()[..]].concat()
}

fn get_reqnum() -> u64 {
    let res = storage::get(REQNUM_KEY)
        .map_err(|e| e.to_string())
        .expect("error in storage get");
    let reqnum = if let Some(res) = res {
        let reqnum: u64 = u64::from_be_bytes(TryInto::<[u8; 8]>::try_into(res).unwrap());
        println!("==> current reqnum: {:?}", reqnum);

        // increase reqnum on every request of reqnum
        let new_reqnum = reqnum + 1;
        _ = storage::put(REQNUM_KEY, new_reqnum.encode()).map_err(|e| e.to_string());

        reqnum
    } else {
        // initialize it on start
        let reqnum = 1;
        _ = storage::put(REQNUM_KEY, reqnum.encode()).map_err(|e| e.to_string());

        reqnum
    };

    reqnum
}
