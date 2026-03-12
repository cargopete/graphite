# Graphite

A Rust SDK for building subgraphs on [The Graph](https://thegraph.com/).

**Status:** Early development — requires graph-node modifications (see Architecture).

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

## CLI Usage

```bash
graphite init my-subgraph --network mainnet
graphite codegen      # Generate Rust types from ABI + schema
graphite build        # Compile to WASM
graphite test         # Run tests (delegates to cargo test)
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

## Status

### SDK (this repo) — complete

- [x] Core primitives (BigInt, Address, Bytes)
- [x] `HostFunctions` trait + `MockHost` for native testing
- [x] `#[derive(Entity)]` macro with load/save/remove
- [x] `#[handler]` macro with WASM wrapper generation
- [x] `FromWasmBytes` trait for TLV event deserialization
- [x] ABI → Rust event struct codegen with `EventDecode` + `FromWasmBytes`
- [x] Schema.graphql → Entity struct codegen
- [x] CLI: `init`, `codegen`, `build`, `test`
- [x] WASM ABI layer (Rust-native ptr+len protocol)
- [x] no_std WASM support with alloc

### Graph-node modifications — in progress

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for the detailed plan.

Files created in graph-node fork (`runtime/wasm/src/rust_abi/`):
- [x] `mod.rs` — MappingLanguage enum
- [x] `types.rs` — `ToRustWasm`/`FromRustWasm` traits + primitives
- [x] `entity.rs` — Entity TLV serialization
- [x] `host.rs` — Linker function stubs (needs async integration)

Remaining work:
- [ ] Parse `language: wasm/rust` in manifest
- [ ] Wire up dispatch based on MappingLanguage
- [ ] Complete async store operation integration
- [ ] Gas metering integration
- [ ] Integration testing with real subgraph

## License

MIT OR Apache-2.0
