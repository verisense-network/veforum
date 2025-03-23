

use rlp::{RlpStream};
use serde::{Deserialize, Serialize};
use crate::eth_types::{Address, U256, U64, H256};
use crate::eth_types::bytes::Bytes;
use crate::eth_types::ens::NameOrAddress;
use crate::eth_types::hash::keccak256;
use crate::eth_types::signature::Signature;
pub const NUM_TX_FIELDS: usize = 9;

pub(super) fn rlp_opt<T: rlp::Encodable>(rlp: &mut rlp::RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        rlp.append(&"");
    }
}

/// Parameters for sending a transaction
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TransactionRequest {
    /// Sender address or ENS name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,

    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<NameOrAddress>,

    /// Supplied gas (None for sensible default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,

    /// Gas price (None for sensible default)
    #[serde(rename = "gasPrice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,

    /// Transferred value (None for no transfer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,

    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    #[serde(skip_serializing_if = "Option::is_none", alias = "input")]
    pub data: Option<Bytes>,

    /// Transaction nonce (None for next available nonce)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,

    /// Chain ID (None for mainnet)
    #[serde(skip_serializing)]
    #[serde(default, rename = "chainId")]
    pub chain_id: Option<U64>,

}

impl TransactionRequest {
    /// Creates an empty transaction request with all fields left empty
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience function for sending a new payment transaction to the receiver.
    pub fn pay<T: Into<NameOrAddress>, V: Into<U256>>(to: T, value: V) -> Self {
        TransactionRequest { to: Some(to.into()), value: Some(value.into()), ..Default::default() }
    }

    // Builder pattern helpers

    /// Sets the `from` field in the transaction to the provided value
    #[must_use]
    pub fn from<T: Into<Address>>(mut self, from: T) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Sets the `to` field in the transaction to the provided value
    #[must_use]
    pub fn to<T: Into<NameOrAddress>>(mut self, to: T) -> Self {
        self.to = Some(to.into());
        self
    }

    /// Sets the `gas` field in the transaction to the provided value
    #[must_use]
    pub fn gas<T: Into<U256>>(mut self, gas: T) -> Self {
        self.gas = Some(gas.into());
        self
    }

    /// Sets the `gas_price` field in the transaction to the provided value
    #[must_use]
    pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
        self.gas_price = Some(gas_price.into());
        self
    }

    /// Sets the `value` field in the transaction to the provided value
    #[must_use]
    pub fn value<T: Into<U256>>(mut self, value: T) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the `data` field in the transaction to the provided value
    #[must_use]
    pub fn data<T: Into<Bytes>>(mut self, data: T) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Sets the `nonce` field in the transaction to the provided value
    #[must_use]
    pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
        self.nonce = Some(nonce.into());
        self
    }

    /// Sets the `chain_id` field in the transaction to the provided value
    #[must_use]
    pub fn chain_id<T: Into<U64>>(mut self, chain_id: T) -> Self {
        self.chain_id = Some(chain_id.into());
        self
    }

    /// Hashes the transaction's data with the provided chain id
    pub fn sighash(&self) -> H256 {
        match self.chain_id {
            Some(_) => keccak256(self.rlp().as_ref()).into(),
            None => keccak256(self.rlp_unsigned().as_ref()).into(),
        }
    }

    /// Gets the transaction's RLP encoding, prepared with the chain_id and extra fields for
    /// signing. Assumes the chainid exists.
    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        if let Some(chain_id) = self.chain_id {
            rlp.begin_list(NUM_TX_FIELDS);
            self.rlp_base(&mut rlp);
            rlp.append(&chain_id);
            rlp.append(&0u8);
            rlp.append(&0u8);
        } else {
            rlp.begin_list(NUM_TX_FIELDS - 3);
            self.rlp_base(&mut rlp);
        }
        rlp.out().freeze().into()
    }

    /// Gets the unsigned transaction's RLP encoding
    pub fn rlp_unsigned(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS - 3);
        self.rlp_base(&mut rlp);
        rlp.out().freeze().into()
    }

    /// Produces the RLP encoding of the transaction with the provided signature
    pub fn rlp_signed(&self, signature: &Signature) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS);

        self.rlp_base(&mut rlp);

        // append the signature
        rlp.append(&signature.v);
        rlp.append(&signature.r);
        rlp.append(&signature.s);
        rlp.out().freeze().into()
    }

    pub(crate) fn rlp_base(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.nonce);
        rlp_opt(rlp, &self.gas_price);
        rlp_opt(rlp, &self.gas);

        rlp_opt(rlp, &self.to.as_ref());
        rlp_opt(rlp, &self.value);
        rlp_opt(rlp, &self.data.as_ref().map(|d| d.as_ref()));
    }

}
