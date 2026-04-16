# Crypto Utilities

All crypto functions run natively in `cargo test` — no host calls, no mocking required.

## keccak256

```rust
use graphite::crypto;

let hash: [u8; 32] = crypto::keccak256(b"hello world");
```

Useful for computing event topics, storage slot keys, and entity IDs derived from content.

## sha256

```rust
let hash: [u8; 32] = crypto::sha256(b"hello world");
```

## sha3

```rust
let hash: [u8; 32] = crypto::sha3(b"hello world");
```

## secp256k1 Recovery

Recover an Ethereum address from a message hash and ECDSA signature:

```rust
let address: Option<[u8; 20]> = crypto::secp256k1_recover(
    &msg_hash,  // [u8; 32]
    &r,         // [u8; 32]
    &s,         // [u8; 32]
    v,          // u8 (recovery id: 0 or 1)
);
```

Returns `None` if the signature is invalid.

## Function Selector

Compute the 4-byte ABI function selector from a signature string:

```rust
let selector: [u8; 4] = crypto::selector("transfer(address,uint256)");
// → [0xa9, 0x05, 0x9c, 0xbb]
```

This is `keccak256(sig)[0..4]`, computed at compile time if used with a constant string.

## Event Topic

Compute an event topic from the event signature:

```rust
let topic: [u8; 32] = crypto::keccak256(b"Transfer(address,address,uint256)");
```

## Using in Tests

All these functions work identically in tests and in WASM:

```rust
#[test]
fn topic_matches() {
    let topic = crypto::keccak256(b"Transfer(address,address,uint256)");
    assert_eq!(topic[0], 0xdd);  // first byte of the Transfer topic
}
```
