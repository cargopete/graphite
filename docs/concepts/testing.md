# Testing

Graphite subgraphs can be tested with `cargo test` — no Docker, no PostgreSQL, no graph-node. Tests run natively using an in-process mock store.

## Basic Setup

```bash
cargo test
```

Tests live in the same `src/lib.rs` file alongside your handlers (or in a `tests/` directory). The `graphite::mock` module provides the in-memory store.

## The Mock API

### `mock::reset()`

Clears the entire in-memory store. Always call this at the start of each test to prevent state leaking between tests.

```rust
#[test]
fn my_test() {
    mock::reset();
    // ...
}
```

### `mock::has_entity(type, id)`

Returns `true` if an entity with the given type name and ID exists in the store.

```rust
assert!(mock::has_entity("Transfer", "0xabc-00"));
```

### `mock::entity_count(type)`

Returns the number of entities of a given type.

```rust
assert_eq!(mock::entity_count("Transfer"), 1);
```

### `mock::assert_entity(type, id)`

Returns an assertion builder for inspecting a specific entity's field values.

```rust
mock::assert_entity("Transfer", &id)
    .field_bytes("from", &[0xaa; 20])
    .field_bytes("to", &[0xbb; 20])
    .field_exists("value")
    .field_exists("blockNumber");
```

| Method | Description |
|--------|-------------|
| `.field_exists(name)` | Asserts the field is set |
| `.field_bytes(name, expected)` | Asserts a `Bytes`/`Address` field equals the given bytes |
| `.field_string(name, expected)` | Asserts a `String` field equals the given value |
| `.field_bool(name, expected)` | Asserts a `Boolean` field |

### `mock::set_call_result(result)`

Mocks the return value of an `ethereum.call`. See [Contract Calls](../advanced/contract-calls.md).

### `mock::set_current_address(address)`

Sets the value returned by `data_source::address_current()`. Useful when testing template handlers:

```rust
mock::set_current_address([0xAA; 20]);
handle_swap_impl(&event, &graphite::EventContext::default());
```

### `mock::assert_contract_data_source_created(template, address)`

Asserts that `data_source::create_contract` was called with the given template name and address:

```rust
mock::assert_contract_data_source_created("Pair", pair_address);
```

## Writing a Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use graph_as_runtime::ethereum::{EthereumValue, EventParam, FromRawEvent, RawEthereumEvent};
    use graphite::mock;

    fn mock_transfer() -> RawEthereumEvent {
        RawEthereumEvent {
            tx_hash: [0xab; 32],
            log_index: alloc::vec![0],
            block_number: alloc::vec![1, 0, 0, 0],
            block_timestamp: alloc::vec![100, 0, 0, 0],
            params: alloc::vec![
                EventParam { name: "from".into(),  value: EthereumValue::Address([0xaa; 20]) },
                EventParam { name: "to".into(),    value: EthereumValue::Address([0xbb; 20]) },
                EventParam { name: "value".into(), value: EthereumValue::Uint(alloc::vec![100]) },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn transfer_creates_entity() {
        mock::reset();

        let raw = mock_transfer();
        let event = ERC20TransferEvent::from_raw_event(&raw).unwrap();
        handle_transfer_impl(&event, &graphite::EventContext::default());

        let tx_hex = "ab".repeat(32);
        let id = format!("{}-00", tx_hex);

        assert!(mock::has_entity("Transfer", &id));
        mock::assert_entity("Transfer", &id)
            .field_bytes("from", &[0xaa; 20])
            .field_bytes("to", &[0xbb; 20])
            .field_exists("value");
    }

    #[test]
    fn upsert_does_not_duplicate() {
        mock::reset();

        let raw = mock_transfer();
        let event = ERC20TransferEvent::from_raw_event(&raw).unwrap();

        handle_transfer_impl(&event, &graphite::EventContext::default());
        handle_transfer_impl(&event, &graphite::EventContext::default()); // same id

        assert_eq!(mock::entity_count("Transfer"), 1);
    }
}
```

## Constructing Mock Events

`RawEthereumEvent` has a `Default` implementation — you only need to set the fields your handler uses.

```rust
RawEthereumEvent {
    tx_hash: [0xab; 32],
    log_index: alloc::vec![0],
    block_number: alloc::vec![1, 0, 0, 0],  // little-endian: block 1
    block_timestamp: alloc::vec![100, 0, 0, 0],
    params: alloc::vec![
        EventParam { name: "from".into(), value: EthereumValue::Address([0xaa; 20]) },
        EventParam { name: "value".into(), value: EthereumValue::Uint(alloc::vec![100]) },
    ],
    ..Default::default()
}
```

### EthereumValue Variants

| Variant | Solidity type |
|---------|--------------|
| `EthereumValue::Address([u8; 20])` | `address` |
| `EthereumValue::Uint(Vec<u8>)` | `uint256`, `uint128`, etc. (LE bytes) |
| `EthereumValue::Int(Vec<u8>)` | `int256`, `int128`, etc. (LE signed bytes) |
| `EthereumValue::Bool(bool)` | `bool` |
| `EthereumValue::String(String)` | `string` |
| `EthereumValue::Bytes(Vec<u8>)` | `bytes` |
| `EthereumValue::FixedBytes([u8; N])` | `bytesN` |
| `EthereumValue::Array(Vec<EthereumValue>)` | array types |
