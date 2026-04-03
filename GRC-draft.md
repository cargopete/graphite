**Stage:** RFC (Request for Comment)
**Authors:** [@cargopete](https://github.com/cargopete)
**Related:** [graph-node PR #6462](https://github.com/graphprotocol/graph-node/pull/6462) · [Graphite SDK](https://github.com/cargopete/graphite)

## Summary

This GRC proposes adding first-class Rust support to graph-node as a second mapping language alongside AssemblyScript. Subgraph developers would write handlers in Rust, compile to `wasm32-unknown-unknown`, and declare `language: wasm/rust` in their manifest. Graph-node dispatches to a new `rust_abi` serialization layer; the existing `HostExports<C>` layer is unchanged.

A working implementation exists: the Graphite SDK compiles ERC20 handlers to WASM, and the graph-node fork has been live-tested indexing real USDC Transfer events from Ethereum mainnet via GraphQL.



## Motivation

AssemblyScript's pain points are well-documented in the community, but they are structural — not fixable at the SDK layer:

**Nullable handling crashes at runtime.** AS type narrowing only works on local variables. Accessing a nullable property directly compiles without error but produces a WASM trap at runtime. The workaround — assigning every nullable field to a local before use — turns three lines into eight. Rust's `Option<T>` with `match` and `map` eliminates this entire class of bug at compile time.

**No closures, no functional programming.** `array.map()`, `.filter()`, and `.forEach()` all crash the AS compiler. Developers fall back to C-style index loops. Rust's iterator chains are transformational for handler code that manipulates entity arrays and event parameter lists.

**Hostile debugging.** When the AS compiler crashes on basic operations (null-checking `event.transaction.to` is a known trigger), the official guidance from maintainers is to "comment the whole file and uncomment it little by little." Rust's compiler gives actionable errors pointing directly to the problem.

Additional issues compound: no `Date` support (developers write 50-line timestamp parsers), no regex, `===` performs identity comparison not value equality, array entity fields cannot be mutated in place, and Matchstick (the AS testing framework) requires a local PostgreSQL installation and produces silent type-mismatch failures.

**The Graph has already validated this path.** Substreams uses Rust compiled to WASM and is described by the team as enabling "extremely high-performance indexing." This proposal extends that same foundation to the subgraph layer.



## Design

### Key insight: the coupling is only in the serialization layer

`HostExports<C>` is already fully language-agnostic. It operates on native Rust types: `String`, `HashMap<String, Value>`, `Vec<u8>`. The AS coupling lives entirely in `runtime/wasm/src/asc_abi/` — the code that serialises data in and out of AS memory. Adding Rust support means adding a parallel `rust_abi/` module and dispatching on `language:` in the manifest.

```
                    ┌─────────────────────────┐
                    │    HostExports<C>        │  ← Unchanged
                    │  (store, crypto, ipfs)   │
                    └────────────┬────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              │                                      │
    ┌─────────┴─────────┐              ┌─────────────┴────────────┐
    │  asc_abi/          │              │  rust_abi/  (new)        │
    │  AscPtr<T>         │              │  ptr+len, TLV serde      │
    └────────────────────┘              └──────────────────────────┘
```

### ABI protocol

Graph-node calls Rust handlers with a simple C calling convention:

```
extern "C" fn handle_transfer(event_ptr: u32, event_len: u32) -> u32;
```

Serialization uses a simple TLV (Type-Length-Value) format. Graph-node writes serialised event data into the module's memory (via an exported `allocate(size) -> ptr`), calls the handler, then calls `reset_arena()` to return the bump allocator to its initial state. No managed heap, no garbage collection, no graph-node reaching into AS-style TypedMaps.

### What we tried to learn from the ASC ABI

- **No managed heap.** The module owns a bump allocator. Graph-node writes data, the module reads it, arena resets after the handler. No `allocate`-on-behalf-of-the-runtime complexity.
- **Explicit TLV format.** Fixed, documented value tag table. No implicit type coercions. Endianness is specified (little-endian) in the protocol — the BigInt endianness bug that plagued early AS integrations is baked into the spec.
- **Language detection via manifest.** `language: wasm/rust` — clean dispatch, no heuristics.
- **Versioned from day one.** `apiVersion: 0.0.1` in the manifest gives an explicit migration path.

### Manifest (no other changes required from subgraph authors)

```yaml
mapping:
  kind: wasm/rust
  apiVersion: 0.0.1
  file: ./target/wasm32-unknown-unknown/release/my_subgraph.wasm
  entities:
    - Transfer
  eventHandlers:
    - event: Transfer(indexed address,indexed address,uint256)
      handler: handle_transfer
```



## Developer experience

A Rust ERC20 handler looks like this:

```rust
use graphite::prelude::*;

#[handler]
pub fn handle_transfer(event: TransferEvent) {
    let id = format!("{}-{}", event.tx_hash, event.log_index);
    let mut transfer = Transfer::new(&id);
    transfer.from = event.from;
    transfer.to = event.to;
    transfer.value = event.value.clone();
    transfer.block_number = event.block_number.clone();
    transfer.save();
}
```

Testing runs natively with `cargo test` — no WASM compilation, no PostgreSQL, no external binaries:

```rust
#[test]
fn transfer_creates_entity() {
    let mut host = MockHost::default();
    let event = TransferEvent { from: addr("0xaaaa..."), to: addr("0xbbbb..."), value: 1000u64.into(), .. };
    handle_transfer(&mut host, &event);
    assert_eq!(host.store.entity_count("Transfer"), 1);
}
```

| | AssemblyScript | Rust |
|---|---|---|
| Nullable safety | Runtime crashes | Compile-time `Option<T>` |
| Closures / iterators | Compiler crashes | Full support |
| Testing setup | Node.js + PostgreSQL + binary | `cargo test`, nothing else |
| Type errors | Silent runtime coercions | Compile-time |
| Debugging | Comment-driven bisect | Actionable compiler errors |
| Crate ecosystem | None | Full crates.io |
| WASM performance | Baseline | ~2× faster |



## Implementation status

This is not a design-only proposal. A complete implementation exists and has been live-tested.

**graph-node fork** (`cargopete/graph-node`, branch `rust-abi-support`):
- `rust_abi/` module (~1,450 LOC): types, entity serialization, trigger serialization, host function wrappers
- Manifest parsing (`wasm/rust` detection and language dispatch)
- Rust handler invocation (ptr+len calling convention, arena reset after each handler)
- Skip of AS-specific exports (`id_of_type`, `_start`, parity_wasm pipeline)
- Wasmtime fuel metering (10B instruction budget, `OutOfFuel` as a deterministic error)
- All existing tests passing

**Graphite SDK** (`cargopete/graphite`):
- `#[derive(Entity)]` and `#[handler]` proc macros
- TLV deserializer (`TlvReader`, `FromWasmBytes`)
- ABI + GraphQL schema codegen (generates typed event structs and entity structs)
- CLI: `init`, `codegen`, `build`, `test`, `deploy`
- Panic hook (forwards panic message + file + line to graph-node via `abort`)
- `MockHost` for native unit testing

**Live test:** deployed ERC20 subgraph to the graph-node fork, indexed real USDC Transfer events from Ethereum mainnet (block 24756400+), queried via GraphQL. All fields correct: `from`, `to`, `value`, `blockNumber`, `timestamp`, `transactionHash`.

Draft PR: [graphprotocol/graph-node#6462](https://github.com/graphprotocol/graph-node/pull/6462)



## Scope of graph-node changes

The change is additive. Existing AS subgraphs are unaffected.

| Area | Change |
|---|---|
| `runtime/wasm/src/rust_abi/` | **New** — ~1,450 LOC across 5 files |
| `graph/src/data_source/` | **Modified** — `MappingLanguage` enum, manifest parsing |
| `runtime/wasm/src/module/` | **Modified** — linker dispatch, handler invocation |
| `chain/ethereum/src/trigger.rs` | **Modified** — `ToRustBytes` for Log/Call/Block triggers |
| `HostExports<C>` | **Unchanged** |

A new host function costs one entry in `rust_abi/host.rs` (a thin ptr+len wrapper over the existing `HostExports` method). No changes to the AS path.



## Open questions for community feedback

1. **Namespace.** Host functions currently use `"graphite"` as the WASM import namespace. Should this be `"graph"` or something more official?

2. **API versioning.** Should `apiVersion: 0.0.1` be independent of the AS versioning scheme, or aligned with it?

3. **Shared constants.** TLV value tags are currently duplicated between graph-node and the SDK. Worth a shared `graph-abi` crate, or is the spec doc sufficient?

4. **NEAR and other chains.** Ethereum triggers are fully serialized. NEAR has a stub (`unimplemented!()`). Scope for initial merge: Ethereum-only, with other chains following the same pattern.

5. **Long-term ABI evolution.** If the TLV format needs to change, what's the migration story? `apiVersion` in the manifest is the current answer, but we'd welcome input on whether a more formal versioning mechanism is warranted.



## References

- [graph-node PR #6462](https://github.com/graphprotocol/graph-node/pull/6462)
- [Graphite SDK](https://github.com/cargopete/graphite)
- [RFC: Designing a native Rust subgraph experience](https://github.com/cargopete/graphite/blob/main/rfc-rust-subgraph.md)
- [Substreams documentation](https://thegraph.com/docs/en/substreams/) — prior art for Rust→WASM in The Graph ecosystem
