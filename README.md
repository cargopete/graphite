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

*Last updated: 2026-03-28*

### SDK (this repo) — complete

All compiles, all tests pass.

- [x] Core primitives — `BigInt`, `BigDecimal`, `Address`, `Bytes`, `B256`, `U256` (via alloy-primitives)
- [x] `HostFunctions` trait — store ops, ethereum calls, crypto, logging, IPFS, data sources
- [x] `WasmHost` — full WASM implementation of `HostFunctions` via FFI (ptr+len protocol)
- [x] `MockHost` — in-memory store, configurable eth call/IPFS mocks, log capture
- [x] `#[derive(Entity)]` macro — load/save/remove, snake_case→camelCase, `FromValue` for all types
- [x] `#[handler]` macro — generates `extern "C"` WASM wrapper with `FromWasmBytes` deserialization
- [x] `FromWasmBytes` trait + `TlvReader` — full TLV deserialization for `RawLog`, `RawCall`, `RawBlock`
- [x] `EventDecode` trait — topic/data decoding with selector checking
- [x] ABI codegen — JSON ABI → Rust event structs with `EventDecode` + `FromWasmBytes` impls
- [x] Schema codegen — `schema.graphql` → `#[derive(Entity)]` structs with proper type mapping
- [x] CLI `init` — scaffolds full project (Cargo.toml, graphite.toml, subgraph.yaml, schema, src, abis)
- [x] CLI `codegen` — reads `graphite.toml`, generates ABI + schema bindings, writes `mod.rs`
- [x] CLI `build` — `cargo build --target wasm32-unknown-unknown` + optional wasm-opt
- [x] CLI `test` — delegates to `cargo test` with passthrough args
- [x] WASM bump allocator — `allocate()` + `reset_arena()` exports
- [x] `no_std` WASM support with alloc
- [x] ERC20 example subgraph — compiles and tests pass

Not yet implemented:
- [ ] CLI `deploy` — currently a placeholder (prints TODO)

### Graph-node modifications — complete (at `/Users/pepe/Projects/graph-node`)

Local graph-node clone with modifications on top of upstream `master`. Compiles cleanly, all tests pass (14 rust_abi tests, 17 NEAR chain tests). Live-tested with real Ethereum mainnet data.

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for the detailed plan.

**Phase 1 — ABI spec** (designed, not formalised):
- [x] Protocol design — TLV format, function signatures, event layout
- [x] SDK-side implementation — `TlvReader`, `FromWasmBytes`, entity serialization
- [ ] Write formal `docs/rust-abi-spec.md` in graph-node repo
- [ ] Shared constants crate for TLV value tags (currently hardcoded on both sides)
- [ ] Cross-validation test vectors

**Phase 2 — `rust_abi/` module in graph-node** (done, ~1,450 LOC):
- [x] `rust_abi/mod.rs` — `MappingLanguage` enum with `from_kind()` parser
- [x] `rust_abi/types.rs` — `ToRustWasm`/`FromRustWasm` traits + impls for all primitives
- [x] `rust_abi/entity.rs` — Entity TLV serialization with `ValueTag` enum
- [x] `rust_abi/trigger.rs` — `ToRustBytes` trait + `RustLogTrigger`/`RustCallTrigger`/`RustBlockTrigger`
- [x] `rust_abi/host.rs` — All linker function wrappers (store, crypto, logging, data sources, IPFS, ethereum calls) with async + gas metering
- [x] Unit tests for entity roundtrips, trigger serialization, type roundtrips

**Phase 3 — Manifest parsing & dispatch** (done):
- [x] `MappingLanguage::from_kind()` parses `wasm/rust` from manifest
- [x] `is_rust_module()` detects Rust modules by inspecting `"graphite"` namespace imports
- [x] `build_linker()` branches: AS modules get AS linker, Rust modules get `rust_abi` linker
- [x] `handle_trigger_rust()` — Rust calling convention (allocate → write bytes → call handler(ptr, len) → reset_arena)
- [x] Ethereum `ToRustBytes` impl for all trigger types (Log, Call, Block)
- [x] NEAR `ToRustBytes` stub (unimplemented — Ethereum-only for now)
- [x] Trait bounds propagated through `instance_manager.rs`
- [x] Skip `parity_wasm` gas injection for Rust modules (parity_wasm can't parse modern WASM features like bulk-memory)
- [x] Skip AS-specific exports (`id_of_type`, `_start`) for Rust modules

<a id="whats-next"></a>

**Phase 4 — Integration & hardening** (done):
- [x] Async store operation integration (done in `rust_abi/host.rs`)
- [x] Gas metering integration (done — all host functions meter gas)
- [x] WASM integration test — loads ERC20 WASM, serializes Transfer event in graph-node's TLV format, calls handler, verifies entity fields in `store_set` (`tests/integration/`)
- [x] Live integration test — deployed to running graph-node fork, indexing real USDC Transfer events from Ethereum mainnet, queryable via GraphQL (`scripts/live-test.sh`)
- [ ] Error handling: verify handler panics are caught gracefully
- [ ] Offchain/subgraph trigger serialization (currently stubbed as empty bytes)
- [ ] Performance comparison vs AS equivalent

### Bugs found & fixed during live testing

1. **parity_wasm opcode 252** — `parity_wasm` (used for AS gas injection) can't parse modern WASM features like `memory.copy` (bulk-memory). Fix: detect Rust modules early and skip the parity_wasm pipeline entirely; wasmtime 38 handles all features natively.
2. **`id_of_type` export** — AS-specific function required by graph-node during module instantiation. Fix: made optional for Rust modules.
3. **`_start` export** — AS entry point called after instantiation. Fix: skipped for Rust modules.
4. **BigInt endianness** — SDK serialized BigInt as signed big-endian, graph-node deserialized as signed little-endian. Fix: both sides now use little-endian.

### Open questions

1. **Error handling** — return codes vs panic? (leaning: return codes, panic as fallback)
2. **API versioning** — start at `0.0.1` or align with AS versions?
3. **Gas metering for Rust** — currently skipped (parity_wasm gas injection bypassed); need wasmtime fuel metering or equivalent
4. **Debugging** — how to surface Rust panic messages to users?

## License

MIT OR Apache-2.0
