# Graphite

Write [The Graph](https://thegraph.com/) subgraph handlers in Rust. The compiled WASM is AssemblyScript-ABI-compatible — unmodified graph-node accepts it as a standard subgraph.

**v1.0.0 — Live on The Graph Studio (Arbitrum One).** ERC20 and ERC721 subgraphs deployed and indexing on the decentralised network. Zero graph-node changes required.

## What It Is

Graphite lets you write subgraph mappings in Rust instead of AssemblyScript. You get type safety, native `cargo test`, closures, iterators, and the full Rust ecosystem. graph-node sees perfectly ordinary AssemblyScript subgraph output and doesn't need to know otherwise.

## How It Works

Rust compiles to `wasm32-unknown-unknown`. The `graph-as-runtime` crate implements the AssemblyScript memory model — 20-byte object headers, UTF-16LE strings, `TypedMap` entity layout — so the WASM binary is structurally indistinguishable from AssemblyScript output. Host functions (`store.set`, `log.log`, etc.) are matched by name only, so Rust can import them directly. The manifest declares `language: wasm/assemblyscript`, `apiVersion: 0.0.6` — graph-node accepts it without any special handling.

```
Your Rust handler
      │
      ▼
graphite-macros  (#[handler], #[derive(Entity)])
      │
      ▼
graph-as-runtime  (AS ABI layer: allocator, UTF-16LE strings, TypedMap, host imports)
      │
      ▼
WASM binary  ──────────────────►  unmodified graph-node / The Graph Studio
```

## Quick Example

```rust
use graphite::prelude::*;

#[handler]
pub fn handle_transfer(event: TransferEvent) {
    let id = format!("{}-{}", hex(event.tx_hash), event.log_index[0]);
    Transfer::new(&id)
        .set_from(event.from)
        .set_to(event.to)
        .set_value(event.value)
        .save();
}

#[test]
fn test_transfer() {
    graphite::mock::reset();
    handle_transfer_impl(&mock_transfer());
    graphite::mock::assert_entity("Transfer", "0xabcd...-0")
        .field_bytes("from", &[0xaa; 20]);
}
```

See [examples/erc20/src/lib.rs](examples/erc20/src/lib.rs) for the full working handler.

## CLI

```bash
graphite init my-subgraph          # Scaffold a new subgraph project
graphite codegen                   # Generate Rust types from ABI + schema
graphite build                     # Compile to WASM (runs cargo + wasm-opt)
graphite deploy myname/mysubgraph  # Deploy to local graph-node

# Deploy to The Graph Studio
graphite deploy \
  --node https://api.studio.thegraph.com/deploy/ \
  --ipfs https://api.thegraph.com/ipfs/ \
  --deploy-key <YOUR_DEPLOY_KEY> \
  --version-label v1.0.0 \
  my-subgraph-slug
```

Install the CLI:

```bash
cargo install --git https://github.com/cargopete/graphite.git graphite-cli
```

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `graph-as-runtime` | `no_std` AS ABI layer: allocator, type layout, host FFI |
| `graphite-macros` | `#[handler]`, `#[derive(Entity)]` proc macros |
| `graphite-cli` | CLI: `init`, `codegen`, `build`, `deploy` |
| `graphite` | User-facing SDK, `MockHost` for native testing |

## Examples

- [examples/erc20](examples/erc20/) — ERC20 Transfer indexer (live on The Graph Studio, Arbitrum One)
- [examples/erc721](examples/erc721/) — ERC721 NFT transfer indexer (live on The Graph Studio, Arbitrum One)

## Documentation

- [docs/getting-started.md](docs/getting-started.md) — end-to-end tutorial

## Building

```bash
# Run tests natively (no WASM toolchain needed)
cargo test -p graphite -p graphite-macros

# Build an example to WASM
rustup target add wasm32-unknown-unknown
cargo build -p erc20-subgraph --target wasm32-unknown-unknown --release
```

## License

MIT OR Apache-2.0
