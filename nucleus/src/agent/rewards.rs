use ethabi::Token;
use vrs_core_sdk::tss::{CryptoType, tss_sign};

use vemodel::{AccountId, Community, CommunityId, RewardPayload};

use crate::eth_types::Address;
use crate::eth_types::bytes::Bytes;
use crate::eth_types::hash::keccak256;

pub const  SEQUENCE_KEY: &str = "SEQUENCE_KEY";


pub fn generate_rewards(to: Address, amt: u128,  community: &Community) -> Option<RewardPayload> {
    let current_sequence: u64 = crate::find(SEQUENCE_KEY.as_bytes()).unwrap_or_default().unwrap_or_default();
    let seq = current_sequence+1;
    let _ = crate::save(SEQUENCE_KEY.as_bytes(), &seq);
    let reward_data = ethabi::encode(&[Token::Uint(seq.into()), Token::Address(to), Token::Uint(amt.into())]);
    let prefix = format!("\x19Ethereum Signed Message:\n{}", reward_data.len());
    let mut prefixed_message = prefix.as_bytes().to_vec();
    prefixed_message.extend_from_slice(reward_data.as_slice());
    let message_hash = keccak256(&prefixed_message);
    match tss_sign(CryptoType::Secp256k1, community.id().to_be_bytes(), message_hash) {
        Ok(s) => {
            let payload = RewardPayload {
                payload: reward_data.to_vec(),
                signature: s,
                agent_contract: community.agent_contract.unwrap(),
            };
            Some(payload)
        }
        Err(_) => {
            vrs_core_sdk::println!("generate reward signature failed {:?} {} {}", to, seq, amt);
            None
        }
    }
}
