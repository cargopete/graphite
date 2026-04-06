//! ERC721 NFT Subgraph Example
//!
//! Demonstrates how to use the Graphite SDK to build a subgraph
//! that indexes ERC721 Transfer and Approval events.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;

mod generated;

use generated::{Approval, ERC721ApprovalEvent, ERC721TransferEvent, Token, Transfer};
use graphite::prelude::*;

/// Handle ERC721 Transfer events.
///
/// Tracks the current owner of each token and records every transfer.
/// The zero address is used by the ERC721 spec for mints (from = 0x0)
/// and burns (to = 0x0).
#[handler]
pub fn handle_transfer(event: ERC721TransferEvent) {
    let id = event.id();
    let mut transfer = Transfer::new(&id);

    transfer.from = Bytes::from_slice(event.from.as_slice());
    transfer.to = Bytes::from_slice(event.to.as_slice());
    transfer.token_id = event.token_id.clone();
    transfer.block_number = event.block_number.clone();
    transfer.timestamp = event.block_timestamp.clone();
    transfer.transaction_hash = Bytes::from_slice(event.tx_hash.as_slice());
    transfer.save(host);

    // Update the token's current owner
    let token_id_str = format!("{}", event.token_id);
    let mut token = Token::new(&token_id_str);
    token.owner = Bytes::from_slice(event.to.as_slice());
    // Clear any pending approval on transfer (ERC721 spec)
    token.approved = Bytes::from_slice(&[0u8; 20]);
    token.save(host);
}

/// Handle ERC721 Approval events.
///
/// Records the approved address for a specific token ID.
#[handler]
pub fn handle_approval(event: ERC721ApprovalEvent) {
    let id = event.id();
    let mut approval = Approval::new(&id);

    approval.owner = Bytes::from_slice(event.owner.as_slice());
    approval.approved = Bytes::from_slice(event.approved.as_slice());
    approval.token_id = event.token_id.clone();
    approval.block_number = event.block_number.clone();
    approval.transaction_hash = Bytes::from_slice(event.tx_hash.as_slice());
    approval.save(host);

    // Update the token's approved address
    let token_id_str = format!("{}", event.token_id);
    let mut token = Token::new(&token_id_str);
    token.owner = Bytes::from_slice(&[0u8; 20]); // Will be overwritten if token exists
    token.approved = Bytes::from_slice(event.approved.as_slice());
    token.save(host);
}
