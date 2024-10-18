use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use meilisearch_sdk::client::Client;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
// use serde_json::Value;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Encode, Decode, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    // Add other fields as needed
}

#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub enum Method {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct VeSubspace {
    pub id: u64,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub banner: String,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
}

#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct VeArticle {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author_id: u64,
    pub author_nickname: String,
    pub subspace_id: u64,
    pub extlink: String,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
    pub updated_time: i64,
}

#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct VeComment {
    pub id: u64,
    pub content: String,
    pub author_id: u64,
    pub author_nickname: u64,
    pub post_id: u64,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
}

const PREFIX_SUBSPACE_KEY: &[u8; 5] = b"vesb:";
const PREFIX_ARTICLE_KEY: &[u8; 5] = b"vear:";
const PREFIX_COMMENT_KEY: &[u8; 5] = b"veco:";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn a task for MeiliSearch indexing
    tokio::spawn(async move {
        let client = Client::new("http://localhost:7700", Some("masterKey")).unwrap();
        let index = client.index("users");

        while let Some((method, value)) = rx.recv().await {
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
    let http_client = HttpClientBuilder::default().build("http://localhost:9944")?;

    let avs_id = "5FsXfPrUDqq6abYccExCTUxyzjYaaYTr5utLx2wwdBv1m8R8";
    let mut sentinel = 0;
    loop {
        let params = rpc_params![avs_id, "get_from_common_key", sentinel.encode()];
        let result: Vec<(u64, Method, Vec<u8>)> =
            http_client.request("nucleus_get", params).await?;

        for (reqnum, method, key) in result {
            match slice_to_array(&key[..5]).unwrap() {
                PREFIX_SUBSPACE_KEY => {
                    let id = vec_to_u64(&key[5..]);
                    match method {
                        Method::Create | Method::Update => {
                            let params = rpc_params![avs_id, "get_subspace", id.encode()];
                            let result: Result<Option<VeSubspace>, String> =
                                http_client.request("nucleus_get", params).await?;
                            match result {
                                Ok(Some(sb)) => {
                                    // Serialize the user to a JSON Value
                                    let json_value = serde_json::to_value(&sb)?;

                                    // Send the JSON Value through the channel
                                    tx.send((method, json_value)).await?;
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
                            tx.send((method, json_value)).await?;
                        }
                    }
                }
                PREFIX_ARTICLE_KEY => {
                    let id = vec_to_u64(&key[5..]);
                    match method {
                        Method::Create | Method::Update => {
                            let params = rpc_params![avs_id, "get_article", id.encode()];
                            let result: Result<Option<VeArticle>, String> =
                                http_client.request("nucleus_get", params).await?;
                            match result {
                                Ok(Some(sb)) => {
                                    // Serialize the user to a JSON Value
                                    let json_value = serde_json::to_value(&sb)?;

                                    // Send the JSON Value through the channel
                                    tx.send((method, json_value)).await?;
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
                            tx.send((method, json_value)).await?;
                        }
                    }
                }
                PREFIX_COMMENT_KEY => {
                    let id = vec_to_u64(&key[5..]);
                    match method {
                        Method::Create | Method::Update => {
                            let params = rpc_params![avs_id, "get_comment", id.encode()];
                            let result: Result<Option<VeComment>, String> =
                                http_client.request("nucleus_get", params).await?;
                            match result {
                                Ok(Some(sb)) => {
                                    // Serialize the user to a JSON Value
                                    let json_value = serde_json::to_value(&sb)?;

                                    // Send the JSON Value through the channel
                                    tx.send((method, json_value)).await?;
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
                            tx.send((method, json_value)).await?;
                        }
                    }
                }
                _ => {}
            }
            sentinel = reqnum;
        }

        // // Decode the SCALE-encoded result
        // let user = User::decode(&mut &result[..])?;

        // // Serialize the user to a JSON Value
        // let json_value = serde_json::to_value(&user)?;

        // // Send the JSON Value through the channel
        // tx.send(json_value).await?;

        sleep(Duration::from_secs(1)).await;
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
