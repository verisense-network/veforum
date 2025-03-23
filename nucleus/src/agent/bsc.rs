//! generate by OpenAI
use std::collections::BTreeMap;

use ethabi::{Token};
use serde::{Deserialize, Serialize};
use vrs_core_sdk::{
    CallResult,
    http::{self, HttpMethod, HttpRequest, HttpResponse, RequestHead},
};
use vrs_core_sdk::tss::CryptoType;

use vemodel::{Community, CommunityId};

use crate::agent::{GASPRICE_STORAGE_KEY, HttpCallType, trace};
use crate::agent::contract::{BYTECODE};
use crate::eth_types::{Address, U64, U256, TxHash};
use crate::eth_types::bytes::Bytes;
use crate::eth_types::signature::Signature;
use crate::eth_types::transaction::{TransactionRequest};
use crate::eth_types::typed_transaction::TypedTransaction;

pub const BSC_CHAIN_ID:u64 = 56;
pub const BSC_URL: &str = "https://bsc-dataseed.binance.org";

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
#[derive(Deserialize, Serialize, Debug)]
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
#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
struct RpcResponse<T> {
    #[serde(rename = "result")]
    result: Option<T>,
}



#[derive(Deserialize, Serialize, Debug)]
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

pub fn on_check_issue_result(response: CallResult<HttpResponse>,) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>>{
    let r = response.map_err(|e|e.to_string())?;
    let response: RpcResponse<ResultData> = serde_json::from_slice(&r.body)
        .map_err(|e| format!("unable to deserialize body from BSC rpc: {:?}", e))?;
    vrs_core_sdk::println!("resp: {:?}",serde_json::to_string(&response));
    if let Some(result_data) = response.result {
        let logs = result_data.receipt.logs;

        let first = logs.get(0).map(|l|l.address.clone());
        let second = logs.get(1).map(|l|l.address.clone());
        return Ok((first, second));
    }
    Ok((None, None))
}

pub fn issuse_token(community: &Community, community_id: &CommunityId) -> Result<(), String> {
    let contract_bytecode = hex::decode(BYTECODE.trim_start_matches("0x")).expect("invalid bytecode");
    let token = community.token_info.clone();
    let constructor_args = ethabi::encode(&[
            Token::String(token.symbol.clone()),
            Token::String(token.symbol.clone()),
            Token::Uint(token.decimals.into()),
            Token::Uint(token.total_issuance.into()),
            Token::Bool(true),
            Token::Address(Address::zero()),
    ]);
    let full_bytecode = [contract_bytecode, constructor_args].concat();
    let gas_price: Option<u64> = crate::find(GASPRICE_STORAGE_KEY.as_bytes()).unwrap_or_default();
    let gas_price = gas_price.map(|s|U256::from(s));
    let addr = community.agent_pubkey.clone();
    let addr = Address::from_slice(addr.0.as_slice());
    let tx = TransactionRequest {
        from: Some(addr),
        to: None,
        gas: Some(U256::from(2000000)),
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
    let resp = String::from_utf8(response.body).map_err(|e| e.to_string())?;
    vrs_core_sdk::println!("send issue result: {}", &resp );
    let response: crate::agent::bsc::RpcResponse<String> = serde_json::from_str(&resp)
        .map_err(|e| format!("unable to deserialize body from BSC rpc: {:?}", e))?;
    if let Some(result_data) = response.result {
        let tx_hash = TxHash::from_slice(hex::decode(&result_data[2..])?.as_slice());
        return Ok(Some(tx_hash));
    }
    Ok(None)
}