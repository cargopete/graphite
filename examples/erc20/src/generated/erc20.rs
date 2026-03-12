//! Generated bindings for ERC20 contract.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

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

    /// The event topic (keccak256 of signature).
    pub const SELECTOR: [u8; 32] = [140, 91, 225, 229, 235, 236, 125, 91, 209, 79, 113, 66, 125, 30, 132, 243, 221, 3, 20, 192, 247, 178, 41, 30, 91, 32, 10, 200, 199, 195, 185, 37];
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

    /// The event topic (keccak256 of signature).
    pub const SELECTOR: [u8; 32] = [221, 242, 82, 173, 27, 226, 200, 155, 105, 194, 176, 104, 252, 55, 141, 170, 149, 43, 167, 241, 99, 196, 161, 22, 40, 245, 90, 77, 245, 35, 179, 239];
}

/// All events emitted by the ERC20 contract.
#[derive(Debug, Clone, PartialEq)]
pub enum ERC20Event {
    Approval(ERC20ApprovalEvent),
    Transfer(ERC20TransferEvent),
}
