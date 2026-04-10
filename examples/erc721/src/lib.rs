//! ERC721 NFT Subgraph — Phase 3: uses generated typed event and entity structs.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use graph_as_runtime::ethereum::{FromRawEvent, RawEthereumEvent};

#[cfg(target_arch = "wasm32")]
use graph_as_runtime::as_types::new_asc_string;
#[cfg(target_arch = "wasm32")]
use graph_as_runtime::ethereum::read_ethereum_event;
#[cfg(target_arch = "wasm32")]
use graph_as_runtime::ffi::{LOG_INFO, log_log};

mod generated;
use generated::{Approval, ERC721ApprovalEvent, ERC721TransferEvent, Token, Transfer};

/// Format a raw byte slice as a lowercase hex string (no "0x" prefix).
fn hex_bytes(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

// ============================================================================
// Core handler logic — runs on both WASM and native
// ============================================================================

pub fn handle_transfer_impl(raw: &RawEthereumEvent) {
    let event = match ERC721TransferEvent::from_raw_event(raw) {
        Ok(e) => e,
        Err(_) => return,
    };

    let id = format!(
        "{}-{}",
        hex_bytes(&event.tx_hash),
        hex_bytes(&event.log_index)
    );

    Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_token_id(event.token_id.clone())
        .set_block_number(event.block_number.clone())
        .set_timestamp(event.block_timestamp.clone())
        .set_transaction_hash(event.tx_hash.to_vec())
        .save();

    let token_id_str = hex_bytes(&event.token_id);
    Token::new(&token_id_str)
        .set_owner(event.to.to_vec())
        .set_approved(alloc::vec![0u8; 20])
        .save();
}

pub fn handle_approval_impl(raw: &RawEthereumEvent) {
    let event = match ERC721ApprovalEvent::from_raw_event(raw) {
        Ok(e) => e,
        Err(_) => return,
    };

    let id = format!(
        "{}-{}",
        hex_bytes(&event.tx_hash),
        hex_bytes(&event.log_index)
    );

    Approval::new(&id)
        .set_owner(event.owner.to_vec())
        .set_approved(event.approved.to_vec())
        .set_token_id(event.token_id.clone())
        .set_block_number(event.block_number.clone())
        .set_transaction_hash(event.tx_hash.to_vec())
        .save();

    let token_id_str = hex_bytes(&event.token_id);
    let token = match Token::load(&token_id_str) {
        Some(t) => t.set_approved(event.approved.to_vec()),
        None => Token::new(&token_id_str)
            .set_owner(event.owner.to_vec())
            .set_approved(event.approved.to_vec()),
    };
    token.save();
}

// ============================================================================
// WASM entry points — only compiled for wasm32
// ============================================================================

/// Handle ERC721 Transfer events.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn handle_transfer(event_ptr: i32) {
    unsafe {
        log_log(LOG_INFO, new_asc_string("erc721: handle_transfer called"));
    }
    let raw = unsafe { read_ethereum_event(event_ptr as u32) };
    unsafe {
        log_log(LOG_INFO, new_asc_string("erc721: event read ok"));
    }
    handle_transfer_impl(&raw);
    unsafe {
        log_log(LOG_INFO, new_asc_string("erc721: Transfer entity saved"));
    }
}

/// Handle ERC721 Approval events.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn handle_approval(event_ptr: i32) {
    let raw = unsafe { read_ethereum_event(event_ptr as u32) };
    handle_approval_impl(&raw);
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
            block_number: vec![1, 0, 0, 0],
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
                    name: "tokenId".into(),
                    value: EthereumValue::Uint(alloc::vec![7, 0, 0, 0, 0, 0, 0, 0]),
                },
            ],
            receipt: None,
        }
    }

    fn mock_approval_event() -> RawEthereumEvent {
        RawEthereumEvent {
            address: [0x00; 20],
            log_index: vec![1],
            block_number: vec![2, 0, 0, 0],
            block_timestamp: vec![200, 0, 0, 0],
            tx_hash: [0xcd; 32],
            params: alloc::vec![
                EventParam {
                    name: "owner".into(),
                    value: EthereumValue::Address([0xaa; 20]),
                },
                EventParam {
                    name: "approved".into(),
                    value: EthereumValue::Address([0xcc; 20]),
                },
                EventParam {
                    name: "tokenId".into(),
                    value: EthereumValue::Uint(alloc::vec![7, 0, 0, 0, 0, 0, 0, 0]),
                },
            ],
            receipt: None,
        }
    }

    #[test]
    fn transfer_creates_transfer_and_token_entities() {
        mock::reset();

        handle_transfer_impl(&mock_transfer_event());

        let tx_hex = "ab".repeat(32);
        let id = format!("{}-00", tx_hex);

        assert!(mock::has_entity("Transfer", &id));
        mock::assert_entity("Transfer", &id)
            .field_bytes("from", &[0xaa; 20])
            .field_bytes("to", &[0xbb; 20])
            .field_exists("tokenId")
            .field_exists("blockNumber");

        // Token entity should track owner
        let token_id_str = "0700000000000000";
        assert!(mock::has_entity("Token", token_id_str));
        mock::assert_entity("Token", token_id_str).field_bytes("owner", &[0xbb; 20]);
    }

    #[test]
    fn transfer_clears_approval() {
        mock::reset();

        handle_transfer_impl(&mock_transfer_event());

        let token_id_str = "0700000000000000";
        mock::assert_entity("Token", token_id_str).field_bytes("approved", &[0u8; 20]);
    }

    #[test]
    fn approval_creates_approval_and_updates_token() {
        mock::reset();

        // Transfer first so the token exists
        handle_transfer_impl(&mock_transfer_event());
        handle_approval_impl(&mock_approval_event());

        let tx_hex = "cd".repeat(32);
        let id = format!("{}-01", tx_hex);

        assert!(mock::has_entity("Approval", &id));
        mock::assert_entity("Approval", &id)
            .field_bytes("owner", &[0xaa; 20])
            .field_bytes("approved", &[0xcc; 20])
            .field_exists("tokenId");

        // Token should now have 0xcc as approved
        let token_id_str = "0700000000000000";
        mock::assert_entity("Token", token_id_str).field_bytes("approved", &[0xcc; 20]);
    }

    #[test]
    fn load_returns_none_for_missing_entity() {
        mock::reset();
        assert!(Token::load("nonexistent").is_none());
        assert!(Transfer::load("nonexistent").is_none());
    }

    #[test]
    fn load_returns_saved_entity_and_can_be_resaved() {
        mock::reset();

        handle_transfer_impl(&mock_transfer_event());

        let token_id_str = "0700000000000000";

        // load() should find the entity the handler wrote
        let token = Token::load(token_id_str).expect("Token should exist after transfer");

        // Round-trip: re-save the loaded entity, store should still have it
        token.save();
        assert!(mock::has_entity("Token", token_id_str));
        mock::assert_entity("Token", token_id_str).field_bytes("owner", &[0xbb; 20]);
    }

    #[test]
    fn transfer_ownership_change() {
        mock::reset();

        // First transfer: 0xaa → 0xbb
        handle_transfer_impl(&mock_transfer_event());

        // Second transfer: 0xbb → 0xdd (same token)
        let mut event2 = mock_transfer_event();
        event2.tx_hash = [0xef; 32];
        event2.params[0] = EventParam {
            name: "from".into(),
            value: EthereumValue::Address([0xbb; 20]),
        };
        event2.params[1] = EventParam {
            name: "to".into(),
            value: EthereumValue::Address([0xdd; 20]),
        };
        handle_transfer_impl(&event2);

        let token_id_str = "0700000000000000";
        mock::assert_entity("Token", token_id_str).field_bytes("owner", &[0xdd; 20]);

        assert_eq!(mock::entity_count("Transfer"), 2);
        assert_eq!(mock::entity_count("Token"), 1);
    }
}
