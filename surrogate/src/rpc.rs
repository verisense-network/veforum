use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use memmap2::MmapMut;
use vemodel::{AccountId, EventId};

pub async fn poll(url: impl AsRef<str>, nucleus_id: AccountId) -> anyhow::Result<()> {
    let rpc_client = HttpClientBuilder::default().build(url.as_ref()).await?;
    let mmap = mmap("./events")?;
    let event_id = EventId::from_be_bytes(*mmap[0..=8]);
    let params = rpc_params![
        nucleus_id.to_string(),
        "get_events",
        hex::encode((event_id + 1, 100).encode())
    ];
    let res: Result<serde_json::Value, _> = http_client.request("nucleus_get", params).await;
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
