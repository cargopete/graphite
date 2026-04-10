//! Multi-source subgraph — tracks ERC20 and ERC721 Transfer events in a single
//! WASM module with two data sources defined in subgraph.yaml.
//!
//! This example validates that the deploy tool correctly handles manifests with
//! multiple dataSources sharing one WASM file.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use graph_as_runtime::ethereum::{FromRawEvent, RawEthereumEvent};

#[cfg(target_arch = "wasm32")]
use graph_as_runtime::ethereum::read_ethereum_event;

mod generated;
use generated::{Erc20Transfer, Erc721Transfer, ERC20TransferEvent, ERC721TransferEvent};

/// Format bytes as lowercase hex (no 0x prefix).
fn hex_bytes(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

// ============================================================================
// ERC20 handler
// ============================================================================

pub fn handle_erc20_transfer_impl(raw: &RawEthereumEvent) {
    let event = match ERC20TransferEvent::from_raw_event(raw) {
        Ok(e) => e,
        Err(_) => return,
    };

    let id = format!("{}-{}", hex_bytes(&event.tx_hash), hex_bytes(&event.log_index));

    Erc20Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_value(event.value)
        .set_block_number(event.block_number)
        .set_timestamp(event.block_timestamp)
        .set_transaction_hash(event.tx_hash.to_vec())
        .save();
}

// ============================================================================
// ERC721 handler
// ============================================================================

pub fn handle_erc721_transfer_impl(raw: &RawEthereumEvent) {
    let event = match ERC721TransferEvent::from_raw_event(raw) {
        Ok(e) => e,
        Err(_) => return,
    };

    let id = format!("{}-{}", hex_bytes(&event.tx_hash), hex_bytes(&event.log_index));

    Erc721Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_token_id(event.token_id)
        .set_block_number(event.block_number)
        .set_timestamp(event.block_timestamp)
        .set_transaction_hash(event.tx_hash.to_vec())
        .save();
}

// ============================================================================
// WASM entry points
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn handle_erc20_transfer(event_ptr: i32) {
    let raw = unsafe { read_ethereum_event(event_ptr as u32) };
    handle_erc20_transfer_impl(&raw);
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn handle_erc721_transfer(event_ptr: i32) {
    let raw = unsafe { read_ethereum_event(event_ptr as u32) };
    handle_erc721_transfer_impl(&raw);
}

// ============================================================================
// Tests — native only
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use graph_as_runtime::ethereum::{EthereumValue, EventParam};
    use graphite::mock;

    fn erc20_event(tx: u8, log: u8) -> RawEthereumEvent {
        RawEthereumEvent {
            address: [0x11; 20],
            log_index: alloc::vec![log],
            block_number: alloc::vec![10, 0, 0, 0],
            block_timestamp: alloc::vec![100, 0, 0, 0],
            tx_hash: [tx; 32],
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
                    value: EthereumValue::Uint(alloc::vec![200, 0, 0, 0, 0, 0, 0, 0]),
                },
            ],
            receipt: None,
        }
    }

    fn erc721_event(tx: u8, log: u8) -> RawEthereumEvent {
        RawEthereumEvent {
            address: [0x22; 20],
            log_index: alloc::vec![log],
            block_number: alloc::vec![20, 0, 0, 0],
            block_timestamp: alloc::vec![200, 0, 0, 0],
            tx_hash: [tx; 32],
            params: alloc::vec![
                EventParam {
                    name: "from".into(),
                    value: EthereumValue::Address([0xcc; 20]),
                },
                EventParam {
                    name: "to".into(),
                    value: EthereumValue::Address([0xdd; 20]),
                },
                EventParam {
                    name: "tokenId".into(),
                    value: EthereumValue::Uint(alloc::vec![42, 0, 0, 0, 0, 0, 0, 0]),
                },
            ],
            receipt: None,
        }
    }

    #[test]
    fn erc20_transfer_writes_entity() {
        mock::reset();
        handle_erc20_transfer_impl(&erc20_event(0xab, 0));

        let tx = "ab".repeat(32);
        let id = format!("{}-00", tx);

        assert!(mock::has_entity("Erc20Transfer", &id));
        mock::assert_entity("Erc20Transfer", &id)
            .field_bytes("from", &[0xaa; 20])
            .field_bytes("to", &[0xbb; 20])
            .field_exists("value")
            .field_exists("blockNumber");
    }

    #[test]
    fn erc721_transfer_writes_entity() {
        mock::reset();
        handle_erc721_transfer_impl(&erc721_event(0xcd, 1));

        let tx = "cd".repeat(32);
        let id = format!("{}-01", tx);

        assert!(mock::has_entity("Erc721Transfer", &id));
        mock::assert_entity("Erc721Transfer", &id)
            .field_bytes("from", &[0xcc; 20])
            .field_bytes("to", &[0xdd; 20])
            .field_exists("tokenId")
            .field_exists("blockNumber");
    }

    #[test]
    fn both_handlers_independent_namespaces() {
        mock::reset();

        // Same tx hash and log index — different entity types, should coexist.
        handle_erc20_transfer_impl(&erc20_event(0xff, 0));
        handle_erc721_transfer_impl(&erc721_event(0xff, 0));

        assert_eq!(mock::entity_count("Erc20Transfer"), 1);
        assert_eq!(mock::entity_count("Erc721Transfer"), 1);
    }

    #[test]
    fn multiple_erc20_and_erc721_events() {
        mock::reset();

        handle_erc20_transfer_impl(&erc20_event(0x01, 0));
        handle_erc20_transfer_impl(&erc20_event(0x02, 0));
        handle_erc721_transfer_impl(&erc721_event(0x03, 0));

        assert_eq!(mock::entity_count("Erc20Transfer"), 2);
        assert_eq!(mock::entity_count("Erc721Transfer"), 1);
    }
}
