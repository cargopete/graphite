# Graphite

Write [The Graph](https://thegraph.com/) subgraph handlers in Rust. The compiled WASM is AssemblyScript-ABI-compatible — unmodified graph-node accepts it as a standard subgraph.

**Live on The Graph Studio (Arbitrum One).** ERC20 and ERC721 subgraphs deployed and indexing on the decentralised network. Zero graph-node changes required.

## What It Is

Graphite lets you write subgraph mappings in Rust instead of AssemblyScript. You get type safety, native `cargo test`, closures, iterators, and the full Rust ecosystem. graph-node sees perfectly ordinary AssemblyScript subgraph output and doesn't need to know otherwise.

## How It Works

Rust compiles to `wasm32-unknown-unknown`. The `graph-as-runtime` crate implements the AssemblyScript memory model — 20-byte object headers, UTF-16LE strings, `TypedMap` entity layout — so the WASM binary is structurally indistinguishable from AssemblyScript output. Host functions (`store.set`, `log.log`, etc.) are matched by name only, so Rust can import them directly. The manifest declares `language: wasm/assemblyscript` — graph-node accepts it without any special handling.

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
#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

use alloc::format;
use graphite_macros::handler;

mod generated;
use generated::{ERC20TransferEvent, Transfer};

#[handler]
pub fn handle_transfer(event: &ERC20TransferEvent, ctx: &graphite::EventContext) {
    let id = format!("{}-{}", hex(&event.tx_hash), hex(&event.log_index));
    Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_value(event.value.clone())
        .set_block_number(ctx.block_number.clone())
        .set_timestamp(ctx.block_timestamp.clone())
        .save();
}

fn hex(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_as_runtime::ethereum::{EthereumValue, EventParam, FromRawEvent, RawEthereumEvent};
    use graphite::mock;

    #[test]
    fn transfer_creates_entity() {
        mock::reset();
        let raw = RawEthereumEvent {
            tx_hash: [0xab; 32],
            params: alloc::vec![
                EventParam { name: "from".into(), value: EthereumValue::Address([0xaa; 20]) },
                EventParam { name: "to".into(),   value: EthereumValue::Address([0xbb; 20]) },
                EventParam { name: "value".into(), value: EthereumValue::Uint(alloc::vec![100]) },
            ],
            ..Default::default()
        };
        handle_transfer_impl(
            &ERC20TransferEvent::from_raw_event(&raw).unwrap(),
            &graphite::EventContext::default(),
        );

        assert!(mock::has_entity("Transfer", &format!("{}-00", "ab".repeat(32))));
    }
}
```

See [examples/erc20/src/lib.rs](examples/erc20/src/lib.rs) for the full working handler.

## Feature Parity with AssemblyScript graph-ts

| Feature | Status |
|---------|--------|
| Event handlers (`#[handler]`) | ✅ |
| Call handlers (`#[handler(call)]`) | ✅ |
| Block handlers (`#[handler(block)]`) | ✅ |
| File data source handlers (`#[handler(file)]`) | ✅ |
| Full block + tx + receipt context fields | ✅ |
| `store.set` / `store.get` / `store.remove` | ✅ |
| `store.getInBlock` | ✅ |
| `ethereum.call` (contract view calls) | ✅ |
| `ethereum.encode` / `ethereum.decode` | ✅ |
| `log.info` / `log.warning` / `log.error` / `log.critical` | ✅ |
| `ipfs.cat` | ✅ |
| `json.fromBytes` | ✅ |
| `ens.nameByAddress` | ✅ |
| `dataSource.create` / `createWithContext` | ✅ |
| `dataSource.address` / `network` / `context` / `id` | ✅ |
| `crypto.keccak256` / `sha256` / `sha3` / `secp256k1.recover` | ✅ |
| `BigInt` — full arithmetic, bitwise, shifts | ✅ |
| `BigDecimal` — full arithmetic | ✅ |
| All GraphQL scalar types (`String`, `Int`, `Float`, `Boolean`, `BigInt`, `BigDecimal`, `Bytes`, `Address`, `Timestamp`, `Int8`, `DateTime`) | ✅ |
| Block handler filters (`polling`, `every: N`) | ✅ |
| Native `cargo test` (no Docker) | ✅ |

## CLI

```bash
# Scaffold a new project (optionally fetching ABI from Etherscan)
graphite init my-subgraph
graphite init my-subgraph --from-contract 0xA0b...eB48 --network mainnet

graphite codegen                   # Generate Rust types from ABI + schema
graphite manifest                  # Generate subgraph.yaml from graphite.toml
graphite build                     # Compile to WASM (runs cargo + wasm-opt)
graphite test                      # Run tests (cargo test)
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
cargo install graphite-cli
```

### Block and Call Handlers in graphite.toml

```toml
[[contracts]]
name = "ERC20"
abi  = "abis/ERC20.json"
address    = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
start_block = 6082465

# Optional: index every block
[[contracts.block_handlers]]
handler = "handleBlock"

# Or poll every N blocks
[[contracts.block_handlers]]
handler = "handleBlockPolled"
filter = { kind = "polling", every = 10 }

# Call handlers
[[contracts.call_handlers]]
function = "transfer(address,uint256)"
handler  = "handleTransfer"

# IPFS file data source template
[[templates]]
name    = "NFTMetadata"
kind    = "file/ipfs"
handler = "handle_nft_metadata"
```

## Crate Structure

| Crate (crates.io name) | Purpose |
|------------------------|---------|
| `graph-as-runtime` | `no_std` AS ABI layer: allocator, type layout, host FFI |
| `graphite-macros` | `#[handler]`, `#[derive(Entity)]` proc macros |
| `graphite-cli` | CLI: `init`, `codegen`, `manifest`, `build`, `test`, `deploy` |
| `graphite-sdk` | User-facing SDK, `MockHost` for native testing |

The SDK crate is published as `graphite-sdk` but imported as `graphite`:

```toml
[dependencies]
graphite = { package = "graphite-sdk", version = "1", default-features = false }
```

## Examples

- [examples/erc20](examples/erc20/) — ERC20 Transfer indexer (live on The Graph Studio, Arbitrum One)
- [examples/erc721](examples/erc721/) — ERC721 NFT transfer and approval indexer
- [examples/erc1155](examples/erc1155/) — ERC1155 multi-token: TransferSingle, TransferBatch, URI
- [examples/multi-source](examples/multi-source/) — Multiple contracts in one subgraph
- [examples/file-ds](examples/file-ds/) — File data source (IPFS content handler)
- [examples/uniswap-v2](examples/uniswap-v2/) — Factory + template pattern: tracks pairs and swaps

## Documentation

- [docs/getting-started.md](docs/getting-started.md) — end-to-end tutorial

## Building

```bash
# Run tests natively (no WASM toolchain needed)
cargo test --workspace

# Build an example to WASM
rustup target add wasm32-unknown-unknown
cargo build -p erc20-subgraph --target wasm32-unknown-unknown --release
```

## License

MIT OR Apache-2.0
