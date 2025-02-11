use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::{core::client::ClientT, rpc_params};
use parity_scale_codec::{Decode, Encode};
use vemodel::*;

pub async fn get_community<T: ClientT>(
    client: &T,
    nucleus_id: &AccountId,
    community_id: CommunityId,
) -> Result<Option<Community>, Box<dyn std::error::Error>> {
    let params = rpc_params![
        nucleus_id.to_string(),
        "get_community",
        hex::encode(community_id.encode())
    ];
    let hex_str: String = client.request("nucleus_get", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<Option<Community>, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn get_contents<T: ClientT>(
    client: &T,
    nucleus_id: &AccountId,
    content_id: ContentId,
) -> Result<Vec<(ContentId, Vec<u8>)>, Box<dyn std::error::Error>> {
    let params = rpc_params![
        nucleus_id.to_string(),
        "get_raw_contents",
        hex::encode((content_id, 100u32).encode())
    ];
    let hex_str: String = client.request("nucleus_get", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<Vec<(ContentId, Vec<u8>)>, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn get_content<T: ClientT>(
    client: &T,
    nucleus_id: &AccountId,
    content_id: ContentId,
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    let params = rpc_params![
        nucleus_id.to_string(),
        "get_raw_content",
        hex::encode(content_id.encode())
    ];
    let hex_str: String = client.request("nucleus_get", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<Option<Vec<u8>>, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn get_events<T: ClientT>(
    client: &T,
    nucleus_id: &AccountId,
    event_id: EventId,
) -> Result<Vec<(EventId, Event)>, Box<dyn std::error::Error>> {
    let params = rpc_params![
        nucleus_id.to_string(),
        "get_events",
        hex::encode((event_id, 100u32).encode())
    ];
    let hex_str: String = client.request("nucleus_get", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<Vec<(EventId, Event)>, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub fn build_client(url: &str) -> HttpClient {
    HttpClientBuilder::default().build(url).unwrap()
}
