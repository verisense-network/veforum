use clap::{Parser, Subcommand};
use vemodel::*;
use vrs_core_sdk::NucleusId;

#[derive(Debug, Parser)]
#[command(author = "Verisense Team <dev@verisense.network>", version)]
pub struct Cli {
    #[clap(flatten)]
    pub options: Options,

    #[clap(subcommand)]
    pub cmd: SubCmd,
}

#[derive(Debug, Parser)]
pub struct Options {
    #[arg(short, long, global = true, help = "Display verbose output")]
    pub verbose: bool,

    #[arg(
        long,
        global = true,
        help = "Connect the devnet",
        default_value = "true",
        conflicts_with_all = ["rpc"]
    )]
    pub devnet: bool,

    #[arg(
        long,
        global = true,
        help = "The custom RPC endpoint to connect to. E.g. \"ws://localhost:9944\"",
        conflicts_with_all = ["devnet"]
    )]
    pub rpc: Option<String>,

    #[arg(long, global = true, help = "The vrx home path, default to \"~/.vrx\"")]
    pub vrx_dir: Option<std::path::PathBuf>,

    #[arg(short, long, global = true, help = "The private key to use")]
    pub key: Option<String>,

    #[arg(long, help = "The nucleus to request")]
    pub nucleus: String,
}

pub(crate) const DEV_RPC_HOST: &'static str = "wss://alpha-devnet.verisense.network";

impl Options {
    pub(crate) fn get_rpc(&self) -> String {
        match self.rpc {
            Some(ref rpc) => rpc.clone(),
            None => {
                if self.devnet {
                    DEV_RPC_HOST.to_string()
                } else {
                    panic!("Please specify the RPC endpoint")
                }
            }
        }
    }

    pub(crate) fn get_nucleus(&self) -> Result<NucleusId, String> {
        use std::str::FromStr;
        let account = NucleusId::from_str(&self.nucleus);
        match account {
            Ok(account) => Ok(account),
            Err(_) => Err("Invalid nucleus address".to_string()),
        }
    }

    pub(crate) fn get_signer(&self) -> Result<ed25519_dalek::SigningKey, String> {
        self.key
            .as_ref()
            .ok_or("Please specify the private key".to_string())
            .and_then(|key| {
                let key = hex::decode(key.trim_start_matches("0x")).map_err(|e| e.to_string())?;
                let key: [u8; 32] = key
                    .try_into()
                    .map_err(|_| "invalid key: 32 bytes expected".to_string())?;
                Ok(ed25519_dalek::SigningKey::from_bytes(&key))
            })
    }
}

#[derive(Debug, Subcommand)]
pub enum SubCmd {
    CreateCommunity(CommunityCommand),
    ActivateCommunity(ActivateCommand),
    PostThread(ThreadCommand),
    PostComment(CommentCommand),
    GetCommunity(GetCommunityCommand),
    GetContent(GetContentCommand),
    GetEvents(GetEventsCommand),
    SetKey(SetKeyCommand),
    GetBalances(GetBalancesCommand),
}

#[derive(Debug, Parser)]
#[command(about = "Get a community")]
pub struct GetCommunityCommand {
    #[arg(help = "The community id")]
    pub id: CommunityId,
}

#[derive(Debug, Parser)]
#[command(about = "Activate a community")]
pub struct ActivateCommand {
    #[arg(long, help = "The community name")]
    pub community: String,

    #[arg(long, help = "The solana tx")]
    pub tx: String,
}

#[derive(Debug, Parser)]
#[command(about = "Get 100 contents")]
pub struct GetContentCommand {
    #[arg(help = "The thread id")]
    pub id: ContentId,
}

#[derive(Debug, Parser)]
#[command(about = "Get 100 events")]
pub struct GetEventsCommand {
    pub id: EventId,
}

#[derive(Debug, Parser)]
#[command(about = "Create a new community")]
pub struct CommunityCommand {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub slug: String,
    #[arg(long)]
    pub description: String,
    #[arg(long)]
    pub prompt: String,
    #[arg(long)]
    pub token_name: String,
    #[arg(long)]
    pub token_decimals: u8,
    #[arg(long)]
    pub token_total_supply: u64,
}

use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;

impl Into<vemodel::args::CreateCommunityArg> for CommunityCommand {
    fn into(self) -> vemodel::args::CreateCommunityArg {
        vemodel::args::CreateCommunityArg {
            name: self.name,
            logo: Default::default(),
            private: false,
            slug: self.slug,
            token: vemodel::args::TokenMetadataArg {
                symbol: self.token_name,
                decimals: self.token_decimals,
                total_issuance: self.token_total_supply,
                image: None,
            },
            description: self.description,
            prompt: self.prompt,
            llm_name: "OpenAI".to_string(),
            llm_api_host: None,
            llm_key: None,
        }
    }
}

impl Into<vemodel::args::ActivateCommunityArg> for ActivateCommand {
    fn into(self) -> vemodel::args::ActivateCommunityArg {
        vemodel::args::ActivateCommunityArg {
            community: self.community,
            tx: self.tx,
        }
    }
}

#[derive(Debug, Parser)]
#[command(about = "Create a new thread")]
pub struct ThreadCommand {
    #[arg(long)]
    pub community: String,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub content: String,
}

impl Into<vemodel::args::PostThreadArg> for ThreadCommand {
    fn into(self) -> vemodel::args::PostThreadArg {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(self.content.as_bytes()).unwrap();
        let stream = encoder.finish().unwrap();
        vemodel::args::PostThreadArg {
            community: self.community,
            title: self.title,
            content: stream,
            images: vec![],
            mention: vec![],
        }
    }
}

#[derive(Debug, Parser)]
#[command(about = "Create a new comment")]
pub struct CommentCommand {
    #[arg(long)]
    pub thread: String,
    #[arg(long)]
    pub content: String,
}

impl Into<vemodel::args::PostCommentArg> for CommentCommand {
    fn into(self) -> vemodel::args::PostCommentArg {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(self.content.as_bytes()).unwrap();
        let stream = encoder.finish().unwrap();
        vemodel::args::PostCommentArg {
            thread: self.thread.parse().expect("invalid thread id"),
            content: self.content,
            images: vec![],
            mention: vec![],
            reply_to: None,
        }
    }
}

#[derive(Debug, Parser)]
#[command(about = "Set a LLM key")]
pub struct SetKeyCommand {
    #[arg(long)]
    pub key: String,
}

#[derive(Debug, Parser)]
#[command(about = "Get balances")]
pub struct GetBalancesCommand {
    #[arg(long)]
    pub account: AccountId,
}
