use crate::{rpc, storage};
use jsonrpsee::http_client::HttpClient;
use meilisearch_sdk::client::Client;
use parity_scale_codec::Decode;
use rocksdb::DB;
use vemodel::*;
use vrs_core_sdk::NucleusId;

pub async fn index_event(
    origin: &HttpClient,
    db: &DB,
    indexer: &Client,
    nucleus_id: &NucleusId,
    id: EventId,
    event: Event,
) -> anyhow::Result<()> {
    match event {
        Event::CommunityCreated(community_id) | Event::CommunityUpdated(community_id) => {
            if !storage::exists(&db, community_id.to_be_bytes()) {
                let community = rpc::get_community(origin, nucleus_id, community_id)
                    .await
                    .inspect_err(|e| println!("Error: {:?}", e))
                    .map_err(|_| anyhow::anyhow!("fetch community failed"))?
                    .ok_or_else(|| anyhow::anyhow!("community not found"))?;
                storage::save_community(&db, &community)?;
                let index = indexer.index("community");
                println!(
                    "indexing community: {}",
                    serde_json::to_string(&community).unwrap()
                );
                let task = index.add_documents(&[community], Some("id")).await?;
                let info = task.wait_for_completion(&indexer, None, None).await?;
                println!("{:?}", info);
            }
            storage::save_event(&db, id, Event::CommunityCreated(community_id))?;
        }
        Event::ThreadPosted(content_id) => {
            if !storage::exists(&db, content_id.to_be_bytes()) {
                let contents = rpc::get_contents(origin, nucleus_id, content_id)
                    .await
                    .map_err(|_| anyhow::anyhow!("fetch contents failed"))?;
                println!("{} contents fetched", contents.len());
                let mut comments = vec![];
                let mut threads = vec![];
                for (id, content) in contents.iter() {
                    if vemodel::is_comment(*id) {
                        let comment = Comment::decode(&mut &content[..])
                            .map_err(|_| anyhow::anyhow!("decode comment failed"))?;
                        if !storage::exists(&db, id.to_be_bytes()) {
                            comments.push(comment);
                        }
                    } else {
                        let thread = Thread::decode(&mut &content[..])
                            .map_err(|_| anyhow::anyhow!("decode thread failed"))?;
                        if !storage::exists(&db, id.to_be_bytes()) {
                            threads.push(thread);
                        }
                    }
                }
                storage::save_contents(&db, &contents)?;
                let task = indexer
                    .index("thread")
                    .add_documents(&threads, Some("id"))
                    .await?;
                let info = task.wait_for_completion(&indexer, None, None).await?;
                println!("{:?}", info);
                indexer
                    .index("comment")
                    .add_documents(&threads, Some("id"))
                    .await?;
            }
            storage::save_event(&db, id, Event::ThreadPosted(content_id))?;
        }
        Event::ThreadDeleted(content_id) => {
            storage::del_content(&db, content_id)?;
            let index = indexer.index("thread");
            index.delete_document(content_id).await?;
            storage::save_event(&db, id, Event::ThreadDeleted(content_id))?;
        }
        Event::CommentPosted(content_id) => {
            if !storage::exists(&db, content_id.to_be_bytes()) {
                let content = rpc::get_content(origin, nucleus_id, content_id)
                    .await
                    .map_err(|_| anyhow::anyhow!("fetch comments failed"))?;
                if let Some(raw) = content {
                    storage::save_contents(&db, &[(content_id, raw.clone())])?;
                    let comment = Comment::decode(&mut &raw[..])
                        .map_err(|_| anyhow::anyhow!("decode comment failed"))?;
                    let index = indexer.index("comment");
                    index.add_documents(&[comment], Some("id")).await?;
                }
            }
            storage::save_event(&db, id, Event::CommentPosted(content_id))?;
        }
        Event::CommentDeleted(content_id) => {
            storage::del_content(&db, content_id)?;
            let index = indexer.index("comment");
            index.delete_document(content_id).await?;
            storage::save_event(&db, id, Event::CommentDeleted(content_id))?;
        }
    }
    Ok(())
}
