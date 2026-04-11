//! Generated event bindings for Pair template contract.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]

extern crate alloc;

use alloc::vec::Vec;

/// Generated from `Swap` event.
pub struct SwapEvent {
    pub sender: [u8; 20],
    pub amount0_in: Vec<u8>,
    pub amount1_in: Vec<u8>,
    pub amount0_out: Vec<u8>,
    pub amount1_out: Vec<u8>,
    pub to: [u8; 20],
    pub block_number: Vec<u8>,
    pub block_timestamp: Vec<u8>,
    pub tx_hash: [u8; 32],
    pub log_index: Vec<u8>,
    pub address: [u8; 20],
}

impl graph_as_runtime::ethereum::FromRawEvent for SwapEvent {
    fn from_raw_event(
        raw: &graph_as_runtime::ethereum::RawEthereumEvent,
    ) -> Result<Self, &'static str> {
        let sender = raw.find_address("sender")?;
        let amount0_in = raw.find_uint("amount0In")?;
        let amount1_in = raw.find_uint("amount1In")?;
        let amount0_out = raw.find_uint("amount0Out")?;
        let amount1_out = raw.find_uint("amount1Out")?;
        let to = raw.find_address("to")?;

        Ok(Self {
            sender,
            amount0_in,
            amount1_in,
            amount0_out,
            amount1_out,
            to,
            block_number: raw.block_number.clone(),
            block_timestamp: raw.block_timestamp.clone(),
            tx_hash: raw.tx_hash,
            log_index: raw.log_index.clone(),
            address: raw.address,
        })
    }
}
