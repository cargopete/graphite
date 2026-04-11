//! ERC-1155 multi-token subgraph.
//!
//! Demonstrates:
//! - TransferSingle handler (single token transfer)
//! - TransferBatch handler (batch transfer with array params)
//! - URI handler (token metadata URI update)
//! - `#[handler]` macro (both event variants)

#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

use alloc::{format, string::String};
use graphite_macros::handler;

mod generated;
use generated::{ERC1155TransferBatchEvent, ERC1155TransferSingleEvent, ERC1155UriEvent};
use generated::{Token, Transfer};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

/// Load or create a Token entity, update its totalSupply by `delta` (signed).
fn update_token_supply(token_id: &[u8], delta: i64) {
    let id = hex(token_id);
    let token = Token::load(&id).unwrap_or_else(|| Token::new(&id));
    // totalSupply is little-endian BigInt bytes — decode to i64 for arithmetic.
    let current: i64 = if token.total_supply().is_empty() {
        0
    } else {
        let mut arr = [0u8; 8];
        let len = token.total_supply().len().min(8);
        arr[..len].copy_from_slice(&token.total_supply()[..len]);
        i64::from_le_bytes(arr)
    };
    let next = current + delta;
    token.set_total_supply(next.to_le_bytes().to_vec()).save();
}

// ─── TransferSingle ──────────────────────────────────────────────────────────

#[handler]
pub fn handle_transfer_single(
    event: &ERC1155TransferSingleEvent,
    ctx: &graphite::EventContext,
) {
    let id = format!("{}-{}", hex(&ctx.tx_hash), hex(&ctx.log_index));

    // Update token supply: mint (+) or burn (-) or transfer (neutral).
    let is_mint = event.from == [0u8; 20];
    let is_burn = event.to == [0u8; 20];
    if is_mint {
        update_token_supply(&event.id, 1);
    } else if is_burn {
        update_token_supply(&event.id, -1);
    } else {
        // Make sure the token entity exists even for regular transfers.
        let token_id = hex(&event.id);
        if Token::load(&token_id).is_none() {
            Token::new(&token_id).save();
        }
    }

    Transfer::new(&id)
        .set_operator(event.operator.to_vec())
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_token(hex(&event.id))
        .set_amount(event.value.clone())
        .set_block_number(ctx.block_number.clone())
        .set_timestamp(ctx.block_timestamp.clone())
        .save();
}

// ─── TransferBatch ───────────────────────────────────────────────────────────

#[handler]
pub fn handle_transfer_batch(
    event: &ERC1155TransferBatchEvent,
    ctx: &graphite::EventContext,
) {
    let is_mint = event.from == [0u8; 20];
    let is_burn = event.to == [0u8; 20];

    for (i, (token_id_bytes, amount_bytes)) in
        event.ids.iter().zip(event.values.iter()).enumerate()
    {
        let entry_id = format!("{}-{}-{}", hex(&ctx.tx_hash), hex(&ctx.log_index), i);

        if is_mint {
            update_token_supply(token_id_bytes, 1);
        } else if is_burn {
            update_token_supply(token_id_bytes, -1);
        } else {
            let token_id = hex(token_id_bytes);
            if Token::load(&token_id).is_none() {
                Token::new(&token_id).save();
            }
        }

        Transfer::new(&entry_id)
            .set_operator(event.operator.to_vec())
            .set_from(event.from.to_vec())
            .set_to(event.to.to_vec())
            .set_token(hex(token_id_bytes))
            .set_amount(amount_bytes.clone())
            .set_block_number(ctx.block_number.clone())
            .set_timestamp(ctx.block_timestamp.clone())
            .save();
    }
}

// ─── URI ─────────────────────────────────────────────────────────────────────

#[handler]
pub fn handle_uri(event: &ERC1155UriEvent, ctx: &graphite::EventContext) {
    let token_id = hex(&event.id);
    let token = Token::load(&token_id).unwrap_or_else(|| Token::new(&token_id));
    token.set_uri(event.value.clone()).save();
    let _ = ctx;
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use graph_as_runtime::ethereum::{EthereumValue, EventParam, FromRawEvent, RawEthereumEvent};
    use graph_as_runtime::native_store;

    fn make_raw(
        from: [u8; 20],
        to: [u8; 20],
        operator: [u8; 20],
    ) -> RawEthereumEvent {
        RawEthereumEvent {
            address: [0u8; 20],
            log_index: vec![0],
            block_number: vec![1],
            block_timestamp: vec![100],
            tx_hash: [0xab; 32],
            params: alloc::vec![
                EventParam {
                    name: "operator".into(),
                    value: EthereumValue::Address(operator),
                },
                EventParam {
                    name: "from".into(),
                    value: EthereumValue::Address(from),
                },
                EventParam {
                    name: "to".into(),
                    value: EthereumValue::Address(to),
                },
            ],
            ..Default::default()
        }
    }

    fn reset() {
        native_store::reset();
    }

    #[test]
    fn transfer_single_mint_creates_token() {
        reset();
        let mint_address = [0u8; 20];
        let recipient = [0xbb; 20];
        let operator = [0xcc; 20];

        let mut raw = make_raw(mint_address, recipient, operator);
        raw.params.push(EventParam {
            name: "id".into(),
            value: EthereumValue::Uint(vec![1]),
        });
        raw.params.push(EventParam {
            name: "value".into(),
            value: EthereumValue::Uint(vec![10]),
        });

        let event = ERC1155TransferSingleEvent::from_raw_event(&raw).unwrap();
        let ctx = graphite::EventContext {
            block_number: vec![1],
            block_timestamp: vec![100],
            tx_hash: [0xab; 32],
            log_index: vec![0],
            address: [0u8; 20],
            ..Default::default()
        };
        handle_transfer_single_impl(&event, &ctx);

        assert_eq!(native_store::with_store(|s| s.entity_count("Token")), 1);
        assert_eq!(native_store::with_store(|s| s.entity_count("Transfer")), 1);
    }

    #[test]
    fn transfer_batch_creates_multiple_transfers() {
        reset();
        let from = [0u8; 20]; // mint
        let to = [0xdd; 20];
        let operator = [0xcc; 20];

        let mut raw = make_raw(from, to, operator);
        raw.params.push(EventParam {
            name: "ids".into(),
            value: EthereumValue::Array(alloc::vec![
                EthereumValue::Uint(vec![1]),
                EthereumValue::Uint(vec![2]),
                EthereumValue::Uint(vec![3]),
            ]),
        });
        raw.params.push(EventParam {
            name: "values".into(),
            value: EthereumValue::Array(alloc::vec![
                EthereumValue::Uint(vec![10]),
                EthereumValue::Uint(vec![20]),
                EthereumValue::Uint(vec![30]),
            ]),
        });

        let event = ERC1155TransferBatchEvent::from_raw_event(&raw).unwrap();
        let ctx = graphite::EventContext {
            block_number: vec![1],
            block_timestamp: vec![100],
            tx_hash: [0xab; 32],
            log_index: vec![0],
            address: [0u8; 20],
            ..Default::default()
        };
        handle_transfer_batch_impl(&event, &ctx);

        assert_eq!(native_store::with_store(|s| s.entity_count("Token")), 3);
        assert_eq!(native_store::with_store(|s| s.entity_count("Transfer")), 3);
    }

    #[test]
    fn uri_update_sets_token_uri() {
        reset();
        let raw = RawEthereumEvent {
            address: [0u8; 20],
            log_index: vec![0],
            block_number: vec![1],
            block_timestamp: vec![100],
            tx_hash: [0xab; 32],
            params: alloc::vec![
                EventParam {
                    name: "value".into(),
                    value: EthereumValue::String(
                        "https://api.example.com/token/{id}".into(),
                    ),
                },
                EventParam {
                    name: "id".into(),
                    value: EthereumValue::Uint(vec![42]),
                },
            ],
            ..Default::default()
        };

        let event = ERC1155UriEvent::from_raw_event(&raw).unwrap();
        let ctx = graphite::EventContext {
            block_number: vec![1],
            block_timestamp: vec![100],
            tx_hash: [0xab; 32],
            log_index: vec![0],
            address: [0u8; 20],
            ..Default::default()
        };
        handle_uri_impl(&event, &ctx);

        assert_eq!(native_store::with_store(|s| s.entity_count("Token")), 1);
    }
}
