# Graphite

A Rust SDK for writing subgraphs on [The Graph](https://thegraph.com/) — no graph-node changes required.

**Status:** Phase 1 in progress (`graph-as-runtime` AS ABI layer).

## What Is It?

Graphite lets you write Graph subgraph mappings in Rust instead of AssemblyScript. The compiled WASM is AS-ABI-compatible, so it runs on any standard graph-node installation without modification — graph-node sees it as a normal subgraph.

AssemblyScript subgraphs suffer from broken nullable handling, missing closures, opaque compiler crashes, and a hostile debugging experience. Graphite fixes all of that:

- **Type safety** — `Option<T>` instead of runtime null crashes
- **Ergonomic APIs** — `#[derive(Entity)]` and `#[handler]` macros
- **Native testing** — `cargo test` with `MockHost`, no PostgreSQL required
- **Rust ecosystem** — iterators, closures, and the full crates.io

## How It Works

Graphite compiles to WASM that speaks the AssemblyScript object layout graph-node expects. The key facts:

- graph-node only reads `rtId` and `rtSize` from AS object headers — the GC fields can be zero.
- Strings are UTF-16LE. Entities are `TypedMap` objects. Graphite handles all of this transparently.
- Host functions (`store.set`, `log.log`, etc.) are matched by name — Rust can import them directly.
- The manifest declares `language: wasm/assemblyscript`, `apiVersion: 0.0.6` — graph-node accepts it without any special casing.

```
Your Rust handler
      │
      ▼
graphite-macros (#[handler], #[derive(Entity)])
      │
      ▼
graph-as-runtime (AS ABI layer: allocator, UTF-16LE strings, TypedMap, host imports)
      │
      ▼
WASM binary  ──────────────────►  unmodified graph-node
```

## Crate Structure

```
graphite/
├── graphite/           # Core SDK — primitives, host trait, MockHost
├── graphite-macros/    # Proc macros — #[derive(Entity)], #[handler]
├── graph-as-runtime/   # AS ABI layer (Phase 1, in progress)
└── graphite-cli/       # CLI — graphite init, codegen, build, deploy
```

## API Preview

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
    transfer.save();
}
```

The `#[handler]` macro generates the `extern "C"` entry point that graph-node calls. In native tests the same code runs with `MockHost` — no Docker, no PostgreSQL.

## CLI

```bash
graphite init my-subgraph --network mainnet
graphite codegen       # Generate Rust types from ABI + schema
graphite build         # Compile to WASM (wasm32-unknown-unknown)
graphite test          # Run tests (delegates to cargo test)
graphite deploy myname/mysubgraph  # Upload to IPFS + deploy via JSON-RPC
```

## Status

| Component | Status |
|-----------|--------|
| Core primitives (`BigInt`, `Address`, `Bytes`, etc.) | Done |
| `HostFunctions` trait + `MockHost` | Done |
| `#[derive(Entity)]` + `#[handler]` macro structure | Done |
| ABI + schema codegen | Done |
| CLI (`init`, `codegen`, `build`, `test`, `deploy`) | Done |
| `graph-as-runtime` AS ABI layer | **In progress (Phase 1)** |
| AS-ABI handler entry point in macros | Phase 2 |
| Codegen updated for AS types | Phase 3 |
| Integration test on unmodified graph-node | Phase 4 |
| ERC20/ERC721 examples ported | Phase 5 |

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for the full breakdown.

## Building

```bash
# Run all tests (native — no WASM toolchain needed)
cargo test -p graphite -p graphite-macros

# Build an example to WASM (once Phase 2+ is complete)
rustup target add wasm32-unknown-unknown
cargo build -p erc20-subgraph --target wasm32-unknown-unknown --release
```

## License

MIT OR Apache-2.0
