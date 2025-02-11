use crate::{rpc, storage};
use meilisearch_sdk::client::Client;
use parity_scale_codec::Decode;
use rocksdb::DB;
use vemodel::*;

pub async fn index_event(
    origin: &str,
    db: &DB,
    indexer: &Client,
    nucleus_id: &AccountId,
    id: EventId,
    event: Event,
) -> anyhow::Result<()> {
    let client = rpc::build_client(origin);
    match event {
        Event::CommunityCreated(community_id) | Event::CommunityUpdated(community_id) => {
            if !storage::exists(&db, id.to_be_bytes()) {
                let community = rpc::get_community(&client, nucleus_id, community_id)
                    .await
                    .map_err(|_| anyhow::anyhow!("fetch community failed"))?
                    .ok_or_else(|| anyhow::anyhow!("community not found"))?;
                storage::save_community(&db, &community)?;
                storage::save_event(&db, id, Event::CommunityCreated(community_id))?;
                let index = indexer.index("community");
                index.add_documents(&[community], Some("id")).await?;
            }
        }
        Event::ThreadPosted(content_id) => {
            if !storage::exists(&db, id.to_be_bytes()) {
                let contents = rpc::get_contents(&client, nucleus_id, content_id)
                    .await
                    .map_err(|_| anyhow::anyhow!("fetch contents failed"))?;
                storage::save_contents(&db, &contents)?;
                storage::save_event(&db, id, Event::ThreadPosted(content_id))?;
                let thread_index = indexer.index("thread");
                let comment_index = indexer.index("comment");
                for (id, content) in contents {
                    if id & 0xffffffff == 0 {
                        let thread = Thread::decode(&mut &content[..])
                            .map_err(|_| anyhow::anyhow!("decode thread failed"))?;
                        thread_index.add_documents(&[thread], Some("id")).await?;
                    } else {
                        let comment = Comment::decode(&mut &content[..])
                            .map_err(|_| anyhow::anyhow!("decode comment failed"))?;
                        comment_index.add_documents(&[comment], Some("id")).await?;
                    }
                }
            }
        }
        Event::ThreadDeleted(content_id) => {
            storage::del_content(&db, content_id)?;
            storage::save_event(&db, id, Event::ThreadDeleted(content_id))?;
            let index = indexer.index("thread");
            index.delete_document(content_id).await?;
        }
        Event::CommentPosted(content_id) => {
            if !storage::exists(&db, id.to_be_bytes()) {
                let content = rpc::get_content(&client, nucleus_id, content_id)
                    .await
                    .map_err(|_| anyhow::anyhow!("fetch comments failed"))?;
                if let Some(raw) = content {
                    storage::save_contents(&db, &[(content_id, raw.clone())])?;
                    storage::save_event(&db, id, Event::CommentPosted(content_id))?;
                    let comment = Comment::decode(&mut &raw[..])
                        .map_err(|_| anyhow::anyhow!("decode comment failed"))?;
                    let index = indexer.index("comment");
                    index.add_documents(&[comment], Some("id")).await?;
                }
            }
        }
        Event::CommentDeleted(content_id) => {
            storage::del_content(&db, content_id)?;
            storage::save_event(&db, id, Event::CommentDeleted(content_id))?;
            let index = indexer.index("comment");
            index.delete_document(content_id).await?;
        }
    }
    Ok(())
}
