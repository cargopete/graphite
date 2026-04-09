# ERC20 Subgraph Example

Indexes ERC20 `Transfer(indexed address from, indexed address to, uint256 value)` events and stores each one as a `Transfer` entity. Pre-configured for USDC on Ethereum mainnet (`0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`). This is the reference example — it has been tested live against Arbitrum One on an unmodified graph-node.

## What it demonstrates

- Generating typed Rust bindings from an ABI with `graphite codegen`
- The `handle_transfer_impl` / `handle_transfer` split: testable core logic separate from the WASM entry point
- Mapping event fields to entity fields and calling `save`
- Graph-node metadata fields available in every event: `block_number`, `block_timestamp`, `tx_hash`, `log_index`
- Native unit testing via `graphite::mock` — no Docker, no PostgreSQL

## Build

```bash
# From this directory
cargo build --target wasm32-unknown-unknown --release
```

Or using the CLI (which also runs wasm-opt if available):

```bash
graphite build
```

The WASM output lands at `../../target/wasm32-unknown-unknown/release/erc20_subgraph.wasm`, which is the path referenced in `subgraph.yaml`.

## Test

```bash
# From this directory
cargo test

# Or from the repo root
cargo test -p erc20-subgraph
```

Tests run natively — they use an in-process mock store and do not require a running graph-node.

## Deploy

See [docs/getting-started.md](../../docs/getting-started.md) for the full deployment walkthrough.

Quick version:

```bash
graphite deploy --node http://localhost:8020 --ipfs http://localhost:5001 myname/erc20-subgraph
```

To target a different contract, update `address` and `startBlock` in `subgraph.yaml`.
