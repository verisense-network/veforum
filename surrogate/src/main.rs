mod indexer;
mod rpc;
mod storage;

use meilisearch_sdk::client::*;
use std::str::FromStr;
use vemodel::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = storage::open("./data")?;
    let client = rpc::build_client("http://localhost:9944");
    let nucleus_id = AccountId::from_str("kGk1FJCoPv4JTxez4aaWgGVaTPvsc2YPStz6ZWni4e61FVUW6")?;
    let indexer = Client::new("", Some(""))?;

    loop {
        let event_id = storage::get_max_event(&db)? + 1;
        let events = rpc::get_events(&client, &nucleus_id, event_id)
            .await
            .map_err(|_| anyhow::anyhow!("fetch events failed"))?;
        if events.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            continue;
        }
        for (id, event) in events.iter() {
            if let Err(e) = indexer::index_event(
                "http://localhost:9944",
                &db,
                &indexer,
                &nucleus_id,
                *id,
                *event,
            )
            .await
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
