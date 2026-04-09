//! Generated bindings for ERC20 contract.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use graphite::prelude::*;

/// Event: `Approval(address,address,uint256)`
#[derive(Debug, Clone, PartialEq)]
pub struct ERC20ApprovalEvent {
    /// Transaction hash
    pub tx_hash: B256,
    /// Log index within the transaction
    pub log_index: BigInt,
    /// Block number
    pub block_number: BigInt,
    /// Block timestamp
    pub block_timestamp: BigInt,
    /// Contract address that emitted the event
    pub address: Address,
    /// `address (indexed)`
    pub owner: Address,
    /// `address (indexed)`
    pub spender: Address,
    /// `uint256`
    pub value: BigInt,
}

impl ERC20ApprovalEvent {
    /// Generate a unique ID for this event.
    pub fn id(&self) -> String {
        format!("{:?}-{}", self.tx_hash, self.log_index)
    }

    /// Decode from a raw log.
    pub fn from_raw_log(log: &RawLog) -> Result<Self, DecodeError> {
        <Self as EventDecode>::decode(&log.topics, log.data.as_slice())
            .map(|mut e| {
                e.tx_hash = log.tx_hash;
                e.log_index = BigInt::from(log.log_index);
                e.block_number = BigInt::from(log.block_number);
                e.block_timestamp = BigInt::from(log.block_timestamp);
                e.address = log.address;
                e
            })
    }
}

impl EventDecode for ERC20ApprovalEvent {
    const SELECTOR: [u8; 32] = [140, 91, 225, 229, 235, 236, 125, 91, 209, 79, 113, 66, 125, 30, 132, 243, 221, 3, 20, 192, 247, 178, 41, 30, 91, 32, 10, 200, 199, 195, 185, 37];

    fn decode(topics: &[B256], data: &[u8]) -> Result<Self, DecodeError> {
        // Verify selector
        if topics.is_empty() || topics[0].0 != Self::SELECTOR {
            return Err(DecodeError::SelectorMismatch {
                expected: Self::SELECTOR,
                got: topics.first().map(|t| t.0).unwrap_or([0; 32]),
            });
        }

        // Verify topic count
        if topics.len() < 3 {
            return Err(DecodeError::NotEnoughTopics {
                expected: 3,
                got: topics.len(),
            });
        }

        let owner = graphite::decode::decode_address_from_topic(&topics[1]);
        let spender = graphite::decode::decode_address_from_topic(&topics[2]);
        let value = graphite::decode::decode_uint256(data, 0)?;

        Ok(Self {
            tx_hash: B256::default(),
            log_index: BigInt::zero(),
            block_number: BigInt::zero(),
            block_timestamp: BigInt::zero(),
            address: Address::ZERO,
            owner,
            spender,
            value,
        })
    }
}
impl FromWasmBytes for ERC20ApprovalEvent {
    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        // Parse as RawLog first (graph-node sends RustLogTrigger format)
        let raw_log = RawLog::from_wasm_bytes(bytes)?;
        Self::from_raw_log(&raw_log)
    }
}

impl graph_as_runtime::ethereum::FromRawEvent for ERC20ApprovalEvent {
    fn from_raw_event(raw: &graph_as_runtime::ethereum::RawEthereumEvent) -> Result<Self, &'static str> {
        let owner = {
            let _p = raw.params.iter().find(|p| p.name == "owner").ok_or("missing param: owner")?;
            match &_p.value {
                graph_as_runtime::ethereum::EthereumValue::Address(a) => Address::from(*a),
                _ => return Err("wrong type for owner"),
            }
        };
        let spender = {
            let _p = raw.params.iter().find(|p| p.name == "spender").ok_or("missing param: spender")?;
            match &_p.value {
                graph_as_runtime::ethereum::EthereumValue::Address(a) => Address::from(*a),
                _ => return Err("wrong type for spender"),
            }
        };
        let value = {
            let _p = raw.params.iter().find(|p| p.name == "value").ok_or("missing param: value")?;
            match &_p.value {
                graph_as_runtime::ethereum::EthereumValue::Uint(b) =>
                    graphite::primitives::BigInt::from_signed_bytes_le(b),
                graph_as_runtime::ethereum::EthereumValue::Int(b) =>
                    graphite::primitives::BigInt::from_signed_bytes_le(b),
                _ => return Err("wrong type for value"),
            }
        };

        Ok(Self {
            tx_hash:         B256(raw.tx_hash),
            log_index:       graphite::primitives::BigInt::from_signed_bytes_le(&raw.log_index),
            block_number:    graphite::primitives::BigInt::from_signed_bytes_le(&raw.block_number),
            block_timestamp: graphite::primitives::BigInt::from_signed_bytes_le(&raw.block_timestamp),
            address:         Address::from(raw.address),
            owner,
            spender,
            value,
        })
    }
}


/// Event: `Transfer(address,address,uint256)`
#[derive(Debug, Clone, PartialEq)]
pub struct ERC20TransferEvent {
    /// Transaction hash
    pub tx_hash: B256,
    /// Log index within the transaction
    pub log_index: BigInt,
    /// Block number
    pub block_number: BigInt,
    /// Block timestamp
    pub block_timestamp: BigInt,
    /// Contract address that emitted the event
    pub address: Address,
    /// `address (indexed)`
    pub from: Address,
    /// `address (indexed)`
    pub to: Address,
    /// `uint256`
    pub value: BigInt,
}

impl ERC20TransferEvent {
    /// Generate a unique ID for this event.
    pub fn id(&self) -> String {
        format!("{:?}-{}", self.tx_hash, self.log_index)
    }

    /// Decode from a raw log.
    pub fn from_raw_log(log: &RawLog) -> Result<Self, DecodeError> {
        <Self as EventDecode>::decode(&log.topics, log.data.as_slice())
            .map(|mut e| {
                e.tx_hash = log.tx_hash;
                e.log_index = BigInt::from(log.log_index);
                e.block_number = BigInt::from(log.block_number);
                e.block_timestamp = BigInt::from(log.block_timestamp);
                e.address = log.address;
                e
            })
    }
}

impl EventDecode for ERC20TransferEvent {
    const SELECTOR: [u8; 32] = [221, 242, 82, 173, 27, 226, 200, 155, 105, 194, 176, 104, 252, 55, 141, 170, 149, 43, 167, 241, 99, 196, 161, 22, 40, 245, 90, 77, 245, 35, 179, 239];

    fn decode(topics: &[B256], data: &[u8]) -> Result<Self, DecodeError> {
        // Verify selector
        if topics.is_empty() || topics[0].0 != Self::SELECTOR {
            return Err(DecodeError::SelectorMismatch {
                expected: Self::SELECTOR,
                got: topics.first().map(|t| t.0).unwrap_or([0; 32]),
            });
        }

        // Verify topic count
        if topics.len() < 3 {
            return Err(DecodeError::NotEnoughTopics {
                expected: 3,
                got: topics.len(),
            });
        }

        let from = graphite::decode::decode_address_from_topic(&topics[1]);
        let to = graphite::decode::decode_address_from_topic(&topics[2]);
        let value = graphite::decode::decode_uint256(data, 0)?;

        Ok(Self {
            tx_hash: B256::default(),
            log_index: BigInt::zero(),
            block_number: BigInt::zero(),
            block_timestamp: BigInt::zero(),
            address: Address::ZERO,
            from,
            to,
            value,
        })
    }
}
impl FromWasmBytes for ERC20TransferEvent {
    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        // Parse as RawLog first (graph-node sends RustLogTrigger format)
        let raw_log = RawLog::from_wasm_bytes(bytes)?;
        Self::from_raw_log(&raw_log)
    }
}

impl graph_as_runtime::ethereum::FromRawEvent for ERC20TransferEvent {
    fn from_raw_event(raw: &graph_as_runtime::ethereum::RawEthereumEvent) -> Result<Self, &'static str> {
        let from = {
            let _p = raw.params.iter().find(|p| p.name == "from").ok_or("missing param: from")?;
            match &_p.value {
                graph_as_runtime::ethereum::EthereumValue::Address(a) => Address::from(*a),
                _ => return Err("wrong type for from"),
            }
        };
        let to = {
            let _p = raw.params.iter().find(|p| p.name == "to").ok_or("missing param: to")?;
            match &_p.value {
                graph_as_runtime::ethereum::EthereumValue::Address(a) => Address::from(*a),
                _ => return Err("wrong type for to"),
            }
        };
        let value = {
            let _p = raw.params.iter().find(|p| p.name == "value").ok_or("missing param: value")?;
            match &_p.value {
                graph_as_runtime::ethereum::EthereumValue::Uint(b) =>
                    graphite::primitives::BigInt::from_signed_bytes_le(b),
                graph_as_runtime::ethereum::EthereumValue::Int(b) =>
                    graphite::primitives::BigInt::from_signed_bytes_le(b),
                _ => return Err("wrong type for value"),
            }
        };

        Ok(Self {
            tx_hash:         B256(raw.tx_hash),
            log_index:       graphite::primitives::BigInt::from_signed_bytes_le(&raw.log_index),
            block_number:    graphite::primitives::BigInt::from_signed_bytes_le(&raw.block_number),
            block_timestamp: graphite::primitives::BigInt::from_signed_bytes_le(&raw.block_timestamp),
            address:         Address::from(raw.address),
            from,
            to,
            value,
        })
    }
}


/// All events emitted by the ERC20 contract.
#[derive(Debug, Clone, PartialEq)]
pub enum ERC20Event {
    Approval(ERC20ApprovalEvent),
    Transfer(ERC20TransferEvent),
}
