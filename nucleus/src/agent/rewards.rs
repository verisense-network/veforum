use ethabi::Token;
use vrs_core_sdk::tss::{CryptoType, tss_sign};

use vemodel::CommunityId;

use crate::eth_types::Address;
use crate::eth_types::bytes::Bytes;
use crate::eth_types::hash::keccak256;

pub fn generate_rewards(to: Address, seq: u64, amt: u128,  community: &CommunityId) -> Option<(Bytes, Bytes)> {
    let reward_data = ethabi::encode(&[Token::Uint(seq.into()), Token::Address(to), Token::Uint(amt.into())]);
    let prefix = format!("\x19Ethereum Signed Message:\n{}", reward_data.len());
    let mut prefixed_message = prefix.as_bytes().to_vec();
    prefixed_message.extend_from_slice(reward_data.as_slice());
    let message_hash = keccak256(&prefixed_message);
    match tss_sign(CryptoType::Secp256k1, community.to_be_bytes(), message_hash) {
        Ok(s) => {
            Some((Bytes::from(reward_data.to_vec()), Bytes::from(s)))
        }
        Err(_) => {
            vrs_core_sdk::println!("generate reward signature failed {:?} {} {}", to, seq, amt);
            None
        }
    }


}
