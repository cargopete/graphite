# Graphite — Implementation Plan

## Goal

Produce AssemblyScript-ABI-compatible WASM from Rust that runs on **unmodified graph-node**.

No graph-node fork. No custom protocol. No PR required. The output WASM is transparent to graph-node — it looks and behaves like any ordinary AssemblyScript subgraph.

## Key Insight

graph-node's AS runtime makes a small set of well-defined assumptions about WASM memory layout:

- Every heap object starts with a 20-byte header. graph-node only reads `rtId` (offset 12) and `rtSize` (offset 16). The GC fields (`mmInfo`, `gcInfo`, `gcInfo2`) can be zero — graph-node never inspects them.
- Strings are UTF-16LE with class ID 2.
- Host function imports are matched by **name only** — the signature is not verified. We can declare exactly the signatures we need.
- The manifest declares `apiVersion: 0.0.6` and `language: wasm/assemblyscript` — graph-node accepts it as a normal subgraph without any special casing.

This means a Rust no_std WASM that honours these layout conventions runs on stock graph-node with zero changes on the graph-node side.

## What Is Kept vs Rewritten

| Component | Status |
|-----------|--------|
| graphite-cli (init, build, deploy, codegen) | Kept |
| `#[handler]` / `#[derive(Entity)]` macro structure | Kept, internals rewritten |
| MockHost concept | Kept, internals rewritten |
| `graphite/src/wasm/` (TLV bindings) | **Deleted** |
| `graphite/src/decode.rs` (TLV reader) | **Deleted** |
| `tests/integration` (TLV integration test) | **Deleted** — rewrite in Phase 4 |
| `benchmarks/` (old approach benchmarks) | **Deleted** |
| `graph-as-runtime` | **New** — AS ABI layer |

---

## Phase 1 — `graph-as-runtime` (in progress)

Build the no_std AS ABI compatibility layer that all Rust subgraphs will depend on.

### Memory allocator

- Bump allocator backed by a static arena.
- Every allocation prefixes a 20-byte AS object header:
  - `mmInfo: u32` — can be zero (graph-node ignores it)
  - `gcInfo: u32` — can be zero
  - `gcInfo2: u32` — can be zero
  - `rtId: u32` — AS class ID (2 = String, 1 = ArrayBuffer, etc.)
  - `rtSize: u32` — payload size in bytes
- Exports: `__new(size: u32, id: u32) -> i32`, `__pin(ptr: i32) -> i32`, `__unpin(ptr: i32)`, `__collect()`.

### String encoding

- `AscString`: UTF-16LE encoded, class ID 2.
- `from_utf8(s: &str) -> AscPtr<AscString>` — allocates header + UTF-16LE payload.
- `to_utf8(ptr: AscPtr<AscString>) -> &str` — reads rtSize, decodes UTF-16LE.

### Core AS types

- `TypedArray<T>` (12 bytes): buffer pointer, data start, byte length — class IDs vary by element type.
- `Array<T>` (16 bytes): buffer pointer, data start, byte length, length — class ID 13.
- `TypedMap` (4 bytes): pointer to `Array<TypedMapEntry>` — class ID 17.
- `TypedMapEntry` (8 bytes): key pointer + value pointer.
- `AscEnum` / `Value` (8 bytes): discriminant i32 + payload i32/ptr.

### Host function imports

Declared as `extern "C"` with the exact names graph-node registers:

```
store.set(entity: i32, id: i32, data: i32)
store.get(entity: i32, id: i32) -> i32
store.remove(entity: i32, id: i32)
log.log(level: i32, msg: i32)
crypto.keccak256(input: i32) -> i32
ethereum.call(call: i32) -> i32
```

### EntityBuilder

Ergonomic builder for constructing the `TypedMap` object graph that `store.set` expects:

```rust
EntityBuilder::new()
    .set("id", Value::string("0xabc..."))
    .set("from", Value::bytes(&addr))
    .set("value", Value::bigint(&n))
    .build()  // -> AscPtr<TypedMap>
```

### Milestone

A single `store.set` call with one string field succeeds on an unmodified local graph-node instance.

---

## Phase 2 — Update `graphite-macros`

### `#[handler]`

graph-node calls handlers as `handle_transfer(eventPtr: i32)` — a single AscPtr argument, not the old `(ptr: u32, len: u32)` pair.

The generated `extern "C"` wrapper:
1. Receives `event_ptr: i32`.
2. Reads the AS `TypedMap` object at that address using `graph-as-runtime` accessors.
3. Constructs the typed Rust event struct.
4. Calls the user's handler body.

### `#[derive(Entity)]`

`save()` delegates to `graph-as-runtime::EntityBuilder` to produce a `TypedMap` and passes it to `store.set`.
`load()` reads the `TypedMap` returned by `store.get` and maps fields back to the Rust struct.

The native path (non-wasm32) continues to use `MockHost` with no AS memory involved.

---

## Phase 3 — Update Codegen

- **ABI → event structs**: field accessors read from AS `TypedMap` / `AscEnum` layout via `graph-as-runtime` rather than TLV bytes.
- **GraphQL schema → entity structs**: `save` / `load` delegates to the new `EntityBuilder`.
- Generated code targets the updated macro output — no hand-written glue needed.

---

## Phase 4 — Integration Test

- A full ERC20 Transfer handler compiles to WASM and is deployed to a local graph-node.
- All indexed event fields (from, to, value) are correct.
- Entities are queryable via GraphQL.
- `MockHost` updated for native unit testing of the same handlers.
- CI runs `cargo test` (native) and the WASM integration test against a local graph-node Docker image.

---

## Phase 5 — Polish

- ERC721 example ported to the new approach.
- Getting-started guide updated (no mention of TLV, no fork, no graph-node PR).
- CLI `build` / `deploy` commands verified end-to-end.
- Binary size audit — target < 100 KB for a minimal handler.
