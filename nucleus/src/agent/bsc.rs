//! generate by OpenAI
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use ethabi::{Contract, Token};
use ethers_core::types::{Signature, TxHash};
use ethers_core::types::{Address, U256, U64};
use ethers_core::types::{Bytes, TransactionRequest};
use ethers_core::types::transaction::eip2718::TypedTransaction;
use ethers_signers::{LocalWallet, Signer};
use serde::{Deserialize, Serialize};
use vrs_core_sdk::{
    CallResult,
    http::{self, HttpMethod, HttpRequest, HttpResponse, RequestHead},
};
use vrs_core_sdk::tss::CryptoType;

use vemodel::{AccountId, Community, CommunityId, TokenMetadata};

use crate::agent::{bsc, GASPRICE_STORAGE_KEY, HttpCallType, trace};

pub const BSC_CHAIN_ID:u64 = 56;
pub const BSC_URL: &str = "https://bsc-dataseed.binance.org/";

pub(crate) fn initiate_query_bsc_transaction(tx_hash: &str) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getTransactionDataAndReceipt",
        "params": [tx_hash],
        "id": 1,
    });
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Post,
            uri: BSC_URL.to_string(),
            headers,
        },
        body: serde_json::to_vec(&body).expect("json;qed"),
    })
    .map_err(|e| e.to_string())?;
    Ok(response)
}

pub(crate) fn send_raw_transaction(raw_transaction: &str) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": [raw_transaction],
        "id": 1,
    });
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Post,
            uri: BSC_URL.to_string(),
            headers,
        },
        body: serde_json::to_vec(&body).expect("json;qed"),
    })
        .map_err(|e| e.to_string())?;
    Ok(response)
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TxData {
    block_hash: String,
    block_number: String,
    from: String,
    gas: String,
    gas_price: String,
    hash: String,
    input: String,
    nonce: String,
    to: Option<String>,
    transaction_index: String,
    value: String,
    #[serde(rename = "type")]
    tx_type: String,
    chain_id: String,
    v: String,
    r: String,
    s: String,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    block_hash: String,
    block_number: String,
    from: String,
    to: Option<String>,
    status: String,
    logs: Vec<Log>,
    transaction_hash: String,
    contract_address: Option<String>,
    gas_used: String,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Log {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub block_number: String,
    pub transaction_hash: String,
    pub transaction_index: String,
    pub block_hash: String,
    pub log_index: String,
    pub removed: bool,
}

#[derive(Deserialize, Debug)]
struct RpcResponse<T> {
    #[serde(rename = "result")]
    result: Option<T>,
}



#[derive(Deserialize, Debug)]
struct ResultData {
    #[serde(rename = "txData")]
    tx_data: TxData,
    #[allow(unused)]
    receipt: Receipt,
}

#[derive(Clone)]
pub struct TransactionDetails {
    pub amount_received: u128,
    pub sender: String,
    pub block_number: u64,
}

pub(crate) fn on_checking_bnb_transfer(
    recipient_addr: &str,
    response: CallResult<HttpResponse>,
) -> Result<Option<TransactionDetails>, Box<dyn std::error::Error>> {
    let response = response.map_err(|e| e.to_string())?;
    let response: RpcResponse<ResultData> = serde_json::from_slice(&response.body)
        .map_err(|e| format!("unable to deserialize body from BSC rpc: {:?}", e))?;
    if let Some(result_data) = response.result {
        let tx_data = &result_data.tx_data;

        if let Some(to_address) = &tx_data.to {
            if to_address.to_lowercase() == recipient_addr.to_lowercase() {
                let amount = u64::from_str_radix(&tx_data.value[2..], 16).unwrap_or(0);
                let block_number = u64::from_str_radix(&tx_data.block_number[2..], 16).unwrap_or(0);

                return Ok(Some(TransactionDetails {
                    amount_received: amount as u128,
                    sender: tx_data.from.clone(),
                    block_number,
                }));
            }
        }
    }
    Ok(None)
}

pub fn on_check_issue_result(response: CallResult<HttpResponse>,) -> Result<Option<String>, Box<dyn std::error::Error>>{
    let r = response.map_err(|e|e.to_string())?;
    let response: RpcResponse<ResultData> = serde_json::from_slice(&r.body)
        .map_err(|e| format!("unable to deserialize body from BSC rpc: {:?}", e))?;
    if let Some(result_data) = response.result {
        let receipt = result_data.receipt;
        if let Some(v) = receipt.logs.first() {
            return Ok(Some(v.address.clone()));
        }
    }
    Ok(None)
}
pub(crate) fn issuse_token(community: &Community, community_id: &CommunityId) -> Result<(), String> {
    let contract_bytecode = hex::decode(
        &fs::read_to_string(Path::new("../token.bytecode")).expect("failed  to read bytecode file").trim()
    ).expect("invalid bytecode");
    let contract_abi = Contract::load(
        fs::File::open(Path::new("build/ERC20.abi"))
            .expect("Failed to read ABI file")
    ).expect("invalid abi");
    let token = community.token_info.clone();
    let constructor_args = ethabi::encode(&[
        Token::String(token.name.clone()),
        Token::String(token.symbol.clone()),
        Token::Uint(token.decimals.into()),
        Token::Uint(token.total_issuance.into()),
    ]);
    let full_bytecode = [contract_bytecode, constructor_args].concat();
    let gas_price: Option<u64> = crate::find(GASPRICE_STORAGE_KEY.as_bytes()).unwrap_or_default();
    let gas_price = gas_price.map(|s|U256::from(s));
    let addr = community.agent_pubkey.clone();
    let addr = Address::from_slice(addr.0.as_slice());
    let tx = TransactionRequest {
        from: Some(addr),
        to: None,
        gas: Some(U256::from(1000000)),
        gas_price,
        value: None,
        data: Some(Bytes::from(full_bytecode)),
        nonce: Some(U256::from(0)),
        chain_id: Some(U64::from(56)),
    };
    let tx = TypedTransaction::Legacy(tx);
    let sign_hash = tx.sighash();
    let r = vrs_core_sdk::tss::tss_sign(CryptoType::Secp256k1, community_id.to_be_bytes(), sign_hash.0).map_err(|e|e.to_string())?;
    let v: u64 = r.last().unwrap().clone() as u64 + BSC_CHAIN_ID * 2 + 35 ;
    let signature = Signature {
        v,
        r: U256::from_big_endian(&r[0..32]),
        s: U256::from_big_endian(&r[32..64]),
    };
    let signed_tx = tx.rlp_signed(&signature);
    let raw = format!("0x{}", hex::encode(signed_tx.to_vec()));
    let id = send_raw_transaction(raw.as_str()).expect("send raw error");
    trace(id, HttpCallType::SendIssueTx(community.id())).map_err(|e| e.to_string()).expect("send Issue tx error");
    Ok(())
}



pub(crate) fn check_gas_price( response: CallResult<HttpResponse>,) -> Result<Option<u64>, Box<dyn std::error::Error>> {
    let response = response.map_err(|e| e.to_string())?;
    let response: RpcResponse<String> = serde_json::from_slice(&response.body)
        .map_err(|e| format!("unable to deserialize body from BSC rpc: {:?}", e))?;
    if let Some(result_data) = response.result {
        let price = u64::from_str_radix(&result_data[2..],16)?;
        return Ok(Some(price))
    }
    Ok(None)
}

pub fn query_gas_price() -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_gasPrice",
        "params": [],
        "id": 1,
    });
    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Post,
            uri: BSC_URL.to_string(),
            headers,
        },
        body: serde_json::to_vec(&body).expect("json;qed"),
    })
        .map_err(|e| e.to_string())?;
    Ok(response)
}

pub fn untrace_issue_tx(
                        response: CallResult<HttpResponse>,) -> Result<Option<TxHash>, Box<dyn std::error::Error>> {
    let response = response.map_err(|e| e.to_string())?;
    let response: crate::agent::bsc::RpcResponse<String> = serde_json::from_slice(&response.body)
        .map_err(|e| format!("unable to deserialize body from BSC rpc: {:?}", e))?;
    if let Some(result_data) = response.result {
        let tx_hash = TxHash::from_slice(hex::decode(&result_data[2..])?.as_slice());
        return Ok(Some(tx_hash));
    }
    Ok(None)
}