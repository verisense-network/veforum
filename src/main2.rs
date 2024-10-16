use axum::{
    routing::get,
    Router, Json, extract::State,
};
use jsonrpsee::{
    core::client::ClientT,
    http_client::HttpClientBuilder,
    rpc_params,
};
use serde_json::{Value, json};
use std::time::Duration;
use std::sync::Arc;
use tokio::time;
use redis::AsyncCommands;
use meilisearch_sdk::client::Client as MeiliClient;

// JSON-RPC client using jsonrpsee
struct JsonRpcClient {
    client: jsonrpsee::http_client::HttpClient,
}

impl JsonRpcClient {
    async fn new(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = HttpClientBuilder::default().build(url)?;
        Ok(Self { client })
    }

    async fn call_method(&self, method: &str, params: Vec<Value>) -> Result<Value, Box<dyn std::error::Error>> {
        let params = rpc_params!(params);
        let result = self.client.request(method, params).await?;
        Ok(result)
    }
}

// Shared state
#[derive(Clone)]
struct AppState {
    redis: redis::Client,
    meilisearch: MeiliClient,
}

// Interval task to retrieve JSON from background JSON-RPC server and store in Redis
async fn interval_task(client: JsonRpcClient, redis: redis::Client) {
    let mut interval = time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;
        match client.call_method("get_data", vec![]).await {
            Ok(data) => {
                let mut conn = redis.get_async_connection().await.expect("Failed to connect to Redis");
                let _: () = conn.set("latest_data", data.to_string()).await.expect("Failed to set data in Redis");
                println!("Retrieved and stored data in Redis");
            },
            Err(e) => eprintln!("Error retrieving data: {:?}", e),
        }
    }
}

// Task to poll Redis and push to Meilisearch
async fn redis_to_meilisearch_task(state: AppState) {
    let mut interval = time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;
        let mut conn = state.redis.get_async_connection().await.expect("Failed to connect to Redis");
        let data: String = conn.get("latest_data").await.expect("Failed to get data from Redis");
        
        if !data.is_empty() {
            let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");
            
            // Assuming the data is an array of documents
            if let Value::Array(documents) = value {
                match state.meilisearch.index("my_index").add_documents(&documents, None).await {
                    Ok(_) => {
                        println!("Successfully added documents to Meilisearch");
                        // Clear the data from Redis after successful insertion
                        let _: () = conn.set("latest_data", "").await.expect("Failed to clear data in Redis");
                    },
                    Err(e) => eprintln!("Error adding documents to Meilisearch: {:?}", e),
                }
            }
        }
    }
}

// Axum route handlers
async fn hello_world() -> &'static str {
    "Hello, World!"
}

async fn get_status(State(state): State<AppState>) -> Json<Value> {
    let mut conn = state.redis.get_async_connection().await.expect("Failed to connect to Redis");
    let data: String = conn.get("latest_data").await.expect("Failed to get data from Redis");
    
    if data.is_empty() {
        Json(json!({"status": "No data available"}))
    } else {
        Json(json!({"status": "OK", "data": data}))
    }
}

#[tokio::main]
async fn main() {
    // Initialize JSON-RPC client
    let client = JsonRpcClient::new("http://localhost:8080")
        .await
        .expect("Failed to create JSON-RPC client");

    // Initialize Redis client
    let redis_client = redis::Client::open("redis://127.0.0.1/").expect("Failed to create Redis client");

    // Initialize Meilisearch client
    let meilisearch_client = MeiliClient::new("http://localhost:7700", "masterKey");

    // Create shared state
    let state = AppState {
        redis: redis_client.clone(),
        meilisearch: meilisearch_client,
    };

    // Spawn interval task
    tokio::spawn(interval_task(client, redis_client.clone()));

    // Spawn Redis to Meilisearch task
    tokio::spawn(redis_to_meilisearch_task(state.clone()));

    // Build Axum application
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/status", get(get_status))
        .with_state(state);

    // Run the server
    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

