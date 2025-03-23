use crate::eth_types::{H256, U256, U64};
use serde::{Deserialize, Serialize};
use crate::eth_types::Address;
use crate::eth_types::bytes::Bytes;
use crate::eth_types::ens::NameOrAddress;
use crate::eth_types::hash::keccak256;
use crate::eth_types::signature::Signature;
use crate::eth_types::transaction::TransactionRequest;
use crate::eth_types::typed_transaction::TypedTransaction::Legacy;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(not(feature = "legacy"), serde(tag = "type"))]
#[cfg_attr(feature = "legacy", serde(untagged))]
pub enum TypedTransaction {
    // 0x00
    #[serde(rename = "0x00", alias = "0x0")]
    Legacy(TransactionRequest),
}




impl TypedTransaction {
    pub fn from(&self) -> Option<&Address> {
        match self {
            Legacy(inner) => inner.from.as_ref(),
        }
    }

    pub fn set_from(&mut self, from: Address) -> &mut Self {
        match self {
            Legacy(inner) => inner.from = Some(from),
        };
        self
    }

    pub fn to(&self) -> Option<&NameOrAddress> {
        match self {
            Legacy(inner) => inner.to.as_ref(),

        }
    }

    pub fn to_addr(&self) -> Option<&Address> {
        self.to().and_then(|t| t.as_address())
    }

    pub fn set_to<T: Into<NameOrAddress>>(&mut self, to: T) -> &mut Self {
        let to = to.into();
        match self {
            Legacy(inner) => inner.to = Some(to),

        };
        self
    }

    pub fn nonce(&self) -> Option<&U256> {
        match self {
            Legacy(inner) => inner.nonce.as_ref(),
        }
    }

    pub fn set_nonce<T: Into<U256>>(&mut self, nonce: T) -> &mut Self {
        let nonce = nonce.into();
        match self {
            Legacy(inner) => inner.nonce = Some(nonce),

        };
        self
    }

    pub fn value(&self) -> Option<&U256> {
        match self {
            Legacy(inner) => inner.value.as_ref(),
        }
    }

    pub fn set_value<T: Into<U256>>(&mut self, value: T) -> &mut Self {
        let value = value.into();
        match self {
            Legacy(inner) => inner.value = Some(value),

        };
        self
    }

    pub fn gas(&self) -> Option<&U256> {
        match self {
            Legacy(inner) => inner.gas.as_ref(),
        }
    }

    pub fn gas_mut(&mut self) -> &mut Option<U256> {
        match self {
            Legacy(inner) => &mut inner.gas,
        }
    }

    pub fn set_gas<T: Into<U256>>(&mut self, gas: T) -> &mut Self {
        let gas = gas.into();
        match self {
            Legacy(inner) => inner.gas = Some(gas),
        };
        self
    }

    pub fn gas_price(&self) -> Option<U256> {
        match self {
            Legacy(inner) => inner.gas_price,

        }
    }

    pub fn set_gas_price<T: Into<U256>>(&mut self, gas_price: T) -> &mut Self {
        let gas_price = gas_price.into();
        match self {
            Legacy(inner) => inner.gas_price = Some(gas_price),

        };
        self
    }

    pub fn chain_id(&self) -> Option<U64> {
        match self {
            Legacy(inner) => inner.chain_id,

        }
    }

    pub fn set_chain_id<T: Into<U64>>(&mut self, chain_id: T) -> &mut Self {
        let chain_id = chain_id.into();
        match self {
            Legacy(inner) => inner.chain_id = Some(chain_id),

        };
        self
    }

    pub fn data(&self) -> Option<&Bytes> {
        match self {
            Legacy(inner) => inner.data.as_ref(),

        }
    }


    pub fn set_data(&mut self, data: Bytes) -> &mut Self {
        match self {
            Legacy(inner) => inner.data = Some(data),

        };
        self
    }

    pub fn rlp_signed(&self, signature: &Signature) -> Bytes {
        let mut encoded = vec![];
        match self {
            Legacy(ref tx) => {
                encoded.extend_from_slice(tx.rlp_signed(signature).as_ref());
            }

        };
        encoded.into()
    }

    pub fn rlp(&self) -> Bytes {
        let mut encoded = vec![];
        match self {
            Legacy(inner) => {
                encoded.extend_from_slice(inner.rlp().as_ref());
            }
        };

        encoded.into()
    }

    /// Hashes the transaction's data. Does not double-RLP encode
    pub fn sighash(&self) -> H256 {
        let encoded = self.rlp();
        keccak256(encoded).into()
    }

    /// Max cost of the transaction
    pub fn max_cost(&self) -> Option<U256> {
        let gas_limit = self.gas();
        let gas_price = self.gas_price();
        match (gas_limit, gas_price) {
            (Some(gas_limit), Some(gas_price)) => Some(gas_limit * gas_price),
            _ => None,
        }
    }

    /// Hashes the transaction's data with the included signature.
    pub fn hash(&self, signature: &Signature) -> H256 {
        keccak256(self.rlp_signed(signature).as_ref()).into()
    }
}


impl From<TransactionRequest> for TypedTransaction {
    fn from(src: TransactionRequest) -> TypedTransaction {
        TypedTransaction::Legacy(src)
    }
}

impl TypedTransaction {
    pub fn as_legacy_ref(&self) -> Option<&TransactionRequest> {
        match self {
            Legacy(tx) => Some(tx),
        }
    }
    pub fn as_legacy_mut(&mut self) -> Option<&mut TransactionRequest> {
        match self {
            Legacy(tx) => Some(tx),
        }
    }
}


impl TypedTransaction {
    fn into_legacy(self) -> TransactionRequest {
        match self {
            Legacy(tx) => tx,
        }
    }
}

impl From<TypedTransaction> for TransactionRequest {
    fn from(src: TypedTransaction) -> TransactionRequest {
        src.into_legacy()
    }
}
