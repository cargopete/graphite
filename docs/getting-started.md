# Getting Started with Graphite

This guide walks you through building and deploying a Rust subgraph from scratch. The reference example is an ERC20 Transfer indexer — the same one that's been tested live on Arbitrum One.

---

## Prerequisites

- **Rust** — install via [rustup](https://rustup.rs/). You need the `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- **wasm-opt** — used by `graphite build` to shrink the binary. Optional but recommended:
  ```bash
  # via cargo
  cargo install wasm-opt
  # or via Homebrew (macOS)
  brew install binaryen
  ```
- **A running graph-node** — local Docker setup or The Graph hosted service. For local development, the [graph-node Docker Compose](https://github.com/graphprotocol/graph-node/tree/master/docker) setup is the quickest path.
- **graphite-cli** — install from the repo:
  ```bash
  cargo install --git https://github.com/cargopete/graphite.git graphite-cli
  ```

---

## 1. Create a New Project

```bash
graphite init my-subgraph --network mainnet
cd my-subgraph
```

This generates:

```
my-subgraph/
├── Cargo.toml           # cdylib crate, depends on graphite
├── graphite.toml        # Graphite config: ABI paths, output dir
├── subgraph.yaml        # The Graph manifest
├── schema.graphql       # GraphQL entity schema
├── abis/
│   └── my-subgraph.json # Placeholder ABI (replace with your own)
└── src/
    └── lib.rs           # Skeleton handler
```

---

## 2. Define Your Schema

Edit `schema.graphql` to declare your entities. For an ERC20 Transfer indexer:

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

Each `@entity` type becomes a Rust struct with a builder that codegen produces for you.

---

## 3. Add Your Contract ABI

Drop the contract's ABI JSON into the `abis/` directory:

```bash
cp path/to/ERC20.json abis/ERC20.json
```

The ABI must be standard Ethereum JSON ABI format — an array of event and function descriptors. For an ERC20 Transfer event:

```json
[
  {
    "anonymous": false,
    "inputs": [
      { "indexed": true,  "name": "from",  "type": "address" },
      { "indexed": true,  "name": "to",    "type": "address" },
      { "indexed": false, "name": "value", "type": "uint256" }
    ],
    "name": "Transfer",
    "type": "event"
  }
]
```

---

## 4. Configure the Subgraph

Update `graphite.toml` to point at your ABI:

```toml
output_dir = "src/generated"
schema = "schema.graphql"

[[contracts]]
name = "ERC20"
abi = "abis/ERC20.json"
```

Update `subgraph.yaml` with your contract address and start block:

```yaml
specVersion: 0.0.4
schema:
  file: ./schema.graphql
dataSources:
  - kind: ethereum
    name: ERC20
    network: mainnet
    source:
      address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
      abi: ERC20
      startBlock: 6082465
    mapping:
      kind: wasm/assemblyscript
      apiVersion: 0.0.6
      language: wasm/assemblyscript
      entities:
        - Transfer
      abis:
        - name: ERC20
          file: ./abis/ERC20.json
      eventHandlers:
        - event: Transfer(indexed address,indexed address,uint256)
          handler: handle_transfer
      file: ./target/wasm32-unknown-unknown/release/my_subgraph.wasm
```

Key points:
- `language: wasm/assemblyscript` and `apiVersion: 0.0.6` are what graph-node expects. Graphite produces WASM that satisfies these constraints despite being compiled from Rust.
- `startBlock` should be the block your contract was deployed on — indexing from block 0 is slow.
- The `handler` name must match the `pub fn` name in your Rust code (with the `#[handler]` attribute).

---

## 5. Run Codegen

```bash
graphite codegen
```

This reads `graphite.toml` and generates Rust source into `src/generated/`:

```
src/generated/
├── mod.rs        # Re-exports everything
├── erc20.rs      # Event structs from the ABI (ERC20TransferEvent, etc.)
└── schema.rs     # Entity builders from schema.graphql (Transfer, etc.)
```

The generated `ERC20TransferEvent` struct has typed fields for each ABI parameter (`from`, `to`, `value`) plus graph-node metadata fields (`tx_hash`, `log_index`, `block_number`, `block_timestamp`).

---

## 6. Write Your Handler

Edit `src/lib.rs`. A complete ERC20 Transfer handler:

```rust
#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;
use alloc::format;

use graph_as_runtime::ethereum::{FromRawEvent, RawEthereumEvent};

mod generated;
use generated::{ERC20TransferEvent, Transfer};

fn hex_bytes(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

/// Core handler logic — runs on WASM and in native tests.
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

/// WASM entry point — called by graph-node for each Transfer event.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn handle_transfer(event_ptr: i32) {
    use graph_as_runtime::ethereum::read_ethereum_event;
    let raw = unsafe { read_ethereum_event(event_ptr as u32) };
    handle_transfer_impl(&raw);
}
```

The pattern is:
1. `handle_transfer_impl` contains the logic. It receives a `RawEthereumEvent`, decodes it, and saves an entity. This function is callable from tests without any WASM involved.
2. `handle_transfer` is the WASM entry point — only compiled for `wasm32`. It receives the pointer from graph-node, reads the event, and delegates to `handle_transfer_impl`.

---

## 7. Test Natively

```bash
cargo test
```

No Docker, no PostgreSQL, no graph-node. Tests use an in-process mock store:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use graph_as_runtime::ethereum::{EthereumValue, EventParam};
    use graphite::mock;

    fn mock_transfer() -> RawEthereumEvent {
        RawEthereumEvent {
            address: [0x00; 20],
            log_index: vec![0],
            block_number: vec![1, 0, 0, 0],
            block_timestamp: vec![100, 0, 0, 0],
            tx_hash: [0xab; 32],
            params: alloc::vec![
                EventParam { name: "from".into(), value: EthereumValue::Address([0xaa; 20]) },
                EventParam { name: "to".into(),   value: EthereumValue::Address([0xbb; 20]) },
                EventParam { name: "value".into(), value: EthereumValue::Uint(alloc::vec![100, 0, 0, 0, 0, 0, 0, 0]) },
            ],
        }
    }

    #[test]
    fn transfer_creates_entity() {
        mock::reset();
        handle_transfer_impl(&mock_transfer());

        let tx_hex = "ab".repeat(32);
        let id = format!("{}-00", tx_hex);

        assert!(mock::has_entity("Transfer", &id));
        mock::assert_entity("Transfer", &id)
            .field_bytes("from", &[0xaa; 20])
            .field_bytes("to", &[0xbb; 20])
            .field_exists("value");
    }
}
```

`mock::reset()` clears the in-memory store between tests. `mock::assert_entity` returns a fluent assertion builder.

---

## 8. Build

```bash
graphite build
```

This runs `cargo build --target wasm32-unknown-unknown --release`, then copies the WASM to `build/` and runs `wasm-opt -Oz` if available. A minimal handler lands around 50–80 KB after optimisation.

To build manually without the CLI:

```bash
cargo build --target wasm32-unknown-unknown --release
# output: target/wasm32-unknown-unknown/release/my_subgraph.wasm
```

---

## 9. Deploy

**Local graph-node:**

```bash
graphite deploy --node http://localhost:8020 --ipfs http://localhost:5001 myname/my-subgraph
```

**The Graph Studio:**

1. Create a subgraph at [studio.thegraph.com](https://thegraph.com/studio/) and copy your deploy key.
2. Run:

```bash
graphite deploy \
  --node https://api.studio.thegraph.com/deploy/ \
  --ipfs https://api.thegraph.com/ipfs/ \
  --deploy-key <YOUR_DEPLOY_KEY> \
  --version-label v1.0.0 \
  your-subgraph-slug
```

The CLI uploads the WASM, schema, and ABI to IPFS, rewrites the manifest with IPFS hashes, then calls the graph-node JSON-RPC `subgraph_deploy` endpoint. On success it prints the playground and query URLs.

---

## What's Next

- [examples/erc20](../examples/erc20/) — full ERC20 reference, live on The Graph Studio (Arbitrum One).
- [examples/erc721](../examples/erc721/) — NFT transfer indexing, also live on Studio.
- For anything the CLI doesn't cover, `cargo build` and `graphite deploy` can be used independently.
