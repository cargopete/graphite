//! Generated bindings for ERC721 contract.
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
pub struct ERC721ApprovalEvent {
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
    pub approved: Address,
    /// `uint256 (indexed)`
    pub token_id: BigInt,
}

impl ERC721ApprovalEvent {
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

impl EventDecode for ERC721ApprovalEvent {
    const SELECTOR: [u8; 32] = [140, 91, 225, 229, 235, 236, 125, 91, 209, 79, 113, 66, 125, 30, 132, 243, 221, 3, 20, 192, 247, 178, 41, 30, 91, 32, 10, 200, 199, 195, 185, 37];

    fn decode(topics: &[B256], _data: &[u8]) -> Result<Self, DecodeError> {
        // Verify selector
        if topics.is_empty() || topics[0].0 != Self::SELECTOR {
            return Err(DecodeError::SelectorMismatch {
                expected: Self::SELECTOR,
                got: topics.first().map(|t| t.0).unwrap_or([0; 32]),
            });
        }

        // Verify topic count
        if topics.len() < 4 {
            return Err(DecodeError::NotEnoughTopics {
                expected: 4,
                got: topics.len(),
            });
        }

        let owner = graphite::decode::decode_address_from_topic(&topics[1]);
        let approved = graphite::decode::decode_address_from_topic(&topics[2]);
        let token_id = graphite::decode::decode_uint256_from_topic(&topics[3]);

        Ok(Self {
            tx_hash: B256::default(),
            log_index: BigInt::zero(),
            block_number: BigInt::zero(),
            block_timestamp: BigInt::zero(),
            address: Address::ZERO,
            owner,
            approved,
            token_id,
        })
    }
}
impl FromWasmBytes for ERC721ApprovalEvent {
    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        // Parse as RawLog first (graph-node sends RustLogTrigger format)
        let raw_log = RawLog::from_wasm_bytes(bytes)?;
        Self::from_raw_log(&raw_log)
    }
}


/// Event: `ApprovalForAll(address,address,bool)`
#[derive(Debug, Clone, PartialEq)]
pub struct ERC721ApprovalForAllEvent {
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
    pub operator: Address,
    /// `bool`
    pub approved: bool,
}

impl ERC721ApprovalForAllEvent {
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

impl EventDecode for ERC721ApprovalForAllEvent {
    const SELECTOR: [u8; 32] = [23, 48, 126, 171, 57, 171, 97, 7, 232, 137, 152, 69, 173, 61, 89, 189, 150, 83, 242, 0, 242, 32, 146, 4, 137, 202, 43, 89, 55, 105, 108, 49];

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
        let operator = graphite::decode::decode_address_from_topic(&topics[2]);
        let approved = graphite::decode::decode_bool(data, 0)?;

        Ok(Self {
            tx_hash: B256::default(),
            log_index: BigInt::zero(),
            block_number: BigInt::zero(),
            block_timestamp: BigInt::zero(),
            address: Address::ZERO,
            owner,
            operator,
            approved,
        })
    }
}
impl FromWasmBytes for ERC721ApprovalForAllEvent {
    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        // Parse as RawLog first (graph-node sends RustLogTrigger format)
        let raw_log = RawLog::from_wasm_bytes(bytes)?;
        Self::from_raw_log(&raw_log)
    }
}


/// Event: `Transfer(address,address,uint256)`
#[derive(Debug, Clone, PartialEq)]
pub struct ERC721TransferEvent {
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
    /// `uint256 (indexed)`
    pub token_id: BigInt,
}

impl ERC721TransferEvent {
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

impl EventDecode for ERC721TransferEvent {
    const SELECTOR: [u8; 32] = [221, 242, 82, 173, 27, 226, 200, 155, 105, 194, 176, 104, 252, 55, 141, 170, 149, 43, 167, 241, 99, 196, 161, 22, 40, 245, 90, 77, 245, 35, 179, 239];

    fn decode(topics: &[B256], _data: &[u8]) -> Result<Self, DecodeError> {
        // Verify selector
        if topics.is_empty() || topics[0].0 != Self::SELECTOR {
            return Err(DecodeError::SelectorMismatch {
                expected: Self::SELECTOR,
                got: topics.first().map(|t| t.0).unwrap_or([0; 32]),
            });
        }

        // Verify topic count
        if topics.len() < 4 {
            return Err(DecodeError::NotEnoughTopics {
                expected: 4,
                got: topics.len(),
            });
        }

        let from = graphite::decode::decode_address_from_topic(&topics[1]);
        let to = graphite::decode::decode_address_from_topic(&topics[2]);
        let token_id = graphite::decode::decode_uint256_from_topic(&topics[3]);

        Ok(Self {
            tx_hash: B256::default(),
            log_index: BigInt::zero(),
            block_number: BigInt::zero(),
            block_timestamp: BigInt::zero(),
            address: Address::ZERO,
            from,
            to,
            token_id,
        })
    }
}
impl FromWasmBytes for ERC721TransferEvent {
    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        // Parse as RawLog first (graph-node sends RustLogTrigger format)
        let raw_log = RawLog::from_wasm_bytes(bytes)?;
        Self::from_raw_log(&raw_log)
    }
}


/// All events emitted by the ERC721 contract.
#[derive(Debug, Clone, PartialEq)]
pub enum ERC721Event {
    Approval(ERC721ApprovalEvent),
    ApprovalForAll(ERC721ApprovalForAllEvent),
    Transfer(ERC721TransferEvent),
}
