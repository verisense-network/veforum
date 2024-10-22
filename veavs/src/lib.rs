use parity_scale_codec::{Decode, Encode};
use sp_core::crypto::{Pair, Ss58Codec};
use sp_core::sr25519::{Public, Signature};
use std::isize;
use vrs_core_sdk::{get, post, storage};

use vemodel::{
    Method, VeArticle, VeComment, VeSubspace, COMMON_KEY, PREFIX_ARTICLE_KEY, PREFIX_COMMENT_KEY,
    PREFIX_SUBSPACE_KEY, REQNUM_KEY,
};

// subspace
#[post]
pub fn add_subspace(
    mut sb: VeSubspace,
    account: String,
    msg: String,
    sig: String,
) -> Result<(), String> {
    if !validate(&account, &msg, &sig)? {
        return Err("signature validation error".to_string());
    };

    let max_id = get_max_id(PREFIX_SUBSPACE_KEY);
    // update the id field from the avs
    sb.id = max_id;
    let key = build_key(PREFIX_SUBSPACE_KEY, max_id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;

    add_to_common_key(Method::Create, key)?;

    Ok(())
}

#[post]
pub fn update_subspace(
    sb: VeSubspace,
    account: String,
    msg: String,
    sig: String,
) -> Result<(), String> {
    if !validate(&account, &msg, &sig)? {
        return Err("signature validation error".to_string());
    };

    let id = sb.id;
    let key = build_key(PREFIX_SUBSPACE_KEY, id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;

    add_to_common_key(Method::Update, key)?;

    Ok(())
}

#[post]
pub fn delete_subspace(id: u64, account: String, msg: String, sig: String) -> Result<(), String> {
    if !validate(&account, &msg, &sig)? {
        return Err("signature validation error".to_string());
    };

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
pub fn add_article(
    mut sb: VeArticle,
    account: String,
    msg: String,
    sig: String,
) -> Result<(), String> {
    if !validate(&account, &msg, &sig)? {
        return Err("signature validation error".to_string());
    };

    let max_id = get_max_id(PREFIX_ARTICLE_KEY);
    // update the id field from the avs
    sb.id = max_id;
    let key = build_key(PREFIX_ARTICLE_KEY, max_id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Create, key)?;

    Ok(())
}

#[post]
pub fn update_article(
    sb: VeArticle,
    account: String,
    msg: String,
    sig: String,
) -> Result<(), String> {
    if !validate(&account, &msg, &sig)? {
        return Err("signature validation error".to_string());
    };

    let id = sb.id;
    let key = build_key(PREFIX_ARTICLE_KEY, id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Update, key)?;

    Ok(())
}

#[post]
pub fn delete_article(id: u64, account: String, msg: String, sig: String) -> Result<(), String> {
    if !validate(&account, &msg, &sig)? {
        return Err("signature validation error".to_string());
    };

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
pub fn add_comment(
    mut sb: VeComment,
    account: String,
    msg: String,
    sig: String,
) -> Result<(), String> {
    if !validate(&account, &msg, &sig)? {
        return Err("signature validation error".to_string());
    };

    let max_id = get_max_id(PREFIX_COMMENT_KEY);
    // update the id field from the avs
    sb.id = max_id;
    let key = build_key(PREFIX_COMMENT_KEY, max_id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Create, key)?;

    Ok(())
}

#[post]
pub fn update_comment(
    sb: VeComment,
    account: String,
    msg: String,
    sig: String,
) -> Result<(), String> {
    if !validate(&account, &msg, &sig)? {
        return Err("signature validation error".to_string());
    };

    let id = sb.id;
    let key = build_key(PREFIX_COMMENT_KEY, id);
    storage::put(&key, sb.encode()).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Update, key)?;

    Ok(())
}

#[post]
pub fn delete_comment(id: u64, account: String, msg: String, sig: String) -> Result<(), String> {
    if !validate(&account, &msg, &sig)? {
        return Err("signature validation error".to_string());
    };

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

#[get]
pub fn check_all_range() -> Result<(), String> {
    check_range(PREFIX_SUBSPACE_KEY);
    check_range(PREFIX_ARTICLE_KEY);
    check_range(PREFIX_COMMENT_KEY);
    Ok(())
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
        let mut index: isize = -1;
        for (i, &(reqnum, _, _)) in avec.iter().enumerate() {
            if reqnum <= sentinel {
                index = i as isize;
            } else {
                break;
            }
        }
        let last_part = avec.split_off((index + 1) as usize);

        _ = storage::put(COMMON_KEY, last_part.encode()).map_err(|e| e.to_string());
        return Ok(last_part);
    }

    Ok(vec![])
}

fn get_max_id(prefix: &[u8; 5]) -> u64 {
    let max_id_key = [prefix, &u64::MAX.to_be_bytes()[..]].concat();
    let max_id = match storage::search(&max_id_key, storage::Direction::Reverse)
        .map_err(|e| e.to_string())
    {
        Ok(Some((id, _))) => {
            println!("==-->> max_id_key: {:?}", id);
            if let Ok(id) = id[5..].try_into() {
                u64::from_be_bytes(id) + 1
            } else {
                1u64
            }
        }
        Ok(None) => 1u64,
        Err(_) => 1u64,
    };
    println!("==-->> the next max id is: {}", max_id);

    max_id
}

fn check_range(prefix: &[u8; 5]) {
    match storage::get_range(&prefix, storage::Direction::Forward, 100).map_err(|e| e.to_string()) {
        Ok(vec) => {
            println!("{:?}", vec)
        }
        Err(e) => {
            println!("{:?}", e)
        }
    };
}

fn build_key(prefix: &[u8; 5], id: u64) -> Vec<u8> {
    [prefix, &id.to_be_bytes()[..]].concat()
}

fn get_reqnum() -> u64 {
    let res = storage::get(REQNUM_KEY)
        .map_err(|e| e.to_string())
        .expect("error in storage get");
    let reqnum = if let Some(res) = res {
        // XXX: notice that the SCALE use the little endian format
        let reqnum: u64 = u64::from_le_bytes(TryInto::<[u8; 8]>::try_into(res).unwrap());
        println!("==> current reqnum: {:?}", reqnum);

        // increase reqnum on every request of reqnum
        let reqnum = reqnum + 1;
        _ = storage::put(REQNUM_KEY, reqnum.encode()).map_err(|e| e.to_string());

        reqnum
    } else {
        // initialize it on start
        let reqnum = 1;
        _ = storage::put(REQNUM_KEY, reqnum.encode()).map_err(|e| e.to_string());

        reqnum
    };

    reqnum
}

fn get_publickey_from_address(address: &str) -> Result<Public, String> {
    Public::from_ss58check(address).map_err(|_| "check ss58 address error".to_string())
}

fn check_signature(sig: &str) -> Result<Signature, String> {
    let signature_bytes = hex::decode(sig).map_err(|e| e.to_string())?;
    let signature = sp_core::sr25519::Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| "error while parsing signature from string".to_string())?;
    Ok(signature)
}

fn verify(sig: &Signature, message: &[u8], pubkey: &Public) -> bool {
    // Verify the signature
    sp_core::sr25519::Pair::verify(&sig, message, &pubkey)
}

fn validate(address: &str, sigstr: &str, msg: &str) -> Result<bool, String> {
    let public_key = get_publickey_from_address(address)?;
    let sig = check_signature(sigstr)?;
    Ok(verify(&sig, msg.as_bytes(), &public_key))
}
