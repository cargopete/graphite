# Graphite

A Rust SDK for building subgraphs on [The Graph](https://thegraph.com/).

**Status:** Early development — not yet ready for production use.

## Why Graphite?

AssemblyScript subgraphs suffer from broken nullable handling, missing closures, opaque compiler crashes, and a hostile debugging experience. Graphite aims to provide a proper Rust alternative with:

- **Type safety** — `Option<T>` instead of runtime null crashes
- **Ergonomic APIs** — `#[derive(Entity)]` and `#[handler]` macros
- **Native testing** — `cargo test` with mock host functions, no PostgreSQL required
- **Rust ecosystem** — iterators, closures, and crates that actually work
- **~2× performance** — Rust WASM is faster than AssemblyScript

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
pub fn handle_transfer(event: TransferEvent) {
    let mut transfer = Transfer::new(&event.id());
    transfer.from = event.params.from;
    transfer.to = event.params.to;
    transfer.value = event.params.value.into();
    transfer.save();
}
```

## Testing

Handlers run natively with `MockHost` — no WASM compilation needed:

```rust
#[test]
fn transfer_creates_entity() {
    let mut host = MockHost::default();

    let event = TransferEvent::mock()
        .from(addr("0xaaaa..."))
        .to(addr("0xbbbb..."))
        .value(1000u64)
        .build();

    handle_transfer(&mut host, &event);

    assert_eq!(host.store.entity_count("Transfer"), 1);
}
```

## CLI Usage

```bash
graphite init my-subgraph --from-contract 0x... --network mainnet
graphite codegen      # Generate Rust types from ABI + schema
graphite build        # Compile to WASM
graphite test         # Run tests (delegates to cargo test)
graphite deploy       # Deploy to graph-node
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

Then run `graphite codegen` to generate:
- Entity structs from your GraphQL schema
- Event structs from contract ABIs

## Roadmap

- [x] Core primitives (BigInt, Address, Bytes)
- [x] HostFunctions trait + MockHost for testing
- [x] `#[derive(Entity)]` macro with `FromValue` trait
- [x] ABI → Rust event struct codegen
- [x] Schema.graphql → Entity struct codegen
- [x] CLI with `graphite codegen` command
- [x] Event decoding (`EventDecode` trait + decode helpers)
- [x] WASM ABI layer (FFI to graph-node)
- [x] `graphite init` scaffolding
- [x] `graphite build` WASM compilation (with wasm-opt if available)
- [ ] Full AS memory marshalling (Entity serialization)
- [ ] Integration testing with graph-node
- [ ] Documentation

## Design

See [rfc-rust-subgraph.md](./rfc-rust-subgraph.md) for the full design document.

## License

MIT OR Apache-2.0
