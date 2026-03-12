//! Generated bindings for ERC20 contract.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
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
        use graphite::decode::{TlvReader, value_tag};

        let mut reader = TlvReader::new(bytes);

        // Read field count
        let field_count = reader.read_u32()?;

        // Initialize with defaults
        let mut tx_hash = B256::default();
        let mut log_index = BigInt::zero();
        let mut block_number = BigInt::zero();
        let mut block_timestamp = BigInt::zero();
        let mut address = Address::ZERO;
        let mut owner = Address::ZERO;
        let mut spender = Address::ZERO;
        let mut value = BigInt::zero();

        // Read all fields
        for _ in 0..field_count {
            let key = reader.read_string()?;
            let tag = reader.read_u8()?;

            match key.as_str() {
                "__tx_hash" | "txHash" => {
                    if tag == value_tag::BYTES {
                        tx_hash = reader.read_b256()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "__log_index" | "logIndex" => {
                    if tag == value_tag::BIGINT {
                        log_index = reader.read_bigint()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "__block_number" | "blockNumber" => {
                    if tag == value_tag::BIGINT {
                        block_number = reader.read_bigint()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "__block_timestamp" | "blockTimestamp" => {
                    if tag == value_tag::BIGINT {
                        block_timestamp = reader.read_bigint()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "__address" | "address" => {
                    if tag == value_tag::ADDRESS {
                        address = reader.read_address()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "owner" => {
                    owner = reader.read_address()?;
                }
                "spender" => {
                    spender = reader.read_address()?;
                }
                "value" => {
                    value = reader.read_bigint()?;
                }
                _ => {
                    // Unknown field, skip it
                    reader.skip_value_data(tag)?;
                }
            }
        }

        Ok(Self {
            tx_hash,
            log_index,
            block_number,
            block_timestamp,
            address,
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
        use graphite::decode::{TlvReader, value_tag};

        let mut reader = TlvReader::new(bytes);

        // Read field count
        let field_count = reader.read_u32()?;

        // Initialize with defaults
        let mut tx_hash = B256::default();
        let mut log_index = BigInt::zero();
        let mut block_number = BigInt::zero();
        let mut block_timestamp = BigInt::zero();
        let mut address = Address::ZERO;
        let mut from = Address::ZERO;
        let mut to = Address::ZERO;
        let mut value = BigInt::zero();

        // Read all fields
        for _ in 0..field_count {
            let key = reader.read_string()?;
            let tag = reader.read_u8()?;

            match key.as_str() {
                "__tx_hash" | "txHash" => {
                    if tag == value_tag::BYTES {
                        tx_hash = reader.read_b256()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "__log_index" | "logIndex" => {
                    if tag == value_tag::BIGINT {
                        log_index = reader.read_bigint()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "__block_number" | "blockNumber" => {
                    if tag == value_tag::BIGINT {
                        block_number = reader.read_bigint()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "__block_timestamp" | "blockTimestamp" => {
                    if tag == value_tag::BIGINT {
                        block_timestamp = reader.read_bigint()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "__address" | "address" => {
                    if tag == value_tag::ADDRESS {
                        address = reader.read_address()?;
                    } else {
                        reader.skip_value_data(tag)?;
                    }
                }
                "from" => {
                    from = reader.read_address()?;
                }
                "to" => {
                    to = reader.read_address()?;
                }
                "value" => {
                    value = reader.read_bigint()?;
                }
                _ => {
                    // Unknown field, skip it
                    reader.skip_value_data(tag)?;
                }
            }
        }

        Ok(Self {
            tx_hash,
            log_index,
            block_number,
            block_timestamp,
            address,
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
