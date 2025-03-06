use meilisearch_sdk::{client::Client, settings::Settings};
use parity_scale_codec::Encode;
use reqwest;
use rocksdb::{Options, WriteBatchWithTransaction, DB};
use vemodel::*;

const EVENT_PREFIX: u128 = 0xffffffff_ffffffff_00000000_00000000;

pub fn open(path: impl AsRef<std::path::Path>) -> anyhow::Result<DB> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    DB::open(&opts, path).map_err(Into::into)
}

pub fn save_community(db: &DB, community: &Community) -> anyhow::Result<()> {
    db.put(community.id().to_be_bytes(), &community.encode())?;
    Ok(())
}

pub fn save_event(db: &DB, event_id: EventId, event: Event) -> anyhow::Result<()> {
    let key = EVENT_PREFIX | event_id as u128;
    db.put(key.to_be_bytes(), &event.encode())?;
    Ok(())
}

pub fn get_max_event(db: &DB) -> anyhow::Result<EventId> {
    db.iterator(rocksdb::IteratorMode::End)
        .next()
        .transpose()?
        .filter(|(k, _)| k.starts_with(&EVENT_PREFIX.to_be_bytes()[..=8]))
        .map(|(key, _)| {
            let id = u128::from_be_bytes((*key).try_into().expect("Invalid event id"));
            Ok(id as EventId)
        })
        .unwrap_or(Ok(0))
}

pub fn save_contents(db: &DB, contents: &[(ContentId, Vec<u8>)]) -> anyhow::Result<()> {
    let mut batch = WriteBatchWithTransaction::<false>::default();
    for (id, content) in contents {
        batch.put(id.to_be_bytes(), &content);
    }
    db.write(batch)?;
    Ok(())
}

pub fn del_content(db: &DB, id: ContentId) -> anyhow::Result<()> {
    db.delete(id.to_be_bytes())?;
    Ok(())
}

pub fn exists(db: &DB, id: impl AsRef<[u8]>) -> bool {
    db.key_may_exist(id)
}

async fn enable_experimental_features(
    meilisearch_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let meili_master_key = std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY must be set");
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "containsFilter": true
    });

    let response = client
        .patch(format!("{}/experimental-features", meilisearch_url))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", meili_master_key))
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Request succeeded: {:?}", response.text().await?);
    } else {
        println!("Request failed with status: {}", response.status());
    }

    Ok(())
}

pub async fn set_settings(client: &Client) {
    loop {
        match client.health().await {
            Ok(health) => {
                if health.status == "available" {
                    println!("Service is up and running!");
                    break;
                } else {
                    eprintln!("Received unexpected status: {:?}", health.status);
                }
            }
            Err(e) => {
                eprintln!("Error sending request: {}", e);
            }
        }
        // Wait a bit before retrying
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    let community = client.index("community");
    let community_settings = Settings::default()
        .with_filterable_attributes(["id", "creator"])
        .with_sortable_attributes(["created_time"]);
    community.set_settings(&community_settings).await.unwrap();

    let thread = client.index("thread");
    let thread_settings = Settings::default()
        .with_filterable_attributes(["id", "author"])
        .with_sortable_attributes(["created_time"]);
    thread.set_settings(&thread_settings).await.unwrap();

    let comment = client.index("comment");
    let comment_settings = Settings::default()
        .with_filterable_attributes(["id", "author"])
        .with_sortable_attributes(["created_time"]);
    comment.set_settings(&comment_settings).await.unwrap();

    enable_experimental_features(client.get_host())
        .await
        .unwrap();
}
