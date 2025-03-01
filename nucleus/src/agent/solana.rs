use std::collections::BTreeMap;
use vrs_core_sdk::{
    http::{self, HttpMethod, HttpRequest, HttpResponse, RequestHead},
    CallResult,
};

pub(crate) fn initiate_checking_transfer(tx_hash: &str) -> Result<u64, String> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [tx_hash, "json"]
    });

    let response = http::request(HttpRequest {
        head: RequestHead {
            method: HttpMethod::Post,
            uri: "https://mainnet.helius-rpc.com/?api-key=64dbe6d2-9641-43c6-bb86-0e3d748f31b1"
                .to_string(),
            headers,
        },
        body: serde_json::to_vec(&body).expect("json;qed"),
    })
    .map_err(|e| e.to_string())?;
    Ok(response)
}

/// generate by OpenAI
pub(crate) fn on_checking_transfer(
    target_addr: &str,
    response: CallResult<HttpResponse>,
) -> Result<u64, Box<dyn std::error::Error>> {
    let response = response.map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_slice(&response.body)
        .map_err(|e| format!("unable to deserialize body from llm: {:?}", e))?;
    let result = v.get("result").ok_or("Missing 'result'")?;
    let meta = result.get("meta").ok_or("Missing 'meta'")?;
    let err = meta.get("err");

    if err.is_some() {
        return Err("solana RPC error".into());
    }

    let transaction = result.get("transaction").ok_or("Missing 'transaction'")?;
    let message = transaction.get("message").ok_or("Missing 'message'")?;
    let account_keys = message.get("accountKeys").ok_or("Missing 'accountKeys'")?;
    let post_balances = meta.get("postBalances").ok_or("Missing 'postBalances'")?;
    let pre_balances = meta.get("preBalances").ok_or("Missing 'preBalances'")?;

    let index = account_keys
        .as_array()
        .and_then(|keys| keys.iter().position(|k| k.as_str() == Some(&target_addr)))
        .ok_or("Target address not found")?;
    let pre_balance = pre_balances[index].as_u64().ok_or("Invalid pre balance")?;
    let post_balance = post_balances[index]
        .as_u64()
        .ok_or("Invalid post balance")?;
    let received_amount = post_balance
        .checked_sub(pre_balance)
        .ok_or("Checked subtraction failed")?;

    Ok(received_amount)
}
