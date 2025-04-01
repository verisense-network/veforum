mod cli;

use ed25519_dalek::{Signer, SigningKey};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::{core::client::ClientT, rpc_params};
use parity_scale_codec::{Decode, Encode};
use vemodel::{args::*, *};
use vrs_core_sdk::{NucleusId};

// TODO
pub async fn set_openai_key<T: ClientT>(
    client: T,
    nucleus_id: NucleusId,
    key: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = hex::encode(key.encode());
    let params = rpc_params![nucleus_id.to_string(), "set_llm_key", payload];
    let hex_str: String = client.request("nucleus_post", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<(), String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn get_balances<T: ClientT>(
    client: T,
    nucleus_id: NucleusId,
    account_id: AccountId,
) -> Result<Vec<(Community, u64)>, Box<dyn std::error::Error>> {
    let payload = hex::encode((account_id, None::<CommunityId>, 100u32).encode());
    let params = rpc_params![nucleus_id.to_string(), "get_balances", payload];
    let hex_str: String = client.request("nucleus_get", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<Vec<(Community, u64)>, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

/*pub async fn create_community<T: ClientT>(
    client: T,
    nucleus_id: NucleusId,
    args: CreateCommunityArg,
    signer: SigningKey,
) -> Result<CommunityId, Box<dyn std::error::Error>> {
    let account_id = AccountId(signer.verifying_key().to_bytes());
    let account = get_account_info(&client, &nucleus_id, account_id.clone()).await?;
    let mut args = Args {
        signature: Signature([0u8; 64]),
        signer: account_id,
        nonce: account.nonce,
        payload: args,
    };
    let signature = signer.sign(args.to_be_signed().as_ref());
    args.signature = Signature(signature.to_bytes());
    let payload = hex::encode(args.encode());
    let params = rpc_params![nucleus_id.to_string(), "create_community", payload];
    let hex_str: String = client.request("nucleus_post", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<CommunityId, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn activate_community<T: ClientT>(
    client: T,
    nucleus_id: NucleusId,
    args: ActivateCommunityArg,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = hex::encode(args.encode());
    let params = rpc_params![nucleus_id.to_string(), "activate_community", payload];
    let hex_str: String = client.request("nucleus_post", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<(), String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn post_thread<T: ClientT>(
    client: T,
    nucleus_id: NucleusId,
    args: PostThreadArg,
    signer: SigningKey,
) -> Result<ContentId, Box<dyn std::error::Error>> {
    let account_id = AccountId(signer.verifying_key().to_bytes());
    let account = get_account_info(&client, &nucleus_id, account_id).await?;
    let mut args = Args {
        signature: Signature([0u8; 64]),
        signer: account_id,
        nonce: account.nonce,
        payload: args,
    };
    let signature = signer.sign(args.to_be_signed().as_ref());
    args.signature = Signature(signature.to_bytes());
    let payload = hex::encode(args.encode());
    let params = rpc_params![nucleus_id.to_string(), "post_thread", payload];
    let hex_str: String = client.request("nucleus_post", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<ContentId, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn post_comment<T: ClientT>(
    client: T,
    nucleus_id: NucleusId,
    args: PostCommentArg,
    signer: SigningKey,
) -> Result<ContentId, Box<dyn std::error::Error>> {
    let account_id = AccountId(signer.verifying_key().to_bytes());
    let account = get_account_info(&client, &nucleus_id, account_id).await?;
    let mut args = Args {
        signature: Signature([0u8; 64]),
        signer: account_id,
        nonce: account.nonce,
        payload: args,
    };
    let signature = signer.sign(args.to_be_signed().as_ref());
    args.signature = Signature(signature.to_bytes());
    let payload = hex::encode(args.encode());
    let params = rpc_params![nucleus_id.to_string(), "post_comment", payload];
    let hex_str: String = client.request("nucleus_post", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<ContentId, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}
*/
pub async fn get_community<T: ClientT>(
    client: T,
    nucleus_id: NucleusId,
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

pub async fn get_account_info<T: ClientT>(
    client: &T,
    nucleus_id: &NucleusId,
    account_id: AccountId,
) -> Result<Account, Box<dyn std::error::Error>> {
    let params = rpc_params![
        nucleus_id.to_string(),
        "get_account_info",
        hex::encode(account_id.encode())
    ];
    let hex_str: String = client.request("nucleus_get", params).await?;
    let hex = hex::decode(&hex_str)?;
    Result::<Account, String>::decode(&mut &hex[..])?.map_err(|e| e.into())
}

pub async fn get_contents<T: ClientT>(
    client: T,
    nucleus_id: NucleusId,
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
        if id & 0xffffffff == 0 {
            let thread =
                Thread::decode(&mut &data[..]).inspect_err(|_| println!("error at {}", id))?;
            println!("{}: {}", id, serde_json::to_string(&thread).unwrap());
        } else {
            let comment = Comment::decode(&mut &data[..])?;
            println!("{}: {}", id, serde_json::to_string(&comment).unwrap());
        }
    }
    Ok(())
}

pub async fn get_events<T: ClientT>(
    client: T,
    nucleus_id: NucleusId,
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
    match cli.cmd {
    /*    SubCmd::CreateCommunity(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            let signer = cli.options.get_signer().unwrap();
            match create_community(client, nucleus_id, cmd.into(), signer).await {
                Ok(id) => println!("{:2x}", id),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        SubCmd::ActivateCommunity(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            match activate_community(client, nucleus_id, cmd.into()).await {
                Ok(()) => println!("Activating"),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        SubCmd::PostThread(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            let signer = cli.options.get_signer().unwrap();
            match post_thread(client, nucleus_id, cmd.into(), signer).await {
                Ok(id) => println!("Thread ID = {:2x}", id),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        SubCmd::PostComment(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            let signer = cli.options.get_signer().unwrap();
            match post_comment(client, nucleus_id, cmd.into(), signer).await {
                Ok(id) => println!("{}", id),
                Err(e) => eprintln!("{:?}", e),
            }
        }*/
        SubCmd::GetCommunity(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            let community_id = cmd.id;
            match get_community(client, nucleus_id, community_id).await {
                Ok(Some(community)) => println!("{}", serde_json::to_string(&community).unwrap()),
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
        SubCmd::SetKey(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            match set_openai_key(client, nucleus_id, cmd.key).await {
                Ok(_) => println!("Key set"),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        SubCmd::GetBalances(cmd) => {
            let client = build_client(&cli.options.get_rpc());
            match get_balances(client, nucleus_id, cmd.account).await {
                Ok(balances) => {
                    for (community, balance) in balances {
                        println!("{}: {}", community.name, balance);
                    }
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }
        _ => {}
    }
}
