# ERC20

**Source:** [`examples/erc20/`](https://github.com/your-org/graphite/tree/main/examples/erc20)
**Status:** Live on The Graph Studio (Arbitrum One)

Indexes ERC20 `Transfer` events. The simplest possible subgraph — one event, one entity.

## Schema

```graphql
type Transfer @entity {
  id: ID!
  from: Bytes!
  to: Bytes!
  value: BigInt!
  blockNumber: BigInt!
  timestamp: BigInt!
  transactionHash: Bytes!
}
```

## Handler

```rust
pub fn handle_transfer_impl(raw: &RawEthereumEvent) {
    let event = match ERC20TransferEvent::from_raw_event(raw) {
        Ok(e) => e,
        Err(_) => return,
    };

    let id = format!("{}-{}", hex_bytes(&event.tx_hash), hex_bytes(&event.log_index));

    Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_value(event.value)
        .set_block_number(event.block_number)
        .set_timestamp(event.block_timestamp)
        .set_transaction_hash(event.tx_hash.to_vec())
        .save();
}
```

## Tests

The example includes three tests:

- **`transfer_creates_entity`** — verifies the entity is created with correct field values.
- **`transfer_entity_count`** — verifies upsert behaviour (same tx hash = same ID = 1 entity) and that different tx hashes produce distinct entities.

```bash
cd examples/erc20
cargo test
```

## Key Points

- Entity ID is `{tx_hash_hex}-{log_index_hex}` — globally unique per log entry.
- The handler uses the lower-level `handle_transfer_impl(raw: &RawEthereumEvent)` pattern (no `#[handler]` macro) — the WASM entry point is written manually. Both approaches work identically.
- All numeric fields (`value`, `blockNumber`, `timestamp`) are little-endian `Vec<u8>` BigInt bytes.
