//! ERC20 Token Subgraph — Phase 2: reads real EthereumEvent from AS memory.
//!
//! This handler is called by graph-node with `event_ptr` pointing at an
//! EthereumEvent object in WASM linear memory (AS ABI layout).  We use
//! `graph_as_runtime::ethereum::read_ethereum_event` to decode it, then
//! build and store a Transfer entity via the low-level AS ABI.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use graph_as_runtime::{
    as_types::{new_asc_big_int, new_asc_bytes, new_asc_string, new_typed_map, new_value_big_int, new_value_bytes, new_value_string},
    ethereum::{read_ethereum_event, EthereumValue},
    ffi::{log_log, store_set, LOG_INFO},
};

// ============================================================================
// Helpers
// ============================================================================

/// Format a raw byte slice as a lowercase hex string (no "0x" prefix).
fn hex_bytes(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

/// Find a named address param and return its 20-byte array.
/// Returns zeroes if not found or wrong type.
fn find_address(params: &[graph_as_runtime::ethereum::EventParam], name: &str) -> [u8; 20] {
    for p in params {
        if p.name == name {
            if let EthereumValue::Address(a) = &p.value {
                return *a;
            }
        }
    }
    [0u8; 20]
}

/// Find a named uint/int param and return the raw LE bytes (for BigInt).
fn find_bigint<'a>(params: &'a [graph_as_runtime::ethereum::EventParam], name: &str) -> &'a [u8] {
    for p in params {
        if p.name == name {
            match &p.value {
                EthereumValue::Uint(b) | EthereumValue::Int(b) => return b.as_slice(),
                _ => {}
            }
        }
    }
    &[0u8]
}

// ============================================================================
// Handler
// ============================================================================

/// Handle ERC20 Transfer events.
///
/// graph-node calls this with `event_ptr` = AscPtr<EthereumEvent>.
#[unsafe(no_mangle)]
pub extern "C" fn handle_transfer(event_ptr: i32) -> i32 {
    let raw = unsafe { read_ethereum_event(event_ptr as u32) };

    // Log that we received the event.
    let msg_ptr = new_asc_string("erc20: handle_transfer called");
    unsafe { log_log(LOG_INFO, msg_ptr); }

    // Decode event parameters.
    let from   = find_address(&raw.params, "from");
    let to     = find_address(&raw.params, "to");
    let value  = find_bigint(&raw.params, "value");

    // Build a unique entity ID: <tx_hash_hex>-<log_index_hex>
    let id = format!("{}-{}", hex_bytes(&raw.tx_hash), hex_bytes(&raw.log_index));
    let id_str = new_asc_string(&id);

    // Build entity fields.
    let from_bytes = new_asc_bytes(&from);
    let to_bytes   = new_asc_bytes(&to);
    let value_bi   = new_asc_big_int(value);
    let bn_bi      = new_asc_big_int(&raw.block_number);
    let ts_bi      = new_asc_big_int(&raw.block_timestamp);
    let txhash     = new_asc_bytes(&raw.tx_hash);

    let from_val   = new_value_bytes(from_bytes);
    let to_val     = new_value_bytes(to_bytes);
    let value_val  = new_value_big_int(value_bi);
    let bn_val     = new_value_big_int(bn_bi);
    let ts_val     = new_value_big_int(ts_bi);
    let txhash_val = new_value_bytes(txhash);
    let id_val     = new_value_string(id_str);

    let data_ptr = new_typed_map(&[
        ("id",              id_val),
        ("from",            from_val),
        ("to",              to_val),
        ("value",           value_val),
        ("blockNumber",     bn_val),
        ("timestamp",       ts_val),
        ("transactionHash", txhash_val),
        // token field omitted — set to empty string for now
        ("token",           new_value_string(new_asc_string(""))),
    ]);

    let entity_type = new_asc_string("Transfer");

    unsafe {
        store_set(entity_type, id_str, data_ptr);
        log_log(LOG_INFO, new_asc_string("erc20: Transfer entity saved"));
    }

    0
}
