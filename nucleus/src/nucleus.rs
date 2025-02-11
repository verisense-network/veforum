use parity_scale_codec::{Decode, Encode};
use vemodel::{args::*, trie, *};
use vrs_core_sdk::{get, post, storage, timer, AccountId};

// TODO authorization
#[post]
pub fn set_llm_key(key: String) -> Result<(), String> {
    crate::agent::set_llm_key(key).map_err(|e| e.to_string())
}

#[post]
pub fn create_community(
    creator: AccountId,
    arg: CreateCommunityArg,
) -> Result<CommunityId, String> {
    let id = crate::community_id(&arg.name)
        .ok_or("Community name should only contains `a-zA-Z0-9_-` with length <= 24".to_string())?;
    let key = trie::to_community_key(id);
    let community = crate::find::<Community>(&key)?;
    community
        .is_none()
        .then(|| ())
        .ok_or("community already exists".to_string())?;
    let CreateCommunityArg {
        name,
        slug,
        description,
        prompt,
    } = arg;
    let community = Community {
        id,
        name: name.clone(),
        slug,
        creator,
        description,
        ed25519_pubkey: [0u8; 32],
        // TODO: WaitingTx
        status: CommunityStatus::Active,
        created_time: timer::now() as i64,
    };
    storage::put(&key, community.encode()).map_err(|e| e.to_string())?;
    crate::save_event(Event::CommunityCreated(id))?;
    crate::agent::init_agent(&name, prompt)?;
    Ok(id)
}

#[post]
pub fn activate_community(community: String, tx: [u8; 32]) -> Result<(), String> {
    let community_id = crate::community_id(&community).ok_or("Invalid name".to_string())?;
    let key = trie::to_community_key(community_id);
    let mut community = crate::find::<Community>(&key)?.ok_or("Community not found".to_string())?;
    // TODO check tx_hash
    community.status = CommunityStatus::Active;
    storage::put(&key, community.encode()).map_err(|e| e.to_string())?;
    crate::save_event(Event::CommunityUpdated(community_id))?;
    Ok(())
}

#[post]
pub fn post_thread(author: AccountId, arg: PostThreadArg) -> Result<ContentId, String> {
    let PostThreadArg {
        community,
        title,
        content,
        mention,
    } = arg;
    let community_id =
        crate::community_id(&community).ok_or("Invalid community name".to_string())?;
    let key = trie::to_community_key(community_id);
    let community = crate::find::<Community>(&key)?.ok_or("Community not found".to_string())?;
    (community.status == CommunityStatus::Active)
        .then(|| ())
        .ok_or("Posting threads to this community is forbidden right now!".to_string())?;
    let id = crate::allocate_thread_id(community.id)?;
    let key = trie::to_content_key(id);
    let thread = Thread {
        id,
        title,
        content,
        author,
        mention,
        created_time: timer::now() as i64,
    };
    storage::put(&key, thread.encode()).map_err(|e| e.to_string())?;
    crate::save_event(Event::ThreadPosted(id))?;
    Ok(id)
}

#[post]
pub fn post_comment(author: AccountId, arg: PostCommentArg) -> Result<ContentId, String> {
    let PostCommentArg {
        thread,
        content,
        mention,
        reply_to,
    } = arg;
    let community_id = (thread >> 64) as u32;
    let key = trie::to_community_key(community_id);
    let community = crate::find::<Community>(&key)?.ok_or("Community not found".to_string())?;
    (community.status == CommunityStatus::Active)
        .then(|| ())
        .ok_or("Posting comments to this community is forbidden right now!".to_string())?;
    let thread_id = trie::to_content_key(thread);
    let _thread = crate::find::<Thread>(&thread_id)?.ok_or("Thread not found".to_string())?;
    let id = crate::allocate_comment_id(thread)?;
    let key = trie::to_content_key(id);
    let comment = Comment {
        id,
        content,
        author,
        mention,
        reply_to,
        created_time: timer::now() as i64,
    };
    storage::put(&key, comment.encode()).map_err(|e| e.to_string())?;
    crate::save_event(Event::CommentPosted(id))?;
    Ok(id)
}

#[get]
pub fn get_community(id: CommunityId) -> Result<Option<Community>, String> {
    let key = trie::to_community_key(id);
    crate::find(&key)
}

#[get]
pub fn get_raw_contents(id: ContentId, limit: u32) -> Result<Vec<(ContentId, Vec<u8>)>, String> {
    (limit <= 1000)
        .then(|| ())
        .ok_or("limit should be no more than 1000".to_string())?;
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
    let max_id = crate::allocate_event_id()?;
    if id > max_id {
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
