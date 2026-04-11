//! Generated event bindings for ERC721 contract.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// Generated from `Transfer` event.
pub struct ERC721TransferEvent {
    pub from: [u8; 20],
    pub to: [u8; 20],
    pub token_id: Vec<u8>,
    pub block_number: Vec<u8>,
    pub block_timestamp: Vec<u8>,
    pub tx_hash: [u8; 32],
    pub log_index: Vec<u8>,
    pub address: [u8; 20],
}

impl graph_as_runtime::ethereum::FromRawEvent for ERC721TransferEvent {
    fn from_raw_event(raw: &graph_as_runtime::ethereum::RawEthereumEvent) -> Result<Self, &'static str> {
        let from = raw.find_address("from")?;
        let to = raw.find_address("to")?;
        let token_id = raw.find_uint("tokenId")?;

        Ok(Self {
            from,
            to,
            token_id,
            block_number: raw.block_number.clone(),
            block_timestamp: raw.block_timestamp.clone(),
            tx_hash: raw.tx_hash,
            log_index: raw.log_index.clone(),
            address: raw.address,
        })
    }
}


