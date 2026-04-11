//! File data source example — IPFS NFT metadata indexing.
//!
//! Demonstrates the `file/ipfs` template pattern:
//!
//! 1. `handle_transfer` fires on every ERC721 Transfer event.
//!    It creates an initial `NFT` entity and spawns a `file/ipfs` data source
//!    for the token's metadata CID so graph-node will fetch the IPFS content.
//!
//! 2. `handle_nft_metadata` fires when graph-node delivers the IPFS bytes.
//!    It parses the JSON metadata and updates the `NFT` entity with name,
//!    description, and image URI.
//!
//! In production you would call `tokenURI(uint256)` on the ERC721 contract to
//! get the real IPFS CID. This example uses a synthesised CID for simplicity.

#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use graphite::data_source;
use graphite_macros::handler;

mod generated;
use generated::{ERC721TransferEvent, NFT};

// ============================================================================
// Helpers
// ============================================================================

/// Format a little-endian BigInt byte slice as a decimal string.
fn le_bytes_to_decimal(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "0".into();
    }
    // Simple single-word fast path (fits in u64).
    let mut n = 0u64;
    let len = bytes.len().min(8);
    for (i, &b) in bytes[..len].iter().enumerate() {
        n |= (b as u64) << (i * 8);
    }
    format!("{}", n)
}

// ============================================================================
// ERC721 Transfer handler
// ============================================================================

/// Handle an ERC721 Transfer event.
///
/// Creates an initial `NFT` entity, then triggers an IPFS fetch for the
/// token metadata. In production the tokenURI would be retrieved via a
/// contract view call; here we construct a placeholder CID.
#[handler]
pub fn handle_transfer(event: &ERC721TransferEvent, ctx: &graphite::EventContext) {
    let token_id = le_bytes_to_decimal(&event.token_id);

    // Save initial entity with on-chain data.
    NFT::new(&token_id)
        .set_owner(event.to.to_vec())
        .set_token_uri(format!("ipfs://Qm{}", token_id))
        .save();

    // Trigger IPFS metadata fetch. The CID is passed as `params[0]`.
    // graph-node will call `handle_nft_metadata` once the content arrives.
    //
    // Pass the token ID in the context so `handle_nft_metadata` knows which
    // entity to update.
    let cid = format!("Qm{}", token_id);
    data_source::create_file_with_context(
        "NFTMetadata",
        &cid,
        &[("tokenId", &token_id)],
    );
}

// ============================================================================
// IPFS file handler
// ============================================================================

/// Handle the IPFS content fetched for a token's metadata CID.
///
/// Parses the JSON payload and updates the `NFT` entity with `name`,
/// `description`, and `imageURI`.
#[handler(file)]
pub fn handle_nft_metadata(content: alloc::vec::Vec<u8>, _ctx: &graphite::FileContext) {
    // Retrieve the token ID from the data source context.
    let token_id = match data_source::context_string("tokenId") {
        Some(id) => id,
        None => return,
    };

    // Load the entity created by handle_transfer.
    let nft = match NFT::load(&token_id) {
        Some(n) => n,
        None => return,
    };

    // Parse the JSON metadata.
    let json_str = match alloc::str::from_utf8(&content) {
        Ok(s) => s,
        Err(_) => return,
    };

    let name = extract_json_string(json_str, "name");
    let description = extract_json_string(json_str, "description");
    let image = extract_json_string(json_str, "image");

    // Update and save.
    let mut updated = nft;
    if let Some(n) = name {
        updated = updated.set_name(n);
    }
    if let Some(d) = description {
        updated = updated.set_description(d);
    }
    if let Some(img) = image {
        updated = updated.set_image_uri(img);
    }
    updated.save();
}

// ============================================================================
// Minimal JSON string extractor (no_std, no dependencies)
// ============================================================================

/// Extract the value of a top-level string field from a JSON object.
///
/// Handles simple cases: `{"name": "value", ...}`. Does not handle escaped
/// quotes or nested objects. Sufficient for standard NFT metadata.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\"", key);
    let key_pos = json.find(&search)?;
    let after_key = &json[key_pos + search.len()..];
    // Skip whitespace and colon.
    let after_colon = after_key.trim_start().strip_prefix(':')?.trim_start();
    if !after_colon.starts_with('"') {
        return None;
    }
    let inner = &after_colon[1..];
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use graph_as_runtime::ethereum::{EthereumValue, EventParam, FromRawEvent, RawEthereumEvent};
    use graphite::mock;

    fn transfer_event(token_id: u64, to: [u8; 20]) -> RawEthereumEvent {
        RawEthereumEvent {
            tx_hash: [0xab; 32],
            log_index: alloc::vec![0],
            block_number: alloc::vec![1, 0, 0, 0],
            block_timestamp: alloc::vec![100, 0, 0, 0],
            params: alloc::vec![
                EventParam {
                    name: "from".into(),
                    value: EthereumValue::Address([0x00; 20]),
                },
                EventParam {
                    name: "to".into(),
                    value: EthereumValue::Address(to),
                },
                EventParam {
                    name: "tokenId".into(),
                    value: EthereumValue::Uint(alloc::vec![token_id as u8]),
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn transfer_creates_nft_entity() {
        mock::reset();

        let raw = transfer_event(42, [0xcc; 20]);
        handle_transfer_impl(
            &ERC721TransferEvent::from_raw_event(&raw).unwrap(),
            &graphite::EventContext::default(),
        );

        assert!(mock::has_entity("NFT", "42"));
        mock::assert_entity("NFT", "42")
            .field_bytes("owner", &[0xcc; 20])
            .field_string("tokenURI", "ipfs://Qm42");
    }

    #[test]
    fn transfer_triggers_file_data_source() {
        mock::reset();

        let raw = transfer_event(7, [0xdd; 20]);
        handle_transfer_impl(
            &ERC721TransferEvent::from_raw_event(&raw).unwrap(),
            &graphite::EventContext::default(),
        );

        mock::assert_file_data_source_created("NFTMetadata", "Qm7");
    }

    #[test]
    fn metadata_handler_updates_entity() {
        mock::reset();

        // Seed the initial entity (as handle_transfer would have created it).
        let raw = transfer_event(1, [0xee; 20]);
        handle_transfer_impl(
            &ERC721TransferEvent::from_raw_event(&raw).unwrap(),
            &graphite::EventContext::default(),
        );

        // Set up the data source context (graph-node supplies this at runtime).
        mock::set_data_source_context("tokenId", "1");

        // Simulate graph-node delivering the IPFS content.
        let metadata = br#"{"name":"Cool NFT","description":"A very cool NFT","image":"ipfs://QmImage"}"#;
        handle_nft_metadata_impl(metadata.to_vec(), &graphite::FileContext::new());

        mock::assert_entity("NFT", "1")
            .field_string("name", "Cool NFT")
            .field_string("description", "A very cool NFT")
            .field_string("imageURI", "ipfs://QmImage");
    }

    #[test]
    fn extract_json_string_basic() {
        let json = r#"{"name":"Bored Ape","description":"An ape","image":"ipfs://Qmabc"}"#;
        assert_eq!(extract_json_string(json, "name"), Some("Bored Ape".into()));
        assert_eq!(extract_json_string(json, "description"), Some("An ape".into()));
        assert_eq!(extract_json_string(json, "image"), Some("ipfs://Qmabc".into()));
        assert_eq!(extract_json_string(json, "missing"), None);
    }
}
