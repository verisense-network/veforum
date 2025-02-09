mod cli;

use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::{core::client::ClientT, rpc_params};
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
    println!("{}", payload);
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

use crate::cli::*;
use clap::Parser;

#[tokio::main]
pub async fn main() {
    let cli = Cli::parse();
    let nucleus_id = cli.options.get_nucleus().expect("invalid nucleus id");
    let account_id = cli.options.get_nucleus().expect("invalid account id");
    match cli.cmd {
        SubCmd::CreateCommunity(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            match create_community(client, nucleus_id, account_id, cmd.into()).await {
                Ok(id) => println!("{:2x}", id),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        SubCmd::PostThread(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            match post_thread(client, nucleus_id, account_id, cmd.into()).await {
                Ok(id) => println!("Thread ID = {:2x}", id),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        SubCmd::PostComment(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            match post_comment(client, nucleus_id, account_id, cmd.into()).await {
                Ok(id) => println!("{}", id),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        SubCmd::GetCommunity(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            let community_id = cmd.id;
            match get_community(client, nucleus_id, community_id).await {
                Ok(Some(community)) => println!("{:?}", community),
                Ok(None) => eprintln!("Community not found"),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        SubCmd::GetContent(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            get_contents(client, nucleus_id, cmd.id).await.unwrap();
        }
        SubCmd::GetEvents(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            match get_events(client, nucleus_id, cmd.id).await {
                Ok(events) => {
                    for (id, event) in events {
                        println!("{:?}: {:?}", id, event);
                    }
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }
}
