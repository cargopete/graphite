//! ERC20 Token Subgraph Example
//!
//! Demonstrates how to use the Graphite SDK to build a subgraph
//! that indexes ERC20 Transfer events.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

mod generated;

use generated::{ERC20TransferEvent, Transfer};
use graphite::prelude::*;

/// Handle ERC20 Transfer events.
///
/// This handler is called by graph-node for each Transfer event
/// emitted by the configured contract.
#[handler]
pub fn handle_transfer(event: ERC20TransferEvent) {
    // Create a new Transfer entity with a unique ID
    let mut transfer = Transfer::new(&event.id());

    // Map event fields to entity fields
    transfer.from = Bytes::from_slice(event.from.as_slice());
    transfer.to = Bytes::from_slice(event.to.as_slice());
    transfer.value = event.value.clone();
    transfer.block_number = event.block_number.clone();
    transfer.timestamp = event.block_timestamp.clone();
    transfer.transaction_hash = Bytes::from_slice(event.tx_hash.as_slice());
    // Note: token field would need to be set based on data source address

    // Persist to the store
    transfer.save(host);
}
