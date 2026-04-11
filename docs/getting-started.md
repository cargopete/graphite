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

If you already know the contract address, pass it and the CLI will attempt to fetch the ABI from Etherscan (set `ETHERSCAN_API_KEY` in your environment):

```bash
ETHERSCAN_API_KEY=yourkey graphite init my-subgraph \
  --from-contract 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  --network mainnet
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

The ABI must be standard Ethereum JSON ABI format. For an ERC20 Transfer event:

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

Update `graphite.toml`:

```toml
output_dir = "src/generated"
schema     = "schema.graphql"
network    = "mainnet"

[[contracts]]
name        = "ERC20"
abi         = "abis/ERC20.json"
address     = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
start_block = 6082465
```

Then generate `subgraph.yaml` from it:

```bash
graphite manifest
```

This reads `graphite.toml` and `schema.graphql` and writes a complete `subgraph.yaml`. You can re-run it any time after editing the config.

---

## 5. Run Codegen

```bash
graphite codegen
```

This reads `graphite.toml` and generates Rust source into `src/generated/`:

```
src/generated/
├── mod.rs      # Re-exports everything
├── erc20.rs    # Event/call structs from the ABI (ERC20TransferEvent, etc.)
└── schema.rs   # Entity builders from schema.graphql (Transfer, etc.)
```

The generated `ERC20TransferEvent` struct has typed fields for each ABI parameter (`from`, `to`, `value`) decoded from raw ABI bytes.

---

## 6. Write Your Handler

Edit `src/lib.rs`. A complete ERC20 Transfer handler using the `#[handler]` macro:

```rust
#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

use alloc::format;
use graphite_macros::handler;

mod generated;
use generated::{ERC20TransferEvent, Transfer};

fn hex(b: &[u8]) -> alloc::string::String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

#[handler]
pub fn handle_transfer(event: &ERC20TransferEvent, ctx: &graphite::EventContext) {
    let id = format!("{}-{}", hex(&ctx.tx_hash), hex(&ctx.log_index));

    Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_value(event.value.clone())
        .set_block_number(ctx.block_number.clone())
        .set_timestamp(ctx.block_timestamp.clone())
        .set_transaction_hash(ctx.tx_hash.to_vec())
        .save();
}
```

The `#[handler]` macro expands this into:
- `handle_transfer_impl(event, ctx)` — the logic function, callable from tests.
- `handle_transfer(event_ptr: i32)` — the `extern "C"` WASM entry point that graph-node calls.

### Handler Types

| Attribute | Called for | Signature |
|-----------|-----------|-----------|
| `#[handler]` | Ethereum events | `fn handle_*(event: &FooEvent, ctx: &EventContext)` |
| `#[handler(call)]` | Contract function calls | `fn handle_*(call: &FooCall, ctx: &CallContext)` |
| `#[handler(block)]` | Every block | `fn handle_*(block: &EthereumBlock, ctx: &EventContext)` |
| `#[handler(file)]` | IPFS file content | `fn handle_*(content: &[u8], ctx: &FileContext)` |

### EventContext Fields

The `ctx` parameter carries the full block and transaction context:

```rust
pub struct EventContext {
    pub address:               [u8; 20],        // contract address
    pub log_index:             Vec<u8>,          // log index (LE BigInt bytes)
    pub block_hash:            [u8; 32],
    pub block_number:          Vec<u8>,          // LE BigInt bytes
    pub block_timestamp:       Vec<u8>,
    pub block_gas_used:        Vec<u8>,
    pub block_gas_limit:       Vec<u8>,
    pub block_difficulty:      Vec<u8>,
    pub block_base_fee_per_gas: Option<Vec<u8>>, // EIP-1559
    pub tx_hash:               [u8; 32],
    pub tx_index:              Vec<u8>,
    pub tx_from:               [u8; 20],
    pub tx_to:                 Option<[u8; 20]>,
    pub tx_value:              Vec<u8>,
    pub tx_gas_limit:          Vec<u8>,
    pub tx_gas_price:          Vec<u8>,
    pub tx_nonce:              Vec<u8>,
    pub receipt:               Option<TransactionReceipt>, // if `receipt: true` in manifest
}
```

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
    use graph_as_runtime::ethereum::{EthereumValue, EventParam, FromRawEvent, RawEthereumEvent};
    use graph_as_runtime::native_store;

    fn mock_raw() -> RawEthereumEvent {
        RawEthereumEvent {
            tx_hash: [0xab; 32],
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
        native_store::reset();

        let event = ERC20TransferEvent::from_raw_event(&mock_raw()).unwrap();
        let ctx = graphite::EventContext { tx_hash: [0xab; 32], ..Default::default() };
        handle_transfer_impl(&event, &ctx);

        assert_eq!(native_store::with_store(|s| s.entity_count("Transfer")), 1);
    }
}
```

`native_store::reset()` clears the in-memory store between tests. Use `native_store::with_store` to inspect results.

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

## Advanced Features

### Dynamic Data Sources (Factory Pattern)

Declare a template in `graphite.toml`:

```toml
[[templates]]
name = "Pair"
abi  = "abis/Pair.json"
```

In your factory handler, create a new data source instance:

```rust
use graphite::data_source;

#[handler]
pub fn handle_pair_created(event: &FactoryPairCreatedEvent, ctx: &graphite::EventContext) {
    let pair_addr = graphite::primitives::Address::from(event.pair);
    data_source::create(host, "Pair", pair_addr);
}
```

In the template handler, introspect the current data source:

```rust
#[handler]
pub fn handle_swap(event: &PairSwapEvent, ctx: &graphite::EventContext) {
    let addr   = data_source::address(host);
    let net    = data_source::network(host);
    let id_str = data_source::id(host);
    // ...
}
```

### Crypto Utilities

All crypto runs natively — no host calls, works in `cargo test`:

```rust
use graphite::crypto;

let hash = crypto::keccak256(b"hello");
let sha  = crypto::sha256(b"hello");
let sel  = crypto::selector("transfer(address,uint256)"); // → [0xa9, 0x05, 0x9c, 0xbb]
let addr = crypto::secp256k1_recover(&msg_hash, &r, &s, v);
```

### ABI Encoding

```rust
use graphite::ethereum::{self, EthereumValue};

let encoded = ethereum::encode(&EthereumValue::Uint(value_bytes)).unwrap();
```

### Logging

```rust
use graphite::{log_info, log_warning, nonfatal_error};

log_info!(host, "processing token {}", token_id);
nonfatal_error!(host, "unexpected zero address — skipping");
return;
```

---

## What's Next

- [examples/erc20](../examples/erc20/) — full ERC20 reference, live on The Graph Studio (Arbitrum One).
- [examples/erc721](../examples/erc721/) — NFT transfer + approval indexing.
- [examples/erc1155](../examples/erc1155/) — multi-token: TransferSingle, TransferBatch, URI.
- [examples/multi-source](../examples/multi-source/) — multiple contracts in one subgraph.
- [examples/file-ds](../examples/file-ds/) — IPFS file data source handler.
