//! Milestone 1: minimal block handler that writes a Ping entity to the store.
//!
//! Uses graph-as-runtime low-level API directly — no graphite SDK layer.
//! This is the simplest possible test of the AS-ABI approach.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

#[cfg(target_arch = "wasm32")]
use graph_as_runtime::as_types::{new_asc_string, new_typed_map, new_value_string};
#[cfg(target_arch = "wasm32")]
use graph_as_runtime::ffi::{LOG_INFO, log_log, store_set};

/// Block handler — called by graph-node for each indexed block.
///
/// `_block: u32` is an AscPtr<EthereumBlock> which we ignore.
/// The handler writes a single Ping entity to the store.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn handle_block(_block: u32) {
    // Log that we're running.
    let msg_ptr = new_asc_string("milestone: handle_block called");
    unsafe {
        log_log(LOG_INFO, msg_ptr);
    }

    // Build the entity: Ping { id: "milestone-1", message: "hello from rust" }
    let id_str = new_asc_string("milestone-1");
    let msg_str = new_asc_string("hello from rust");

    let id_val = new_value_string(id_str);
    let msg_val = new_value_string(msg_str);

    // Build the TypedMap with the entity fields.
    let data_ptr = new_typed_map(&[("id", id_val), ("message", msg_val)]);

    // Entity type name and ID as AS strings.
    let entity_type = new_asc_string("Ping");

    unsafe {
        store_set(entity_type, id_str, data_ptr);
    }

    // Log success.
    let done_ptr = new_asc_string("milestone: Ping entity saved");
    unsafe {
        log_log(LOG_INFO, done_ptr);
    }
}
