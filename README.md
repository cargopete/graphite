# Graphite

A Rust SDK for building subgraphs on [The Graph](https://thegraph.com/).

**Status:** SDK complete, graph-node fork live-tested — indexing real USDC Transfer events from Ethereum mainnet via GraphQL (see [What's Next](#whats-next)).

## Why Graphite?

AssemblyScript subgraphs suffer from broken nullable handling, missing closures, opaque compiler crashes, and a hostile debugging experience. Graphite provides a proper Rust alternative with:

- **Type safety** — `Option<T>` instead of runtime null crashes
- **Ergonomic APIs** — `#[derive(Entity)]` and `#[handler]` macros
- **Native testing** — `cargo test` with mock host functions, no PostgreSQL required
- **Rust ecosystem** — iterators, closures, and crates that actually work
- **~2× performance** — Rust WASM is faster than AssemblyScript

## Architecture

Graphite does **not** try to conform to AssemblyScript's memory layout. Instead, it defines a clean Rust-native ABI that requires modifications to graph-node:

```
                    ┌─────────────────────────┐
                    │    HostExports<C>       │  ← Language-agnostic (unchanged)
                    │  (store, crypto, ipfs)  │
                    └────────────┬────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              │                                     │
    ┌─────────┴─────────┐             ┌─────────────┴────────────┐
    │  AscAbiHost       │             │  RustAbiHost             │
    │  (current code)   │             │  (new, in graph-node)    │
    │  AscPtr<T>        │             │  ptr+len, simple serde   │
    └───────────────────┘             └──────────────────────────┘
```

**Graph-node changes required:**
1. Detect `language: wasm/rust` in subgraph.yaml manifest
2. Add `RustAbiHost` with ptr+len FFI protocol
3. Use simple serialization (not AS TypedMap)

See [rfc-rust-subgraph.md](./rfc-rust-subgraph.md) for the full design.

## Project Structure

```
graphite/
├── graphite/           # Core SDK crate
├── graphite-macros/    # Proc macros (#[derive(Entity)], #[handler])
└── graphite-cli/       # CLI tooling (graphite init, codegen, build, deploy)
```

## Quick Start

```rust
use graphite::prelude::*;

#[derive(Entity)]
pub struct Transfer {
    #[id]
    id: String,
    from: Address,
    to: Address,
    value: BigInt,
}

#[handler]
pub fn handle_transfer(host: &mut impl HostFunctions, event: &TransferEvent) {
    let mut transfer = Transfer::new(&event.id());
    transfer.from = event.from;
    transfer.to = event.to;
    transfer.value = event.value.clone();
    transfer.save(host);
}
```

## Testing (works today)

Handlers run natively with `MockHost` — no WASM, no graph-node needed:

```rust
#[test]
fn transfer_creates_entity() {
    let mut host = MockHost::default();

    let event = TransferEvent { /* ... */ };
    handle_transfer(&mut host, &event);

    assert_eq!(host.store.entity_count("Transfer"), 1);
}
```

## Configuration

Create a `graphite.toml` in your project root:

```toml
output_dir = "src/generated"
schema = "schema.graphql"

[[contracts]]
name = "ERC20"
abi = "abis/ERC20.json"
```

## CLI Usage

```bash
graphite init my-subgraph --network mainnet
graphite codegen       # Generate Rust types from ABI + schema
graphite build         # Compile to WASM
graphite test          # Run tests (delegates to cargo test)
graphite deploy myname/mysubgraph  # Deploy to graph-node (IPFS + JSON-RPC)
```

## Status

*Last updated: 2026-03-29*

### SDK — complete

All compiles, all tests pass. Graph-node draft PR: [#6462](https://github.com/graphprotocol/graph-node/pull/6462).

| Component | Status |
|-----------|--------|
| Core primitives (`BigInt`, `Address`, `Bytes`, etc.) | Done |
| `HostFunctions` trait + `WasmHost` FFI + `MockHost` | Done |
| `#[derive(Entity)]` + `#[handler]` macros | Done |
| TLV deserialization (`FromWasmBytes`, `TlvReader`) | Done |
| ABI + schema codegen | Done |
| CLI (`init`, `codegen`, `build`, `test`, `deploy`) | Done |
| Panic handler (forwards to graph-node `abort`) | Done |
| Allocator bounds checking (4MB limit) | Done |
| Decode error logging (type + handler + error details) | Done |
| ERC20 example subgraph | Done |

### Graph-node fork — complete

Fork at [`cargopete/graph-node`](https://github.com/cargopete/graph-node), branch `rust-abi-support`. Draft PR [#6462](https://github.com/graphprotocol/graph-node/pull/6462).

| Component | Status |
|-----------|--------|
| `rust_abi/` module (~1,450 LOC) — types, entities, triggers, host functions | Done |
| Manifest parsing (`wasm/rust` detection, language dispatch) | Done |
| Rust handler invocation (ptr+len calling convention, arena reset) | Done |
| Skip AS-specific exports (`id_of_type`, `_start`, parity_wasm) | Done |
| Wasmtime fuel metering (10B budget, `Trap::OutOfFuel`) | Done |
| Live integration test (USDC Transfers from Ethereum mainnet) | Done |

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for detailed breakdown.

<a id="whats-next"></a>

### What's next

1. **Formal ABI spec** — `docs/rust-abi-spec.md` for the upstream PR
2. **Performance comparison** — Rust vs AS benchmark
3. **Address PR review feedback** on [#6462](https://github.com/graphprotocol/graph-node/pull/6462)

## License

MIT OR Apache-2.0
