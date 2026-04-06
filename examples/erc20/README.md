# ERC20 Subgraph Example

Indexes ERC20 `Transfer(address indexed from, address indexed to, uint256 value)` events and stores each one as a `Transfer` entity.

Pre-configured for USDC on Ethereum mainnet (`0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`). Change `address` and `startBlock` in `subgraph.yaml` to target any ERC20 contract.

## What it demonstrates

- Generating Rust types from an ABI with `graphite codegen`
- The `#[handler]` macro and the `WasmHost` calling convention
- Basic entity creation: mapping event fields to entity fields and calling `save`
- Using the metadata fields injected by graph-node (`block_number`, `block_timestamp`, `tx_hash`)

## Build

```bash
# From the repo root
cargo build -p erc20-subgraph --target wasm32-unknown-unknown --release
```

The WASM output is written to `../../target/wasm32-unknown-unknown/release/erc20_subgraph.wasm`, which is the path referenced in `subgraph.yaml`.

## Test

```bash
cargo test -p erc20-subgraph
```

## Deploy

```bash
graphite deploy <your-node>/myname/erc20-subgraph
```

This uploads the WASM to IPFS and calls the graph-node JSON-RPC endpoint. Requires a running graph-node fork with Rust ABI support ([PR #6462](https://github.com/graphprotocol/graph-node/pull/6462)).
