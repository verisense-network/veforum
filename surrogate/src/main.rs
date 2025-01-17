use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use meilisearch_sdk::client::Client;
use parity_scale_codec::{Decode, Encode};
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use dotenv::dotenv;
use std::env;

use vemodel::{
    Method, VeArticle, VeComment, VeSubspace, PREFIX_ARTICLE_KEY, PREFIX_COMMENT_KEY,
    PREFIX_SUBSPACE_KEY,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let meilisarch_host = std::env::var("MEILISARCH_HOST").expect("no MEILISARCH_HOST env variable.");
    let meilisarch_secrect = std::env::var("MEILISARCH_SECRECT").expect("no MEILISARCH_SECRECT env variable.");
    let verisense_host = std::env::var("VERISENSE_HOST").expect("no MEILISARCH_HOST env variable.");
    let avs_id = std::env::var("AVS_ID").expect("no AVS_ID env variable.");

    let (tx, mut rx) = mpsc::channel(100);

    // Spawn a task for MeiliSearch indexing
    tokio::spawn(async move {
        let client = Client::new(
            meilisarch_host,
            Some(meilisarch_secrect),
        )
        .unwrap();
        // let index = client.index("users");

        while let Some((model, method, value)) = rx.recv().await {
            let index = match model {
                "subspace" => client.index("subspace"),
                "article" => client.index("article"),
                "comment" => client.index("comment"),
                _ => client.index("article"),
            };

            match method {
                Method::Create | Method::Update => {
                    match index.add_or_update(&[value], Some("id")).await {
                        Ok(task) => {
                            println!("Document added to MeiliSearch, task id: {}", task.task_uid);
                            // Optionally, wait for the task to complete
                            match task.wait_for_completion(&client, None, None).await {
                                Ok(task_info) => println!("Task completed: {:?}", task_info),
                                Err(e) => eprintln!("Error waiting for task completion: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Error adding document to MeiliSearch: {}", e),
                    }
                }
                Method::Delete => {
                    match index.delete_documents(&[value]).await {
                        Ok(task) => {
                            // Optionally, wait for the task to complete
                            match task.wait_for_completion(&client, None, None).await {
                                Ok(task_info) => println!("Task completed: {:?}", task_info),
                                Err(e) => eprintln!("Error waiting for task completion: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Error deleting document to MeiliSearch: {}", e),
                    }
                }
            }
        }
    });

    // Main task for RPC querying
    let http_client = HttpClientBuilder::default().build(verisense_host)?;

    // let avs_id = "5FsXfPrUDqq6abYccExCTUxyzjYaaYTr5utLx2wwdBv1m8R8";
    // let avs_id = AVS_ID;
    let mut sentinel: u64 = 0;
    loop {
        let avs_id = avs_id.clone();
        println!("==> sentinel: {}", sentinel);
        let params = rpc_params![
            avs_id.clone(),
            "get_from_common_key",
            hex::encode(sentinel.encode())
        ];

        let res: Result<serde_json::Value, _> = http_client.request("nucleus_post", params).await;
        println!("get from common key result: {:?}", res);
        if let Ok(res) = res {
            let res = res.as_str().expect("a str res");
            println!("str res: {}", res);
            let bytes = hex::decode(res).expect("Invalid hex string");
            // let res = <Option<User>>::decode(&mut &bytes[..]);
            let res = <Result<Vec<(u64, Method, Vec<u8>)>, String>>::decode(&mut &bytes[..]).unwrap();
            if res.is_err() {
                println!("res: {:?}", res);
            } else {
                for (reqnum, method, key) in res? {
                    match slice_to_array(&key[..5]).unwrap() {
                        PREFIX_SUBSPACE_KEY => {
                            let id = vec_to_u64(&key[5..]);
                            match method {
                                Method::Create | Method::Update => {
                                    let params =
                                        rpc_params![avs_id.clone(), "get_subspace", hex::encode(id.encode())];
                                    let res: serde_json::Value =
                                        http_client.request("nucleus_get", params).await?;
                                    let res = res.as_str().expect("a str res");
                                    println!("subspace str res: {}", res);
                                    let bytes = hex::decode(res).expect("Invalid hex string");
                                    let result =
                                        <Result<Option<VeSubspace>, String>>::decode(&mut &bytes[..])
                                            .unwrap();
                                    match result {
                                        Ok(Some(sb)) => {
                                            println!("subspace: {:?}", sb);
                                            // Serialize the user to a JSON Value
                                            let json_value = serde_json::to_value(&sb)?;

                                            // Send the JSON Value through the channel
                                            tx.send(("subspace", method, json_value)).await?;
                                        }
                                        Ok(None) => {
                                            println!("none");
                                        }
                                        Err(err) => {
                                            println!("{err}");
                                        }
                                    }
                                }
                                Method::Delete => {
                                    // Serialize the user to a JSON Value
                                    let json_value = serde_json::to_value(&id)?;

                                    // Send the JSON Value through the channel
                                    tx.send(("subspace", method, json_value)).await?;
                                }
                            }
                        }
                        PREFIX_ARTICLE_KEY => {
                            let id = vec_to_u64(&key[5..]);
                            match method {
                                Method::Create | Method::Update => {
                                    let params =
                                        rpc_params![avs_id.clone(), "get_article", hex::encode(id.encode())];
                                    let res: serde_json::Value =
                                        http_client.request("nucleus_get", params).await?;
                                    let res = res.as_str().expect("a str res");
                                    println!("article str res: {}", res);
                                    let bytes = hex::decode(res).expect("Invalid hex string");
                                    let result =
                                        <Result<Option<VeArticle>, String>>::decode(&mut &bytes[..])
                                            .unwrap();
                                    match result {
                                        Ok(Some(sb)) => {
                                            println!("article: {:?}", sb);
                                            // Serialize the user to a JSON Value
                                            let json_value = serde_json::to_value(&sb)?;

                                            // Send the JSON Value through the channel
                                            tx.send(("article", method, json_value)).await?;
                                        }
                                        Ok(None) => {
                                            println!("none");
                                        }
                                        Err(err) => {
                                            println!("{err}");
                                        }
                                    }
                                }
                                Method::Delete => {
                                    // Serialize the user to a JSON Value
                                    let json_value = serde_json::to_value(&id)?;

                                    // Send the JSON Value through the channel
                                    tx.send(("article", method, json_value)).await?;
                                }
                            }
                        }
                        PREFIX_COMMENT_KEY => {
                            let id = vec_to_u64(&key[5..]);
                            match method {
                                Method::Create | Method::Update => {
                                    let params =
                                        rpc_params![avs_id.clone(), "get_comment", hex::encode(id.encode())];
                                    let res: serde_json::Value =
                                        http_client.request("nucleus_get", params).await?;
                                    let res = res.as_str().expect("a str res");
                                    println!("comment str res: {}", res);
                                    let bytes = hex::decode(res).expect("Invalid hex string");
                                    let result =
                                        <Result<Option<VeComment>, String>>::decode(&mut &bytes[..])
                                            .unwrap();
                                    match result {
                                        Ok(Some(sb)) => {
                                            println!("comment: {:?}", sb);
                                            // Serialize the user to a JSON Value
                                            let json_value = serde_json::to_value(&sb)?;

                                            // Send the JSON Value through the channel
                                            tx.send(("comment", method, json_value)).await?;
                                        }
                                        Ok(None) => {
                                            println!("none");
                                        }
                                        Err(err) => {
                                            println!("{err}");
                                        }
                                    }
                                }
                                Method::Delete => {
                                    // Serialize the user to a JSON Value
                                    let json_value = serde_json::to_value(&id)?;

                                    // Send the JSON Value through the channel
                                    tx.send(("comment", method, json_value)).await?;
                                }
                            }
                        }
                        _ => {}
                    }
                    sentinel = reqnum;
                }
            }
        } else {
            println!("error from get_from_common_key");
        }

        sleep(Duration::from_secs(2)).await;
    }
}

fn vec_to_u64(v: &[u8]) -> u64 {
    let mut array = [0u8; 8];
    let len = std::cmp::min(v.len(), 8);
    array[..len].copy_from_slice(&v[..len]);
    u64::from_be_bytes(array)
}

fn slice_to_array(slice: &[u8]) -> Result<&[u8; 5], &str> {
    slice.try_into().map_err(|_| "Slice must be 5 bytes long")
}
