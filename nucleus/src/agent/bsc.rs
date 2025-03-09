//! generate by OpenAI
use serde::Deserialize;
use std::collections::BTreeMap;
use vrs_core_sdk::{
    http::{self, HttpMethod, HttpRequest, HttpResponse, RequestHead},
    CallResult,
};

pub(crate) fn initiate_checking_bnb_transfer(tx_hash: &str) -> Result<u64, String> {
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
            uri: "https://bsc-dataseed.binance.org/".to_string(),
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
    logs: Vec<String>,
    transaction_hash: String,
    contract_address: Option<String>,
    gas_used: String,
}

#[derive(Deserialize, Debug)]
struct RpcResponse {
    #[serde(rename = "result")]
    result: Option<ResultData>,
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
    let response: RpcResponse = serde_json::from_slice(&response.body)
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
