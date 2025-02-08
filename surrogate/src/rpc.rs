use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::{core::client::ClientT, rpc_params};
use memmap2::MmapMut;
use parity_scale_codec::{Decode, Encode};
use vemodel::{args::*, *};

pub async fn create_community<T: ClientT>(
    client: T,
    nucleus_id: AccountId,
    signer: AccountId,
    args: CreateCommunityArg,
) -> Result<CommunityId, Box<dyn std::error::Error>> {
    let payload = hex::encode((signer, args).encode());
    let params = rpc_params![nucleus_id.to_string(), "create_community", payload];
    let hex_str: String = client.request("nucleus_post", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<CommunityId, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn post_thread<T: ClientT>(
    client: T,
    nucleus_id: AccountId,
    signer: AccountId,
    args: PostThreadArg,
) -> Result<ContentId, Box<dyn std::error::Error>> {
    let payload = hex::encode((signer, args).encode());
    let params = rpc_params![nucleus_id.to_string(), "post_thread", payload];
    let hex_str: String = client.request("nucleus_post", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<ContentId, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn post_comment<T: ClientT>(
    client: T,
    nucleus_id: AccountId,
    signer: AccountId,
    args: PostCommentArg,
) -> Result<ContentId, Box<dyn std::error::Error>> {
    let payload = hex::encode((signer, args).encode());
    let params = rpc_params![nucleus_id.to_string(), "post_comment", payload];
    let hex_str: String = client.request("nucleus_post", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<ContentId, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn get_community<T: ClientT>(
    client: T,
    nucleus_id: AccountId,
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
    client: T,
    nucleus_id: AccountId,
    content_id: ContentId,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = rpc_params![
        nucleus_id.to_string(),
        "get_raw_contents",
        hex::encode((content_id, 100u32).encode())
    ];
    let hex_str: String = client.request("nucleus_get", params).await?;
    let hex = hex::decode(&hex_str)?;
    let r = Result::<Vec<(ContentId, Vec<u8>)>, String>::decode(&mut &hex[..])??;
    for (id, data) in r {
        if id & 0x00000000_00000000_00000000_ffffffff == 0 {
            let thread = Thread::decode(&mut &data[..])?;
            println!("{:?}: {:?}", id, thread);
        } else {
            let comment = Comment::decode(&mut &data[..])?;
            println!("{:?}: {:?}", id, comment);
        }
    }
    Ok(())
}

pub async fn get_events<T: ClientT>(
    client: T,
    nucleus_id: AccountId,
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

pub async fn poll(url: impl AsRef<str>, nucleus_id: AccountId) -> anyhow::Result<()> {
    let rpc_client = HttpClientBuilder::default().build(url.as_ref()).await?;
    let mmap = mmap("./events")?;
    let event_id = EventId::from_be_bytes(*mmap[0..=8]);
    get_events(rpc_client, nucleus_id, event_id).await?;
}

fn mmap(f: impl AsRef<std::path::Path>) -> anyhow::Result<MmapMut> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(f)?;
    let mut file_size = file.metadata()?.len();
    if file_size == 0 {
        let initial_value: u64 = 0;
        file.write_all(&initial_value.to_be_bytes())?;
        file_size = 8;
    }
    Ok(unsafe { MmapMut::map_mut(&file)? })
}
