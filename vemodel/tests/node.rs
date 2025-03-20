use js_sys::JSON;
use serde_json::{json, Value::Null};
use vemodel::{args::*, *};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[cfg_attr(feature = "wasm-bind", wasm_bindgen_test)]
pub fn encode_create_community_arg_test() {
    let json_str = r#"{
        "name": "JOKE",
        "private": false,
        "slug": "推翻人类暴政，地球属于三体！",
        "logo": "",
        "description": "推翻人类暴政，地球属于三体！",
        "prompt": "为地狱笑话帖子和回复评分，如果非常好笑就适当发一些JOKE代币，不要对听过的笑话奖励",
        "token": {
            "symbol": "JOKE",
            "total_issuance": 10000000000,
            "decimals": 2,
            "image": null
        },
        "llm_name": "OpenAI",
        "llm_api_host": null,
        "llm_key": null
    }"#;

    let test_data = JSON::parse(json_str).unwrap();

    let value = encode_create_community_arg(test_data.clone());

    let decoded = decode_create_community_arg(value.unwrap()).unwrap();

    let community: CreateCommunityArg = serde_wasm_bindgen::from_value(test_data.clone()).unwrap();
    let decoded_community: CreateCommunityArg =
        serde_wasm_bindgen::from_value(decoded.clone()).unwrap();

    assert_eq!(community.name, decoded_community.name);
}
