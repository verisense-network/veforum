use ethabi::Token;
use vrs_core_sdk::tss::{tss_sign, CryptoType};

use vemodel::{Community, RewardPayload};

use crate::eth_types::hash::keccak256;
use crate::eth_types::Address;
use crate::trie::to_reward_seq_key;

pub fn generate_rewards(to: Address, amt: u128, community: &Community) -> Option<RewardPayload> {
    let seq_key = to_reward_seq_key(community.id());
    let current_sequence: u64 = crate::find(&seq_key)
        .unwrap_or_default()
        .unwrap_or_default();
    let seq = current_sequence + 1;
    let _ = crate::save(&seq_key, &seq);
    let reward_data = ethabi::encode(&[
        Token::Uint(seq.into()),
        Token::Address(to),
        Token::Uint(amt.into()),
    ]);
    let prefix = format!("\x19Ethereum Signed Message:\n{}", reward_data.len());
    let mut prefixed_message = prefix.as_bytes().to_vec();
    prefixed_message.extend_from_slice(reward_data.as_slice());
    let message_hash = keccak256(&prefixed_message);
    match tss_sign(
        CryptoType::EcdsaSecp256k1,
        community.id().to_be_bytes(),
        message_hash,
    ) {
        Ok(s) => {
            let payload = RewardPayload {
                payload: reward_data.to_vec(),
                signature: s,
                agent_contract: community.agent_contract.unwrap(),
                token_symbol: community.token_info.symbol.clone(),
                token_contract: community.token_info.contract.clone(),
                withdrawed: false,
            };
            Some(payload)
        }
        Err(_) => {
            vrs_core_sdk::println!("generate reward signature failed {:?} {} {}", to, seq, amt);
            None
        }
    }
}
