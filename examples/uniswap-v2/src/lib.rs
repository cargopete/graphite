//! Uniswap V2 subgraph — factory + template pattern.
//!
//! Demonstrates dynamic data sources:
//!
//! 1. `handle_pair_created` fires on every `PairCreated` event from the Factory.
//!    It creates a `Pool` entity and calls `data_source::create_contract` to
//!    start indexing Swap events from the new pair contract.
//!
//! 2. `handle_swap` fires when a Swap event is emitted by any pair contract
//!    tracked via the `Pair` template. It records a `Swap` entity and
//!    increments the parent `Pool.swapCount`.

#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

use alloc::format;
use graphite::data_source;
use graphite_macros::handler;

mod generated;
use generated::{FactoryPairCreatedEvent, Pool, Swap, PairSwapEvent};

// ── Factory handler ──────────────────────────────────────────────────────────

/// Handle a `PairCreated` event.
///
/// Creates a `Pool` entity keyed by the pair address and spawns a dynamic
/// data source so the `Pair` template will start indexing its events.
#[handler]
pub fn handle_pair_created(event: &FactoryPairCreatedEvent, _ctx: &graphite::EventContext) {
    let pool_id = addr_hex(&event.pair);

    Pool::new(&pool_id)
        .set_token0(event.token0.to_vec())
        .set_token1(event.token1.to_vec())
        .save();

    data_source::create_contract("Pair", event.pair);
}

// ── Pair template handler ────────────────────────────────────────────────────

/// Handle a `Swap` event from a tracked pair contract.
///
/// Records a `Swap` entity and increments the parent pool's swap counter.
#[handler]
pub fn handle_swap(event: &PairSwapEvent, ctx: &graphite::EventContext) {
    // The pair address is the data source address at runtime.
    let pair_addr = data_source::address_current();
    let pool_id = addr_hex(&pair_addr);

    let swap_id = format!(
        "{}-{}",
        hex_bytes(&event.tx_hash),
        hex_bytes(&event.log_index),
    );

    Swap::new(&swap_id)
        .set_pool(pool_id.clone())
        .set_amount0_in(event.amount0_in.clone())
        .set_amount1_in(event.amount1_in.clone())
        .set_amount0_out(event.amount0_out.clone())
        .set_amount1_out(event.amount1_out.clone())
        .set_block_number(ctx.block_number.clone())
        .set_timestamp(ctx.block_timestamp.clone())
        .save();

    // Increment the pool's swap count.
    if let Some(pool) = Pool::load(&pool_id) {
        let new_count = le_add_one(pool.swap_count());
        pool.set_swap_count(new_count).save();
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Format a 20-byte address as a lowercase `0x`-prefixed hex string.
fn addr_hex(addr: &[u8; 20]) -> alloc::string::String {
    let mut s = alloc::string::String::with_capacity(42);
    s.push_str("0x");
    for b in addr.iter() {
        let hi = (b >> 4) as usize;
        let lo = (b & 0xf) as usize;
        s.push(HEX_CHARS[hi]);
        s.push(HEX_CHARS[lo]);
    }
    s
}

/// Format a byte slice as a lowercase hex string (no prefix).
fn hex_bytes(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

/// Add one to a little-endian BigInt byte vector.
fn le_add_one(bytes: &[u8]) -> alloc::vec::Vec<u8> {
    let mut result = bytes.to_vec();
    let mut carry = 1u16;
    for byte in result.iter_mut() {
        let sum = *byte as u16 + carry;
        *byte = sum as u8;
        carry = sum >> 8;
    }
    if carry > 0 {
        result.push(carry as u8);
    }
    result
}

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use graph_as_runtime::ethereum::{EthereumValue, EventParam, FromRawEvent, RawEthereumEvent};
    use graphite::mock;

    const PAIR_ADDR: [u8; 20] = [0xAA; 20];
    const TOKEN0: [u8; 20] = [0x11; 20];
    const TOKEN1: [u8; 20] = [0x22; 20];

    fn pair_created_event() -> RawEthereumEvent {
        RawEthereumEvent {
            tx_hash: [0xab; 32],
            log_index: alloc::vec![0],
            block_number: alloc::vec![1, 0, 0, 0],
            block_timestamp: alloc::vec![100, 0, 0, 0],
            params: alloc::vec![
                EventParam { name: "token0".into(), value: EthereumValue::Address(TOKEN0) },
                EventParam { name: "token1".into(), value: EthereumValue::Address(TOKEN1) },
                EventParam { name: "pair".into(),   value: EthereumValue::Address(PAIR_ADDR) },
                EventParam { name: "param_3".into(), value: EthereumValue::Uint(alloc::vec![1]) },
            ],
            ..Default::default()
        }
    }

    fn swap_event() -> RawEthereumEvent {
        RawEthereumEvent {
            address: PAIR_ADDR,
            tx_hash: [0xcd; 32],
            log_index: alloc::vec![1],
            block_number: alloc::vec![2, 0, 0, 0],
            block_timestamp: alloc::vec![200, 0, 0, 0],
            params: alloc::vec![
                EventParam { name: "sender".into(),     value: EthereumValue::Address([0x55; 20]) },
                EventParam { name: "amount0In".into(),  value: EthereumValue::Uint(alloc::vec![100]) },
                EventParam { name: "amount1In".into(),  value: EthereumValue::Uint(alloc::vec![0]) },
                EventParam { name: "amount0Out".into(), value: EthereumValue::Uint(alloc::vec![0]) },
                EventParam { name: "amount1Out".into(), value: EthereumValue::Uint(alloc::vec![99]) },
                EventParam { name: "to".into(),         value: EthereumValue::Address([0x66; 20]) },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn pair_created_makes_pool_and_data_source() {
        mock::reset();

        let raw = pair_created_event();
        handle_pair_created_impl(
            &FactoryPairCreatedEvent::from_raw_event(&raw).unwrap(),
            &graphite::EventContext::default(),
        );

        let pool_id = addr_hex(&PAIR_ADDR);
        assert!(mock::has_entity("Pool", &pool_id));
        mock::assert_entity("Pool", &pool_id)
            .field_bytes("token0", &TOKEN0)
            .field_bytes("token1", &TOKEN1);

        mock::assert_contract_data_source_created("Pair", PAIR_ADDR);
    }

    #[test]
    fn swap_creates_entity_and_increments_pool_count() {
        mock::reset();

        // First create the pool (as handle_pair_created would have).
        let factory_raw = pair_created_event();
        handle_pair_created_impl(
            &FactoryPairCreatedEvent::from_raw_event(&factory_raw).unwrap(),
            &graphite::EventContext::default(),
        );

        // Set the data source address so address_current() returns the pair.
        mock::set_current_address(PAIR_ADDR);

        let raw = swap_event();
        handle_swap_impl(
            &PairSwapEvent::from_raw_event(&raw).unwrap(),
            &graphite::EventContext::default(),
        );

        let tx_hex = "cd".repeat(32);
        let swap_id = format!("{}-01", tx_hex);
        assert!(mock::has_entity("Swap", &swap_id));

        let pool_id = addr_hex(&PAIR_ADDR);
        mock::assert_entity("Pool", &pool_id).field_exists("swapCount");
    }

    #[test]
    fn swap_count_increments_per_swap() {
        mock::reset();

        let factory_raw = pair_created_event();
        handle_pair_created_impl(
            &FactoryPairCreatedEvent::from_raw_event(&factory_raw).unwrap(),
            &graphite::EventContext::default(),
        );

        mock::set_current_address(PAIR_ADDR);

        // Two swaps — swap count should reach 2.
        let pool_id = addr_hex(&PAIR_ADDR);
        for i in 0u8..2 {
            let mut raw = swap_event();
            raw.tx_hash = [i; 32];
            handle_swap_impl(
                &PairSwapEvent::from_raw_event(&raw).unwrap(),
                &graphite::EventContext::default(),
            );
        }

        assert_eq!(mock::entity_count("Swap"), 2);

        let pool = Pool::load(&pool_id).expect("Pool should exist");
        // swapCount should be 2 in LE bytes: [2]
        assert_eq!(pool.swap_count(), &alloc::vec![2u8]);
    }

    #[test]
    fn multiple_pairs_independent() {
        mock::reset();

        let pair_b: [u8; 20] = [0xBB; 20];

        // Create pair A
        let _raw_a = pair_created_event();
        // Pair A uses a patched event with PAIR_ADDR
        let raw_a_patched = RawEthereumEvent {
            tx_hash: [0x01; 32],
            log_index: alloc::vec![0],
            block_number: alloc::vec![1, 0, 0, 0],
            block_timestamp: alloc::vec![100, 0, 0, 0],
            params: alloc::vec![
                EventParam { name: "token0".into(), value: EthereumValue::Address(TOKEN0) },
                EventParam { name: "token1".into(), value: EthereumValue::Address(TOKEN1) },
                EventParam { name: "pair".into(),   value: EthereumValue::Address(PAIR_ADDR) },
                EventParam { name: "param_3".into(), value: EthereumValue::Uint(alloc::vec![1]) },
            ],
            ..Default::default()
        };
        handle_pair_created_impl(
            &FactoryPairCreatedEvent::from_raw_event(&raw_a_patched).unwrap(),
            &graphite::EventContext::default(),
        );

        // Create pair B
        let raw_b = RawEthereumEvent {
            tx_hash: [0x02; 32],
            log_index: alloc::vec![0],
            block_number: alloc::vec![2, 0, 0, 0],
            block_timestamp: alloc::vec![200, 0, 0, 0],
            params: alloc::vec![
                EventParam { name: "token0".into(), value: EthereumValue::Address([0x33; 20]) },
                EventParam { name: "token1".into(), value: EthereumValue::Address([0x44; 20]) },
                EventParam { name: "pair".into(),   value: EthereumValue::Address(pair_b) },
                EventParam { name: "param_3".into(), value: EthereumValue::Uint(alloc::vec![2]) },
            ],
            ..Default::default()
        };
        handle_pair_created_impl(
            &FactoryPairCreatedEvent::from_raw_event(&raw_b).unwrap(),
            &graphite::EventContext::default(),
        );

        assert_eq!(mock::entity_count("Pool"), 2);
    }
}
