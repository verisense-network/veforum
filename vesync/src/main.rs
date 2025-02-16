mod indexer;
mod rpc;
mod storage;

use meilisearch_sdk::client::*;
use std::str::FromStr;
use vrs_core_sdk::NucleusId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = storage::open("./data")?;
    let meili_master_key = std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY must be set");
    let meili_addr = std::env::var("MEILI_ADDR").expect("MEILI_ADDR must be set");
    let verisense_rpc = std::env::var("VERISENSE_RPC").expect("VERISENSE_RPC must be set");
    let nucleus_id = std::env::var("NUCLEUS_ID").expect("NUCLEUS_ID must be set");
    let origin = rpc::build_client(&verisense_rpc);
    let nucleus_id = NucleusId::from_str(&nucleus_id)?;
    let indexer = Client::new(meili_addr, Some(meili_master_key))?;

    loop {
        let event_id = storage::get_max_event(&db)? + 1;
        println!("fetching from event id: {}", event_id);
        let events = rpc::get_events(&origin, &nucleus_id, event_id).await;
        println!(
            "{} events fetched",
            events.as_ref().map(|e| e.len()).unwrap_or(0)
        );
        if events.is_err() {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            continue;
        }
        let events = events.unwrap();
        if events.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            continue;
        }
        for (id, event) in events.iter() {
            if let Err(e) =
                indexer::index_event(&origin, &db, &indexer, &nucleus_id, *id, *event).await
            {
                eprintln!("index event failed: {:?}", e);
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                break;
            }
        }
        if events.len() < 1000 {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}
