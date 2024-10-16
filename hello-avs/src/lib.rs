use parity_scale_codec::{Decode, Encode};
use vrs_core_sdk::{get, post, storage};

#[derive(Debug, Decode, Encode)]
pub struct User {
    pub id: u64,
    pub name: String,
}

#[post]
pub fn add_user(user: User) -> Result<(), String> {
    let max_id_key = [&b"user:"[..], &u64::MAX.to_be_bytes()[..]].concat();
    let max_id = match storage::search(&max_id_key, storage::Direction::Reverse)
        .map_err(|e| e.to_string())?
    {
        Some((id, _)) => u64::from_be_bytes(id[5..].try_into().unwrap()) + 1,
        None => 1u64,
    };
    let key = [&b"user:"[..], &max_id.to_be_bytes()[..]].concat();
    storage::put(&key, user.encode()).map_err(|e| e.to_string())
}

#[get]
pub fn get_user(id: u64) -> Result<Option<User>, String> {
    let key = [&b"user:"[..], &id.to_be_bytes()[..]].concat();
    let r = storage::get(&key).map_err(|e| e.to_string())?;
    let user = r.map(|d| User::decode(&mut &d[..]).unwrap());
    Ok(user)
}

fn add_to_common_key(method: String, model_ins: String) -> Result<(), String> {
    let reqnum_key = b"_reqnum";
    let res = storage::get(&reqnum_key).map_err(|e| e.to_string())?;
    if let Some(res) = res {
        let reqnum: u64 = u64::from_be_bytes(TryInto::<[u8; 8]>::try_into(res).unwrap());
        println!("==> current reqnum: {:?}", reqnum);

        let key = b"_commonkey";
        let res = storage::get(&key).map_err(|e| e.to_string())?;
        if let Some(res) = res {
            let mut avec = Vec::<(u64, String, String)>::decode(&mut &res[..]).unwrap();
            avec.push((reqnum, method, model_ins));
            _ = storage::put(&key, avec.encode()).map_err(|e| e.to_string());
        } else {
            let value = vec![(reqnum, method, model_ins)];
            _ = storage::put(&key, value.encode()).map_err(|e| e.to_string());
        }

        let reqnum = reqnum + 1;
        let reqnum_key = b"_reqnum";
        _ = storage::put(&reqnum_key, reqnum.encode()).map_err(|e| e.to_string());
    } else {
        let reqnum = 1;
        let reqnum_key = b"_reqnum";
        _ = storage::put(&reqnum_key, reqnum.encode()).map_err(|e| e.to_string());

        let key = b"_commonkey";
        let value = vec![(reqnum, method, model_ins)];
        _ = storage::put(&key, value.encode()).map_err(|e| e.to_string());
    }

    Ok(())
}

fn get_from_common_key(sentinel: u64) -> Result<(), String> {
    let key = b"_commonkey";
    let res = storage::get(&key).map_err(|e| e.to_string())?;
    if let Some(res) = res {
        let mut avec = Vec::<(u64, String, String)>::decode(&mut &res[..]).unwrap();
        let mut index = 0;
        for (i, &(reqnum, _, _)) in avec.iter().enumerate() {
            if reqnum > sentinel {
                index = i;
                break;
            }
        }

        let last_part = avec.split_off(index);
        let key = b"_commonkey";
        _ = storage::put(&key, last_part.encode()).map_err(|e| e.to_string());
    }

    Ok(())
}
