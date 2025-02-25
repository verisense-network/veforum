use crate::trie;
use parity_scale_codec::{Decode, Encode};
use vemodel::{args::*, *};
use vrs_core_sdk::{get, post, storage, timer, tss};

// TODO authorization
#[post]
pub fn set_llm_key(key: String) -> Result<(), String> {
    crate::agent::set_sys_key(crate::agent::OPENAI, key).map_err(|e| e.to_string())
}

#[post]
pub fn create_community(args: Args<CreateCommunityArg>) -> Result<CommunityId, String> {
    args.ensure_signed()?;
    let Args {
        signature: _signature,
        signer,
        nonce: _nonce,
        payload,
    } = args;
    payload.validate()?;
    let id = crate::name_to_community_id(&payload.name)
        .ok_or("Community name should only contains `a-zA-Z0-9_-` with length <= 24".to_string())?;
    let key = trie::to_community_key(id);
    let community = crate::find::<Community>(&key)?;
    community
        .is_none()
        .then(|| ())
        .ok_or("community already exists".to_string())?;
    let CreateCommunityArg {
        name,
        logo,
        token,
        slug,
        description,
        prompt,
        llm_name,
        llm_api_host,
        llm_key,
    } = payload;
    let token_info = TokenMetadata {
        symbol: token.symbol,
        total_issuance: token.total_issuance,
        decimals: token.decimals,
        contract: AccountId([0u8; 32]),
        image: token.image,
    };
    let key_id = id.to_be_bytes();
    let pubkey =
        tss::tss_get_public_key(tss::CryptoType::Ed25519, key_id).map_err(|e| e.to_string())?;
    let pubkey: [u8; 32] = pubkey.try_into().map_err(|_| "TSS key error".to_string())?;
    let llm_vendor = crate::from_llm_settings(llm_name, llm_api_host, llm_key)?;
    let community = Community {
        id: hex::encode(id.encode()),
        name: name.clone(),
        logo,
        slug,
        token_info,
        creator: signer,
        description,
        prompt: prompt.clone(),
        llm_vendor,
        llm_assistant_id: Default::default(),
        agent_pubkey: AccountId(pubkey),
        status: CommunityStatus::WaitingTx(500_000_000),
        created_time: timer::now() as i64,
    };
    crate::save(&key, &community)?;
    crate::save_event(Event::CommunityCreated(id))?;
    Ok(id)
}

#[post]
pub fn activate_community(arg: ActivateCommunityArg) -> Result<(), String> {
    let ActivateCommunityArg { community, tx } = arg;
    let id = crate::name_to_community_id(&community).ok_or("Invalid name".to_string())?;
    let key = trie::to_community_key(id);
    let community = crate::find::<Community>(&key)?.ok_or("Community not found".to_string())?;
    crate::agent::check_transfering(&community, tx)?;
    Ok(())
}

#[post]
pub fn post_thread(args: Args<PostThreadArg>) -> Result<ContentId, String> {
    args.ensure_signed()?;
    let Args {
        signature: _signature,
        signer,
        nonce: _nonce,
        payload,
    } = args;
    let PostThreadArg {
        community,
        title,
        content,
        image,
        mention,
    } = payload;
    let community_id =
        crate::name_to_community_id(&community).ok_or("Invalid community name".to_string())?;
    let key = trie::to_community_key(community_id);
    let community = crate::find::<Community>(&key)?.ok_or("Community not found".to_string())?;
    (community.status == CommunityStatus::Active)
        .then(|| ())
        .ok_or("Posting threads to this community is forbidden right now!".to_string())?;
    let id = crate::allocate_thread_id(community_id)?;
    let key = trie::to_content_key(id);
    let thread = Thread {
        id: hex::encode(id.encode()),
        community_name: community.name,
        title,
        content,
        image,
        author: signer,
        mention,
        llm_session_id: Default::default(),
        created_time: timer::now() as i64,
    };
    crate::save(&key, &thread)?;
    crate::save_event(Event::ThreadPosted(id))?;
    crate::agent::create_session_and_run(&thread)?;
    Ok(id)
}

#[post]
pub fn post_comment(args: Args<PostCommentArg>) -> Result<ContentId, String> {
    args.ensure_signed()?;
    let Args {
        signature: _signature,
        signer,
        nonce: _nonce,
        payload,
    } = args;
    let PostCommentArg {
        thread,
        content,
        image,
        mention,
        reply_to,
    } = payload;
    let community_id = (thread >> 64) as u32;
    let key = trie::to_community_key(community_id);
    let community = crate::find::<Community>(&key)?.ok_or("Community not found".to_string())?;
    (community.status == CommunityStatus::Active)
        .then(|| ())
        .ok_or("Posting comments to this community is forbidden right now!".to_string())?;
    let thread_key = trie::to_content_key(thread);
    crate::find::<Thread>(&thread_key)?.ok_or("Thread not found".to_string())?;
    let id = crate::allocate_comment_id(thread)?;
    let key = trie::to_content_key(id);
    let reply_to = reply_to
        .filter(|c| trie::is_comment(*c) && id > *c)
        .map(|c| hex::encode(c.encode()));
    let mention_agent = mention.contains(&community.agent_pubkey);
    let comment = Comment {
        id: hex::encode(id.encode()),
        content,
        image,
        author: signer,
        mention,
        reply_to,
        created_time: timer::now() as i64,
    };
    crate::save(&key, &comment)?;
    crate::save_event(Event::CommentPosted(id))?;
    if mention_agent {
        crate::agent::append_message_then_run(&comment)?;
    }
    Ok(id)
}

#[get]
pub fn get_community(id: CommunityId) -> Result<Option<Community>, String> {
    let key = trie::to_community_key(id);
    let mut community = crate::find::<Community>(&key)?;
    community.as_mut().map(|c| c.mask());
    Ok(community)
}

#[get]
pub fn get_raw_contents(id: ContentId, limit: u32) -> Result<Vec<(ContentId, Vec<u8>)>, String> {
    (limit <= 1000)
        .then(|| ())
        .ok_or("limit should be no more than 1000".to_string())?;
    if id > trie::MAX_CONTENT_ID {
        return Ok(vec![]);
    }
    let key = trie::to_content_key(id);
    let result = storage::get_range(key, storage::Direction::Forward, limit as usize)
        .map_err(|e| e.to_string())?;
    let mut r = vec![];
    for (k, v) in result.into_iter() {
        if let Ok(id) = trie::to_content_id(&k) {
            r.push((id, v));
        }
    }
    Ok(r)
}

#[get]
pub fn get_raw_content(id: ContentId) -> Result<Option<Vec<u8>>, String> {
    let key = trie::to_content_key(id);
    storage::get(&key).map_err(|e| e.to_string())
}

#[get]
pub fn get_events(id: EventId, limit: u32) -> Result<Vec<(EventId, Event)>, String> {
    (limit <= 1000)
        .then(|| ())
        .ok_or("limit should be no more than 1000".to_string())?;
    if id > trie::MAX_EVENT_ID {
        return Ok(vec![]);
    }
    let key = trie::to_event_key(id);
    let result = storage::get_range(key, storage::Direction::Forward, limit as usize)
        .map_err(|e| e.to_string())?;
    let mut r = vec![];
    for (k, v) in result.into_iter() {
        if let Ok(id) = trie::to_event_id(&k) {
            let event = Event::decode(&mut v.as_slice()).map_err(|e| e.to_string())?;
            r.push((id, event));
        }
    }
    Ok(r)
}

#[get]
pub fn get_account_info(account_id: AccountId) -> Result<Option<Account>, String> {
    let key = trie::to_account_key(account_id);
    match crate::find::<AccountData>(&key)? {
        Some(AccountData::Pubkey(data)) => Ok(Some(data)),
        Some(AccountData::AliasOf(id)) => {
            let key = trie::to_account_key(id);
            crate::find::<Account>(&key)
        }
        None => Ok(None),
    }
}

#[get]
pub fn get_balances(
    account_id: AccountId,
    gt: Option<CommunityId>,
    limit: u32,
) -> Result<Vec<(Community, u64)>, String> {
    (limit <= 100)
        .then(|| ())
        .ok_or("limit should be no more than 100".to_string())?;
    let key = trie::to_balance_key(gt.unwrap_or_default(), account_id);
    let result = storage::get_range(&key, storage::Direction::Forward, limit as usize + 1)
        .map_err(|e| e.to_string())?;
    let mut r = vec![];
    for (k, v) in result.into_iter() {
        if k.len() == 40 && k.starts_with(&key[..36]) {
            if let Ok((community, balance)) = compose_balance(k, v) {
                r.push((community, balance));
            }
        }
    }
    Ok(r)
}

fn compose_balance(key: Vec<u8>, value: Vec<u8>) -> Result<(Community, u64), String> {
    let suffix: [u8; 4] = *(&key[36..].try_into().expect("qed"));
    let community_id = CommunityId::from_be_bytes(suffix);
    let balance = u64::decode(&mut &value[..]).map_err(|e| e.to_string())?;
    let community_key = trie::to_community_key(community_id);
    let mut community =
        crate::find::<Community>(&community_key)?.ok_or("Community not found".to_string())?;
    community.prompt = Default::default();
    Ok((community, balance))
}
