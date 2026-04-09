//! ERC20 Token Subgraph — Phase 3: uses generated typed event and entity structs.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use graph_as_runtime::ethereum::{read_ethereum_event, FromRawEvent};
use graph_as_runtime::as_types::new_asc_string;
use graph_as_runtime::ffi::{log_log, LOG_INFO};

mod generated;
use generated::{ERC20TransferEvent, Transfer};

/// Format a raw byte slice as a lowercase hex string (no "0x" prefix).
fn hex_bytes(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

/// Handle ERC20 Transfer events.
///
/// graph-node calls this with `event_ptr` = AscPtr<EthereumEvent>.
#[unsafe(no_mangle)]
pub extern "C" fn handle_transfer(event_ptr: i32) {
    let raw = unsafe { read_ethereum_event(event_ptr as u32) };

    let event = match ERC20TransferEvent::from_raw_event(&raw) {
        Ok(e) => e,
        Err(_) => return,
    };

    let msg_ptr = new_asc_string("erc20: handle_transfer called");
    unsafe { log_log(LOG_INFO, msg_ptr); }

    // Build a unique entity ID: <tx_hash_hex>-<log_index_hex>
    let id = format!("{}-{}", hex_bytes(&event.tx_hash), hex_bytes(&event.log_index));

    Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_value(event.value)
        .set_block_number(event.block_number)
        .set_timestamp(event.block_timestamp)
        .set_transaction_hash(event.tx_hash.to_vec())
        .save();

    unsafe {
        log_log(LOG_INFO, new_asc_string("erc20: Transfer entity saved"));
    }
}
