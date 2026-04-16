# Your First Subgraph

This walkthrough builds an ERC20 Transfer indexer from scratch — the same one that's live on The Graph Studio (Arbitrum One).

---

## 1. Create the Project

```bash
graphite init my-subgraph --network mainnet
cd my-subgraph
```

If you have an Etherscan API key, pass the contract address and the CLI fetches the ABI for you:

```bash
ETHERSCAN_API_KEY=yourkey graphite init my-subgraph \
  --from-contract 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  --network mainnet
```

---

## 2. Define the Schema

Edit `schema.graphql`:

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

Each `@entity` type becomes a generated Rust struct with a builder.

---

## 3. Add the ABI

Drop the ERC20 ABI JSON into `abis/`:

```bash
cp path/to/ERC20.json abis/ERC20.json
```

The ABI must be standard Ethereum JSON format. At minimum you need the Transfer event:

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

## 4. Configure graphite.toml

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

---

## 5. Generate the Manifest

```bash
graphite manifest
```

This reads `graphite.toml` and `schema.graphql` and writes `subgraph.yaml`. Re-run it whenever you change the config.

---

## 6. Run Codegen

```bash
graphite codegen
```

This generates Rust source into `src/generated/`:

```
src/generated/
├── mod.rs      # re-exports
├── erc20.rs    # ERC20TransferEvent and other event/call structs
└── schema.rs   # Transfer entity builder
```

The generated `ERC20TransferEvent` has typed fields (`from: [u8; 20]`, `to: [u8; 20]`, `value: BigInt`) decoded from raw ABI bytes. The `Transfer` entity has setter methods for every schema field.

---

## 7. Write the Handler

Edit `src/lib.rs`:

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

The `#[handler]` macro generates two things:
- `handle_transfer_impl(event, ctx)` — the logic function you call from tests.
- `handle_transfer(event_ptr: i32)` — the `extern "C"` WASM entry point graph-node calls.

In `subgraph.yaml`, the handler name is `handle_transfer`.

> **Note:** `#![cfg_attr(target_arch = "wasm32", no_std)]` + `extern crate alloc` is required. When targeting WASM, the standard library is unavailable. Use `alloc::format!`, `alloc::vec!`, `alloc::string::String`, and so on.

---

## 8. Test Natively

```bash
cargo test
```

No Docker, no PostgreSQL, no graph-node. Tests use an in-process mock store:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use graph_as_runtime::ethereum::{EthereumValue, EventParam, FromRawEvent, RawEthereumEvent};
    use graphite::mock;

    fn mock_event() -> RawEthereumEvent {
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

        let raw = mock_event();
        let event = ERC20TransferEvent::from_raw_event(&raw).unwrap();
        handle_transfer_impl(&event, &graphite::EventContext::default());

        let tx_hex = "ab".repeat(32);
        assert!(mock::has_entity("Transfer", &format!("{}-00", tx_hex)));
    }
}
```

See [Testing](../concepts/testing.md) for the full mock API.

---

## 9. Build

```bash
graphite build
```

Compiles to WASM and runs `wasm-opt`. Output: `build/my-subgraph.wasm`.

---

## 10. Deploy

**The Graph Studio:**

```bash
graphite deploy \
  --node https://api.studio.thegraph.com/deploy/ \
  --ipfs https://api.thegraph.com/ipfs/ \
  --deploy-key YOUR_DEPLOY_KEY \
  --version-label v1.0.0 \
  your-subgraph-slug
```

**Local graph-node:**

```bash
graphite deploy \
  --node http://localhost:8020 \
  --ipfs http://localhost:5001 \
  myname/my-subgraph
```

The CLI uploads the WASM, schema, and ABIs to IPFS, rewrites `subgraph.yaml` with IPFS hashes, then calls `subgraph_deploy` on the graph-node JSON-RPC endpoint.
