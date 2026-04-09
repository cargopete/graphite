//! ERC721 NFT Subgraph — Phase 3: uses generated typed event and entity structs.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use graph_as_runtime::ethereum::{read_ethereum_event, FromRawEvent};
#[cfg(target_arch = "wasm32")]
use graph_as_runtime::as_types::new_asc_string;
#[cfg(target_arch = "wasm32")]
use graph_as_runtime::ffi::{log_log, LOG_INFO};

mod generated;
use generated::{ERC721TransferEvent, ERC721ApprovalEvent, Token, Transfer, Approval};

/// Format a raw byte slice as a lowercase hex string (no "0x" prefix).
fn hex_bytes(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

/// Handle ERC721 Transfer events.
///
/// Tracks the current owner of each token and records every transfer.
#[unsafe(no_mangle)]
pub extern "C" fn handle_transfer(event_ptr: i32) {
    let raw = unsafe { read_ethereum_event(event_ptr as u32) };

    let event = match ERC721TransferEvent::from_raw_event(&raw) {
        Ok(e) => e,
        Err(_) => return,
    };

    #[cfg(target_arch = "wasm32")]
    unsafe { log_log(LOG_INFO, new_asc_string("erc721: handle_transfer called")); }

    // Unique transfer ID
    let id = format!("{}-{}", hex_bytes(&event.tx_hash), hex_bytes(&event.log_index));

    Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_token_id(event.token_id.clone())
        .set_block_number(event.block_number.clone())
        .set_timestamp(event.block_timestamp.clone())
        .set_transaction_hash(event.tx_hash.to_vec())
        .save();

    // Update the token's current owner (use token_id hex as entity ID)
    let token_id_str = hex_bytes(&event.token_id);
    Token::new(&token_id_str)
        .set_owner(event.to.to_vec())
        .set_approved(alloc::vec![0u8; 20]) // clear approval on transfer
        .save();

    #[cfg(target_arch = "wasm32")]
    unsafe { log_log(LOG_INFO, new_asc_string("erc721: Transfer entity saved")); }
}

/// Handle ERC721 Approval events.
#[unsafe(no_mangle)]
pub extern "C" fn handle_approval(event_ptr: i32) {
    let raw = unsafe { read_ethereum_event(event_ptr as u32) };

    let event = match ERC721ApprovalEvent::from_raw_event(&raw) {
        Ok(e) => e,
        Err(_) => return,
    };

    let id = format!("{}-{}", hex_bytes(&event.tx_hash), hex_bytes(&event.log_index));

    Approval::new(&id)
        .set_owner(event.owner.to_vec())
        .set_approved(event.approved.to_vec())
        .set_token_id(event.token_id.clone())
        .set_block_number(event.block_number.clone())
        .set_transaction_hash(event.tx_hash.to_vec())
        .save();

    // Update the token's approved address
    let token_id_str = hex_bytes(&event.token_id);
    Token::new(&token_id_str)
        .set_approved(event.approved.to_vec())
        .save();
}
