//! Generated event bindings for Factory contract.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]

extern crate alloc;

use alloc::vec::Vec;

/// Generated from `PairCreated` event.
pub struct PairCreatedEvent {
    pub token0: [u8; 20],
    pub token1: [u8; 20],
    pub pair: [u8; 20],
    pub block_number: Vec<u8>,
    pub block_timestamp: Vec<u8>,
    pub tx_hash: [u8; 32],
    pub log_index: Vec<u8>,
    pub address: [u8; 20],
}

impl graph_as_runtime::ethereum::FromRawEvent for PairCreatedEvent {
    fn from_raw_event(
        raw: &graph_as_runtime::ethereum::RawEthereumEvent,
    ) -> Result<Self, &'static str> {
        let token0 = raw.find_address("token0")?;
        let token1 = raw.find_address("token1")?;
        let pair = raw.find_address("pair")?;

        Ok(Self {
            token0,
            token1,
            pair,
            block_number: raw.block_number.clone(),
            block_timestamp: raw.block_timestamp.clone(),
            tx_hash: raw.tx_hash,
            log_index: raw.log_index.clone(),
            address: raw.address,
        })
    }
}
