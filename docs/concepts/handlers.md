# Handlers

Handlers are Rust functions that graph-node calls when an indexed event, call, or block occurs. The `#[handler]` macro generates the `extern "C"` WASM entry point and a testable `_impl` function from your Rust function.

## Handler Types

| Attribute | Triggered by | Signature |
|-----------|-------------|-----------|
| `#[handler]` | Ethereum event | `fn(event: &FooEvent, ctx: &EventContext)` |
| `#[handler(call)]` | Contract function call | `fn(call: &FooCall, ctx: &CallContext)` |
| `#[handler(block)]` | Every block | `fn(block: &EthereumBlock, ctx: &EventContext)` |
| `#[handler(file)]` | IPFS file content | `fn(content: &[u8], ctx: &FileContext)` |

## Event Handlers

The most common handler type. Fires for every matching event log.

```rust
#[handler]
pub fn handle_transfer(event: &ERC20TransferEvent, ctx: &graphite::EventContext) {
    let id = format!("{}-{}", hex(&ctx.tx_hash), hex(&ctx.log_index));
    Transfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_value(event.value.clone())
        .save();
}
```

The generated event struct (`ERC20TransferEvent`) has typed fields matching the ABI. Indexed parameters and non-indexed parameters are decoded automatically. See [Entities](entities.md) for the builder API.

## Call Handlers

Fires when a specific contract function is called (requires `call: true` in the graph-node config for the chain).

```rust
#[handler(call)]
pub fn handle_transfer_call(call: &ERC20TransferCall, ctx: &graphite::CallContext) {
    // call.to, call.value — decoded from the calldata
    // ctx.from, ctx.to, ctx.block_number, etc.
}
```

Declare in `graphite.toml`:

```toml
[[contracts.call_handlers]]
function = "transfer(address,uint256)"
handler  = "handle_transfer_call"
```

## Block Handlers

Fires for every block, or every N blocks if a filter is configured.

```rust
#[handler(block)]
pub fn handle_block(block: &graphite::EthereumBlock, ctx: &graphite::EventContext) {
    // ctx.block_number, ctx.block_timestamp, etc.
}
```

Declare in `graphite.toml`:

```toml
# Every block
[[contracts.block_handlers]]
handler = "handle_block"

# Every 10 blocks
[[contracts.block_handlers]]
handler = "handle_block_polled"
filter  = { kind = "polling", every = 10 }
```

## File Handlers

Fires when IPFS content is fetched for a file data source. See [File Handlers](../advanced/file-handlers.md).

```rust
#[handler(file)]
pub fn handle_metadata(content: &[u8], ctx: &graphite::FileContext) {
    // parse JSON or raw bytes from IPFS content
}
```

## The `_impl` Convention

The `#[handler]` macro generates two functions from `pub fn handle_foo(...)`:

- `pub fn handle_foo_impl(event, ctx)` — the actual logic, callable from tests.
- `extern "C" fn handle_foo(event_ptr: i32)` — the WASM entry point graph-node calls.

In `subgraph.yaml` the handler name is `handle_foo`. In tests you call `handle_foo_impl`.

```rust
#[test]
fn my_test() {
    mock::reset();
    handle_transfer_impl(&event, &graphite::EventContext::default());
    // assertions...
}
```

## no_std Requirement

Subgraph crates must be `no_std` when targeting WASM. The standard library is not available in the WASM environment.

```rust
#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

use alloc::{format, string::String, vec, vec::Vec};
```

This only applies to the `wasm32` target — native `cargo test` runs with the full standard library.
