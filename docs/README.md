# Graphite

Write [The Graph](https://thegraph.com/) subgraph handlers in Rust. The compiled WASM is AssemblyScript-ABI-compatible — unmodified graph-node accepts it as a standard subgraph.

**Live on The Graph Studio.** ERC20 and ERC721 subgraphs are deployed and indexing on Arbitrum One. Zero graph-node changes required.

---

## Why Rust?

AssemblyScript is a subset of TypeScript that compiles to WASM. It works, but it gives up most of what makes typed languages useful: no closures, no iterators, no algebraic types, no real ecosystem. Graphite lets you write the same subgraph mappings in Rust and get all of that back — plus `cargo test` without Docker.

## How It Works

Graph-node identifies WASM subgraphs by the structure of their memory and the names of their exported functions — not by who wrote them. The `graph-as-runtime` crate implements the AssemblyScript memory model in Rust: 20-byte object headers, UTF-16LE strings, `TypedMap` entity layout, the full set of host function imports. The resulting WASM is structurally indistinguishable from AssemblyScript output. The manifest declares `language: wasm/assemblyscript` and graph-node accepts it without any special handling.

```
Your Rust handler
      │
      ▼
graphite-macros  (#[handler], #[derive(Entity)])
      │
      ▼
graph-as-runtime  (AS ABI: allocator, UTF-16LE strings, TypedMap, host imports)
      │
      ▼
WASM binary  ──────────────────►  unmodified graph-node / The Graph Studio
```

## Quick Example

```rust
#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

use alloc::format;
use graphite_macros::handler;

mod generated;
use generated::{ERC20TransferEvent, Transfer};

#[handler]
pub fn handle_transfer(event: &ERC20TransferEvent, ctx: &graphite::EventContext) {
    let id = format!("{}-{}", hex(&ctx.tx_hash), hex(&ctx.log_index));
    Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_value(event.value.clone())
        .set_block_number(ctx.block_number.clone())
        .set_timestamp(ctx.block_timestamp.clone())
        .save();
}
```

Test it natively — no Docker, no PostgreSQL:

```rust
#[test]
fn transfer_creates_entity() {
    mock::reset();
    handle_transfer_impl(&event, &graphite::EventContext::default());
    assert_eq!(mock::entity_count("Transfer"), 1);
}
```

## Feature Parity

| Feature | Status |
|---------|--------|
| Event / Call / Block / File handlers | ✅ |
| `store.set` / `store.get` / `store.remove` / `store.getInBlock` | ✅ |
| `ethereum.call`, `ethereum.encode`, `ethereum.decode` | ✅ |
| `log.info` / `log.warning` / `log.error` / `log.critical` | ✅ |
| `ipfs.cat`, `json.fromBytes`, `ens.nameByAddress` | ✅ |
| `dataSource.create` / `createWithContext` / context accessors | ✅ |
| `crypto.keccak256` / `sha256` / `sha3` / `secp256k1.recover` | ✅ |
| `BigInt` — full arithmetic, bitwise, shifts | ✅ |
| `BigDecimal` — full arithmetic | ✅ |
| All GraphQL scalar types | ✅ |
| Block handler filters (`polling`, `every: N`) | ✅ |
| Native `cargo test` (no Docker) | ✅ |
| Non-fatal errors | ✅ |

## Crates

| Crate | Purpose |
|-------|---------|
| `graph-as-runtime` | `no_std` AS ABI layer: allocator, type layout, host FFI |
| `graphite-macros` | `#[handler]`, `#[derive(Entity)]` proc macros |
| `graphite-cli` | CLI: `init`, `codegen`, `manifest`, `build`, `test`, `deploy` |
| `graphite-sdk` | User-facing SDK and `MockHost` for native testing |

The SDK crate is published as `graphite-sdk` but imported as `graphite`:

```toml
[dependencies]
graphite = { package = "graphite-sdk", version = "1", default-features = false }
```
