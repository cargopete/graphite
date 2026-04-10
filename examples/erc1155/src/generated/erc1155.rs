//! Generated event bindings for ERC1155 contract.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// Generated from `TransferBatch` event.
pub struct ERC1155TransferBatchEvent {
    pub operator: [u8; 20],
    pub from: [u8; 20],
    pub to: [u8; 20],
    pub ids: Vec<Vec<u8>>,
    pub values: Vec<Vec<u8>>,
    pub block_number: Vec<u8>,
    pub block_timestamp: Vec<u8>,
    pub tx_hash: [u8; 32],
    pub log_index: Vec<u8>,
    pub address: [u8; 20],
}

impl graph_as_runtime::ethereum::FromRawEvent for ERC1155TransferBatchEvent {
    fn from_raw_event(raw: &graph_as_runtime::ethereum::RawEthereumEvent) -> Result<Self, &'static str> {
        let operator = raw.find_address("operator")?;
        let from = raw.find_address("from")?;
        let to = raw.find_address("to")?;
        let ids = raw.find_array("ids").map(|arr| arr.iter().filter_map(|v| v.as_uint()).collect::<alloc::vec::Vec<_>>())?;
        let values = raw.find_array("values").map(|arr| arr.iter().filter_map(|v| v.as_uint()).collect::<alloc::vec::Vec<_>>())?;

        Ok(Self {
            operator,
            from,
            to,
            ids,
            values,
            block_number: raw.block_number.clone(),
            block_timestamp: raw.block_timestamp.clone(),
            tx_hash: raw.tx_hash,
            log_index: raw.log_index.clone(),
            address: raw.address,
        })
    }
}


/// Generated from `TransferSingle` event.
pub struct ERC1155TransferSingleEvent {
    pub operator: [u8; 20],
    pub from: [u8; 20],
    pub to: [u8; 20],
    pub id: Vec<u8>,
    pub value: Vec<u8>,
    pub block_number: Vec<u8>,
    pub block_timestamp: Vec<u8>,
    pub tx_hash: [u8; 32],
    pub log_index: Vec<u8>,
    pub address: [u8; 20],
}

impl graph_as_runtime::ethereum::FromRawEvent for ERC1155TransferSingleEvent {
    fn from_raw_event(raw: &graph_as_runtime::ethereum::RawEthereumEvent) -> Result<Self, &'static str> {
        let operator = raw.find_address("operator")?;
        let from = raw.find_address("from")?;
        let to = raw.find_address("to")?;
        let id = raw.find_uint("id")?;
        let value = raw.find_uint("value")?;

        Ok(Self {
            operator,
            from,
            to,
            id,
            value,
            block_number: raw.block_number.clone(),
            block_timestamp: raw.block_timestamp.clone(),
            tx_hash: raw.tx_hash,
            log_index: raw.log_index.clone(),
            address: raw.address,
        })
    }
}


/// Generated from `URI` event.
pub struct ERC1155UriEvent {
    pub value: String,
    pub id: Vec<u8>,
    pub block_number: Vec<u8>,
    pub block_timestamp: Vec<u8>,
    pub tx_hash: [u8; 32],
    pub log_index: Vec<u8>,
    pub address: [u8; 20],
}

impl graph_as_runtime::ethereum::FromRawEvent for ERC1155UriEvent {
    fn from_raw_event(raw: &graph_as_runtime::ethereum::RawEthereumEvent) -> Result<Self, &'static str> {
        let value = raw.find_string("value")?;
        let id = raw.find_uint("id")?;

        Ok(Self {
            value,
            id,
            block_number: raw.block_number.clone(),
            block_timestamp: raw.block_timestamp.clone(),
            tx_hash: raw.tx_hash,
            log_index: raw.log_index.clone(),
            address: raw.address,
        })
    }
}


