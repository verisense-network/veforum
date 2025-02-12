mod indexer;
mod rpc;
mod storage;

use meilisearch_sdk::client::*;
use std::str::FromStr;
use vemodel::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = storage::open("./data")?;
    let origin = rpc::build_client("http://localhost:9944");
    let nucleus_id = AccountId::from_str("kGk1FJCoPv4JTxez4aaWgGVaTPvsc2YPStz6ZWni4e61FVUW6")?;
    let indexer = Client::new("http://localhost:7700", Some("masterkey"))?;

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
