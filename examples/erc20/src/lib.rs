//! ERC20 Token Subgraph — Phase 3: uses generated typed event and entity structs.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use graph_as_runtime::ethereum::{FromRawEvent, RawEthereumEvent};

#[cfg(target_arch = "wasm32")]
use graph_as_runtime::ethereum::read_ethereum_event;

#[allow(unused_imports)]
use graph_as_runtime::ffi::LOG_INFO;

mod generated;
use generated::{ERC20TransferEvent, Transfer};

/// Format a raw byte slice as a lowercase hex string (no "0x" prefix).
fn hex_bytes(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

// ============================================================================
// Core handler logic — runs on both WASM and native
// ============================================================================

/// Process a decoded Transfer event and write to the store.
///
/// This is the testable core. On WASM it is called by `handle_transfer`.
/// In tests it is called directly.
pub fn handle_transfer_impl(raw: &RawEthereumEvent) {
    let event = match ERC20TransferEvent::from_raw_event(raw) {
        Ok(e) => e,
        Err(_) => return,
    };

    // Build a unique entity ID: <tx_hash_hex>-<log_index_hex>
    let id = format!(
        "{}-{}",
        hex_bytes(&event.tx_hash),
        hex_bytes(&event.log_index),
    );

    Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_value(event.value)
        .set_block_number(event.block_number)
        .set_timestamp(event.block_timestamp)
        .set_transaction_hash(event.tx_hash.to_vec())
        .save();
}

// ============================================================================
// WASM entry point — only compiled for wasm32
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn handle_transfer(event_ptr: i32) {
    use graph_as_runtime::as_types::new_asc_string;
    use graph_as_runtime::ffi::log_log;

    let raw = unsafe { read_ethereum_event(event_ptr as u32) };

    unsafe {
        log_log(LOG_INFO, new_asc_string("erc20: handle_transfer called"));
    }

    handle_transfer_impl(&raw);

    unsafe {
        log_log(LOG_INFO, new_asc_string("erc20: Transfer entity saved"));
    }
}

// ============================================================================
// Tests — native only
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use graph_as_runtime::ethereum::{EthereumValue, EventParam};
    use graphite::mock;

    fn mock_transfer_event() -> RawEthereumEvent {
        RawEthereumEvent {
            address: [0x00; 20],
            log_index: vec![0],
            block_number: vec![1, 0, 0, 0], // block 1
            block_timestamp: vec![100, 0, 0, 0],
            tx_hash: [0xab; 32],
            params: alloc::vec![
                EventParam {
                    name: "from".into(),
                    value: EthereumValue::Address([0xaa; 20]),
                },
                EventParam {
                    name: "to".into(),
                    value: EthereumValue::Address([0xbb; 20]),
                },
                EventParam {
                    name: "value".into(),
                    value: EthereumValue::Uint(alloc::vec![100, 0, 0, 0, 0, 0, 0, 0]),
                },
            ],
        }
    }

    #[test]
    fn transfer_creates_entity() {
        mock::reset();

        let event = mock_transfer_event();
        handle_transfer_impl(&event);

        // tx_hash is 0xab * 32, log_index is 0
        let tx_hex = "ab".repeat(32);
        let id = format!("{}-00", tx_hex);

        assert!(
            mock::has_entity("Transfer", &id),
            "Transfer entity should exist"
        );
        mock::assert_entity("Transfer", &id)
            .field_bytes("from", &[0xaa; 20])
            .field_bytes("to", &[0xbb; 20])
            .field_exists("value")
            .field_exists("blockNumber")
            .field_exists("timestamp");
    }

    #[test]
    fn transfer_entity_count() {
        mock::reset();

        handle_transfer_impl(&mock_transfer_event());
        assert_eq!(mock::entity_count("Transfer"), 1);

        // Same event again — same id, upsert, still 1.
        handle_transfer_impl(&mock_transfer_event());
        assert_eq!(mock::entity_count("Transfer"), 1);

        // Different tx_hash → new entity.
        let mut event2 = mock_transfer_event();
        event2.tx_hash = [0xcc; 32];
        handle_transfer_impl(&event2);
        assert_eq!(mock::entity_count("Transfer"), 2);
    }
}
