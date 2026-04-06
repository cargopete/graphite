# Graphite

A Rust SDK for building subgraphs on [The Graph](https://thegraph.com/).

**Status:** Fully functional. Graph-node draft PR [#6462](https://github.com/graphprotocol/graph-node/pull/6462) open. Live-tested against USDC Transfer events on Ethereum mainnet.

## Why Graphite?

AssemblyScript subgraphs suffer from broken nullable handling, missing closures, opaque compiler crashes, and a hostile debugging experience. Graphite provides a proper Rust alternative:

- **Type safety** — `Option<T>` instead of runtime null crashes
- **Ergonomic APIs** — `#[derive(Entity)]` and `#[handler]` macros
- **Native testing** — `cargo test` with `MockHost`, no PostgreSQL required
- **Rust ecosystem** — iterators, closures, and crates that actually work
- **~2× performance** — Rust WASM is faster than AssemblyScript (see [benchmarks](./benchmarks/))

## Architecture

Graphite defines a clean Rust-native ABI that runs alongside the existing AssemblyScript path in graph-node — selected by `mapping.kind: wasm/rust` in the subgraph manifest. The `HostExports` layer (store, crypto, IPFS) is unchanged; only the serialization boundary is replaced.

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
    │  (existing code)  │             │  rust_abi/ in graph-node │
    │  AscPtr<T>        │             │  ptr+len, TLV serde      │
    └───────────────────┘             └──────────────────────────┘
```

See [docs/rust-abi-spec.md](https://github.com/cargopete/graph-node/blob/rust-abi-support/docs/rust-abi-spec.md) in the graph-node fork for the full wire-format spec.

## Crate Structure

```
graphite/
├── graphite/           # Core SDK — primitives, host traits, TLV decode, MockHost
├── graphite-macros/    # Proc macros — #[derive(Entity)], #[handler]
└── graphite-cli/       # CLI — graphite init, codegen, build, deploy
```

## Quick Start

See [docs/getting-started.md](./docs/getting-started.md) for a full walkthrough.

```rust
use graphite::prelude::*;

#[derive(Entity)]
pub struct Transfer {
    #[id]
    id: String,
    from: Bytes,
    to: Bytes,
    value: BigInt,
}

#[handler]
pub fn handle_transfer(event: TransferEvent) {
    let mut transfer = Transfer::new(&event.id());
    transfer.from = Bytes::from_slice(event.from.as_slice());
    transfer.to = Bytes::from_slice(event.to.as_slice());
    transfer.value = event.value.clone();
    transfer.save(host);
}
```

The `#[handler]` macro generates the `extern "C"` WASM entry point, reads the serialized trigger from graph-node, deserialises it, constructs a `WasmHost`, and calls your function. In native tests the same code runs with `MockHost`.

## CLI

```bash
graphite init my-subgraph --network mainnet
graphite codegen       # Generate Rust types from ABI + schema
graphite build         # Compile to WASM (wasm32-unknown-unknown)
graphite test          # Run tests (delegates to cargo test)
graphite deploy myname/mysubgraph  # Upload to IPFS + deploy via JSON-RPC
```

## Examples

| Example | Description |
|---------|-------------|
| [`examples/erc20`](./examples/erc20/) | Basic ERC20 Transfer indexer — shows entity creation, field mapping, and `save` |
| [`examples/erc721`](./examples/erc721/) | ERC721 NFT tracker — shows multiple handlers, `store_get` load-and-update, and the mint/burn zero-address pattern |

## Building and Testing

```bash
# Run all tests (native, no WASM toolchain needed)
cargo test

# Run the WASM integration test (requires wasm32 target)
rustup target add wasm32-unknown-unknown
cargo test -p integration

# Build an example to WASM
cargo build -p erc20-subgraph --target wasm32-unknown-unknown --release
```

## Status

*Last updated: 2026-04-06*

### SDK

| Component | Status |
|-----------|--------|
| Core primitives (`BigInt`, `Address`, `Bytes`, etc.) | Done |
| `HostFunctions` trait + `WasmHost` FFI + `MockHost` | Done |
| `#[derive(Entity)]` + `#[handler]` macros | Done |
| TLV deserialization (`FromWasmBytes`, `TlvReader`) | Done |
| 51 TLV unit tests + proptest roundtrip suite | Done |
| Six TLV decode bugs found and fixed | Done |
| ABI + schema codegen | Done |
| CLI (`init`, `codegen`, `build`, `test`, `deploy`) | Done |
| Panic handler (forwards to graph-node `abort`) | Done |
| Allocator bounds checking (4 MiB limit) | Done |
| Decode error logging (type + handler + error details) | Done |
| Binary size optimised: 107 KB → 57 KB raw | Done |
| ERC20 example subgraph | Done |
| ERC721 example subgraph | Done |

### Graph-node fork

Fork at [`cargopete/graph-node`](https://github.com/cargopete/graph-node), branch `rust-abi-support`. Draft PR [#6462](https://github.com/graphprotocol/graph-node/pull/6462).

| Component | Status |
|-----------|--------|
| `rust_abi/` module (~1,450 LOC) — types, entities, triggers, host functions | Done |
| Manifest parsing (`wasm/rust` detection, language dispatch) | Done |
| Rust handler invocation (ptr+len calling convention, arena reset) | Done |
| Skip AS-specific exports (`id_of_type`, `_start`, parity_wasm) | Done |
| Wasmtime fuel metering (10B budget, `Trap::OutOfFuel`) | Done |
| TLV tag bytes extracted to named constants (`tags::*`) | Done |
| 12 graph-node unit tests + ABI test vectors for cross-validation | Done |
| NEAR trigger stub with `0xFF` sentinel (documented behaviour) | Done |
| Formal ABI spec (`docs/rust-abi-spec.md`) | Done |
| Live integration test (USDC Transfers from Ethereum mainnet) | Done |

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for the detailed breakdown.

## Links

- **Graph-node draft PR:** [graphprotocol/graph-node#6462](https://github.com/graphprotocol/graph-node/pull/6462)
- **ABI spec:** [docs/rust-abi-spec.md](https://github.com/cargopete/graph-node/blob/rust-abi-support/docs/rust-abi-spec.md)
- **RFC / GRC draft:** [GRC-draft.md](./GRC-draft.md)

## License

MIT OR Apache-2.0
