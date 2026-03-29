# Rust Subgraph Implementation Plan

This document outlines the changes required to enable Rust subgraphs in The Graph ecosystem. It covers modifications to both graph-node and the Graphite SDK.

*Last updated: 2026-03-29*

---

## Overview

**Goal:** Enable `language: wasm/rust` subgraphs that compile Rust handlers to WASM and run on graph-node with a clean, Rust-native ABI.

**Key Insight:** graph-node's `host_exports.rs` is already language-agnostic. The AS coupling exists only in the serialization layer (`asc_abi/`). We add a parallel `rust_abi/` and dispatch based on manifest language.

### Progress Summary

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Define Rust ABI Protocol | **Designed** — protocol defined in this doc + implemented in SDK, formal spec doc not yet written |
| 2 | Implement rust_abi/ in graph-node | **Done** — ~1,450 LOC across 5 new files, all tests passing |
| 3 | Manifest parsing and dispatch | **Done** — language detection, linker dispatch, Rust calling convention |
| 4 | Complete Graphite SDK | **Done** — all SDK code implemented and compiling |
| 5 | Integration testing | **Done** — WASM integration test + live mainnet test passing |

**Graph-node fork:** `/Users/pepe/Projects/graph-node` (modifications on upstream `master`, compiles clean, live-tested)

---

## Phase 1: Define the Rust ABI Protocol

**Objective:** Specify the exact binary protocol between graph-node and Rust WASM modules.

### 1.1 Function Signatures

Graph-node will call Rust handlers with this signature:

```rust
// Exported by Rust WASM module
extern "C" fn handle_transfer(
    event_ptr: u32,    // Pointer to serialized event data
    event_len: u32,    // Length of event data
) -> u32;              // 0 = success, non-zero = error code
```

Graph-node imports from Rust WASM module:

```rust
// Memory management
extern "C" fn allocate(size: u32) -> u32;
extern "C" fn reset_arena();

// Host functions (graph-node provides these)
extern "C" fn store_set(
    entity_type_ptr: u32, entity_type_len: u32,
    id_ptr: u32, id_len: u32,
    data_ptr: u32, data_len: u32,
);

extern "C" fn store_get(
    entity_type_ptr: u32, entity_type_len: u32,
    id_ptr: u32, id_len: u32,
    out_ptr: u32, out_cap: u32,
) -> u32;  // Returns actual length, 0 if not found

// ... other host functions follow same ptr+len pattern
```

### 1.2 Serialization Format

**Strings:** UTF-8 bytes, passed as (ptr, len).

**Entities:** Simple TLV (Type-Length-Value) format:

```
Entity := field_count:u32 (Field)*
Field  := key_len:u32 key:bytes value_tag:u8 value_data:bytes

Value tags:
  0x00 = Null
  0x01 = String (len:u32, data:bytes)
  0x02 = Int (i32 little-endian)
  0x03 = Int8 (i64 little-endian)
  0x04 = BigInt (len:u32, signed little-endian bytes)
  0x05 = BigDecimal (string representation, len:u32, UTF-8 bytes)
  0x06 = Bool (0x00 or 0x01)
  0x07 = Bytes (len:u32, data:bytes)
  0x08 = Address (20 bytes)
  0x09 = Array (len:u32, Value*)
```

**Events:** Serialized using the same TLV format with well-known field names:
- `__block_number`: BigInt
- `__block_timestamp`: BigInt
- `__tx_hash`: Bytes (32)
- `__log_index`: BigInt
- `__address`: Address (20)
- Event-specific fields by name

### 1.3 Deliverables

The protocol is fully designed (see above) and implemented on the SDK side (`graphite/src/decode.rs`, `graphite/src/wasm/host.rs`). What remains is formalising it in the graph-node repo:

- [x] Protocol design — function signatures, TLV format, event layout (this document)
- [x] SDK-side implementation — `TlvReader`, `FromWasmBytes`, entity serialization in `WasmHost`
- [ ] `docs/rust-abi-spec.md` in graph-node repo (formal standalone spec)
- [ ] Shared constants for value tags (currently hardcoded on both sides — needs a shared crate or spec)
- [ ] Test vectors for serialization (cross-validate SDK and graph-node implementations)

---

## Phase 2: Implement rust_abi/ in graph-node

**Objective:** Create the Rust serialization layer parallel to `asc_abi/`.

### 2.1 New Files

```
runtime/wasm/src/
├── asc_abi/          # Existing AS code (unchanged)
├── rust_abi/         # NEW
│   ├── mod.rs        # Module root, Language enum
│   ├── types.rs      # RustAscType trait, serialization
│   ├── entity.rs     # Entity serialization
│   └── event.rs      # Event/trigger serialization
```

### 2.2 Core Types

```rust
// rust_abi/mod.rs
pub enum Language {
    AssemblyScript,
    Rust,
}

// rust_abi/types.rs
pub trait RustWasmType: Sized {
    fn to_wasm_bytes(&self) -> Vec<u8>;
    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DeserializeError>;
}

impl RustWasmType for String { ... }
impl RustWasmType for Entity { ... }
impl RustWasmType for Vec<u8> { ... }
// etc.
```

### 2.3 Host Function Wrappers

```rust
// rust_abi/host.rs
pub fn link_rust_exports(
    linker: &mut Linker<WasmInstanceData>,
    ctx: &WasmInstanceContext,
) -> Result<()> {
    linker.func_wrap("graphite", "store_set", |caller, args...| {
        // Read ptr+len arguments
        // Deserialize using RustWasmType
        // Call host_exports.store_set()
    })?;
    // ... other functions
}
```

### 2.4 Deliverables

**Status: Done.** Implemented in `/Users/pepe/Projects/graph-node/runtime/wasm/src/rust_abi/` (commits `2028ec3` and `e95972d`).

- [x] `rust_abi/mod.rs` (41 LOC) — `MappingLanguage` enum with `from_kind("wasm/rust")` parser
- [x] `rust_abi/types.rs` (247 LOC) — `ToRustWasm`/`FromRustWasm` traits + impls for i32, i64, bool, String, Vec<u8>, [u8;20], [u8;32], BigInt, BigDecimal. `ValueTag` enum for TLV tags.
- [x] `rust_abi/entity.rs` (260 LOC) — `serialize_entity()`/`deserialize_entity_data()` with full TLV, handles all graph-core Value types including Timestamp
- [x] `rust_abi/trigger.rs` (247 LOC) — `ToRustBytes` trait + `RustLogTrigger`, `RustCallTrigger`, `RustBlockTrigger` structs with serialization
- [x] `rust_abi/host.rs` (526 LOC) — All linker functions: store_set/get/remove (async), crypto_keccak256, log_log, data_source_address/network/create, ipfs_cat (async), ethereum_call (async, reorg detection, 5B gas). Memory helpers with gas metering. `is_rust_module()` for namespace detection.
- [x] Unit tests — entity roundtrips, trigger serialization, BigInt/BigDecimal/String roundtrips (14 tests passing)

---

## Phase 3: Manifest Parsing and Language Dispatch

**Objective:** Detect `language: wasm/rust` and route to correct ABI.

### 3.1 Manifest Changes

```yaml
# subgraph.yaml
mapping:
  kind: wasm/rust          # NEW: was "wasm/assemblyscript"
  apiVersion: 0.0.1        # Rust ABI version
  file: ./target/wasm32-unknown-unknown/release/my_subgraph.wasm
  entities:
    - Transfer
  eventHandlers:
    - event: Transfer(indexed address,indexed address,uint256)
      handler: handle_transfer
```

### 3.2 Code Changes

```rust
// graph/src/data_source/mod.rs
pub struct MappingDetails {
    pub kind: MappingKind,  // NEW enum: WasmAssemblyScript | WasmRust
    pub api_version: Version,
    // ...
}

pub enum MappingKind {
    WasmAssemblyScript,
    WasmRust,
}
```

```rust
// runtime/wasm/src/module/mod.rs
pub fn build_linker(...) -> Result<Linker<...>> {
    match mapping.kind {
        MappingKind::WasmAssemblyScript => {
            link_as_exports(&mut linker, ...)?;
        }
        MappingKind::WasmRust => {
            link_rust_exports(&mut linker, ...)?;
        }
    }
}
```

### 3.3 Handler Invocation

```rust
// runtime/wasm/src/module/instance.rs
fn invoke_handler(&mut self, handler: &str, trigger: &Trigger) -> Result<()> {
    match self.language {
        Language::AssemblyScript => {
            let ptr = trigger.to_asc_ptr(&mut self.heap)?;
            self.call_handler(handler, &[ptr.into()])?;
        }
        Language::Rust => {
            let bytes = trigger.to_rust_bytes()?;
            let ptr = self.write_bytes(&bytes)?;
            let len = bytes.len() as u32;
            self.call_handler(handler, &[ptr.into(), len.into()])?;
            self.call_reset_arena()?;
        }
    }
}
```

### 3.4 Deliverables

**Status: Done.** Implemented across `instance.rs`, `context.rs`, `mapping.rs`, `host.rs`, and `chain/ethereum/src/trigger.rs`.

- [x] Language detection — `MappingLanguage::from_kind()` parses `wasm/rust`; `is_rust_module()` detects "graphite" namespace imports
- [x] `MappingLanguage` enum stored on `ValidModule`, propagated through module construction
- [x] Dispatch in `build_linker()` — Rust modules skip AS linker macro, link only `rust_abi` functions + gas metering
- [x] `handle_trigger_rust()` — serializes trigger via `ToRustBytes`, allocates WASM memory, calls `handler(ptr, len)`, calls `reset_arena()`
- [x] `invoke_handler_rust()` (141 LOC) — full error handling: traps, timeouts, reorg detection, deterministic errors
- [x] `WasmInstanceContext` Rust ABI methods (171 LOC) — `rust_store_set/get/remove`, `rust_log`, `rust_data_source_*`, `rust_ipfs_cat`
- [x] Ethereum `ToRustBytes` — all 3 trigger types (Log, Call, Block) fully serialized
- [x] NEAR `ToRustBytes` — stub impl (unimplemented! — Ethereum-only for now)
- [x] Trait bounds (`ToRustBytes`) propagated through `instance_manager.rs`
- [x] Integration test with minimal Rust WASM module (`tests/integration/tests/wasm_handler.rs`)

---

## Phase 4: Complete the Graphite SDK

**Objective:** Finish the SDK to match the graph-node Rust ABI.

### 4.1 Handler Macro

```rust
// graphite-macros/src/lib.rs
#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Generate:
    // 1. User's function with host parameter
    // 2. extern "C" wrapper that:
    //    - Reads event bytes from WASM memory
    //    - Deserializes event using EventDecode
    //    - Creates WasmHost
    //    - Calls user function
    //    - Returns 0 on success
}

// Expands to:
#[no_mangle]
pub extern "C" fn handle_transfer(event_ptr: u32, event_len: u32) -> u32 {
    let bytes = unsafe {
        core::slice::from_raw_parts(event_ptr as *const u8, event_len as usize)
    };

    let event = match TransferEvent::from_wasm_bytes(bytes) {
        Ok(e) => e,
        Err(_) => return 1,
    };

    let mut host = WasmHost::new();
    _handle_transfer_impl(&mut host, &event);
    0
}
```

### 4.2 Event Deserialization

```rust
// graphite/src/decode.rs
pub trait FromWasmBytes: Sized {
    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;
}

// Generated by codegen for each event
impl FromWasmBytes for ERC20TransferEvent {
    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        // Parse TLV format, extract fields
    }
}
```

### 4.3 Entity Serialization

Ensure `wasm/host.rs` serialization matches Phase 1 spec:

```rust
// Already mostly done, verify matches spec
fn serialize_entity(entity: &Entity) -> Vec<u8> { ... }
fn deserialize_entity(bytes: &[u8]) -> Option<Entity> { ... }
```

### 4.4 Codegen Updates

Update ABI codegen to generate `FromWasmBytes` impl:

```rust
// graphite-cli/src/codegen/abi.rs
fn generate_event_struct(...) {
    // Add: impl FromWasmBytes for {Event} { ... }
}
```

### 4.5 Deliverables

**Status: Done.** All SDK-side work is complete and compiling.

- [x] Complete `#[handler]` macro with proper WASM wrapper — generates `extern "C"` with `FromWasmBytes` deserialization, `WasmHost` creation, conditional compilation for WASM/native
- [x] Add `FromWasmBytes` trait and implementations — full `TlvReader` with all value types, `RawLog`/`RawCall`/`RawBlock` deserialization
- [x] Update codegen to generate `FromWasmBytes` — ABI codegen produces `FromWasmBytes` impl via `RawLog` → `from_raw_log` path
- [x] Entity serialization in `WasmHost` — TLV format with tag bytes matching Phase 1 spec
- [x] `reset_arena` export in `alloc.rs` — bump allocator with `allocate()` + `reset_arena()`
- [x] Unit tests for serialization — decode tests passing, codegen tests passing
- [x] `EventDecode` trait with selector checking and topic/data decoding
- [x] `MockHost` with in-memory store, eth call mocks, IPFS mocks, log capture
- [x] ERC20 example subgraph compiles and tests pass

---

## Phase 5: Integration Testing

**Objective:** Verify end-to-end functionality with a real subgraph.

### 5.1 Test Subgraph

Create a minimal ERC20 transfer indexer:

```
test-subgraph/
├── Cargo.toml
├── graphite.toml
├── subgraph.yaml        # language: wasm/rust
├── schema.graphql
├── abis/ERC20.json
└── src/lib.rs
```

### 5.2 Test Cases

1. **Compile test:** `graphite build` produces valid WASM
2. **Deploy test:** graph-node accepts the subgraph
3. **Index test:** Events are processed, entities created
4. **Query test:** GraphQL queries return expected data
5. **Error handling:** Handler panics are caught gracefully

### 5.3 CI Integration

**Status: Done.** Both WASM-level and live integration tests passing.

- [x] WASM integration test (`tests/integration/tests/wasm_handler.rs`) — loads ERC20 WASM with wasmtime, serializes a Transfer event using graph-node's exact `RustLogTrigger` binary format, calls `handle_transfer(ptr, len)`, captures and verifies `store_set` entity data. All fields validated: from, to, value, blockNumber, timestamp, transactionHash, id.
- [x] Live integration test (`scripts/live-test.sh`) — deployed ERC20 subgraph to running graph-node fork, indexing real USDC Transfer events from Ethereum mainnet (block 24756400+), queried via GraphQL. Full pipeline verified: block ingestion → event scanning → Rust WASM handler → entity storage → GraphQL responses with correct from/to/value/blockNumber/timestamp fields.
- [ ] Performance comparison vs AS equivalent

---

## File Change Summary

### graph-node (new files)

```
runtime/wasm/src/rust_abi/
├── mod.rs           (~100 lines)
├── types.rs         (~200 lines)
├── entity.rs        (~150 lines)
├── event.rs         (~150 lines)
└── host.rs          (~400 lines)
```

### graph-node (modified files)

```
graph/src/data_source/mod.rs           (+50 lines) - MappingKind enum
graph/src/data_source/manifest.rs      (+30 lines) - Parse language
runtime/wasm/src/mapping.rs            (+20 lines) - Skip parity_wasm for Rust modules
runtime/wasm/src/module/mod.rs         (+25 lines) - Linker dispatch, skip id_of_type for Rust
runtime/wasm/src/module/instance.rs    (+50 lines) - Handler invocation, skip _start for Rust
runtime/wasm/src/lib.rs                (+5 lines)  - Export rust_abi
chain/ethereum/src/trigger.rs          (+65 lines) - ToRustBytes for all Ethereum trigger types
chain/near/src/trigger.rs              (+10 lines) - ToRustBytes stub for NEAR
core/src/subgraph/instance_manager.rs  (+5 lines)  - ToRustBytes trait bounds
```

### graph-node (modified files, since initial PR)

```
runtime/wasm/src/mapping.rs            (+10 lines) - Wasmtime fuel metering config for Rust modules
runtime/wasm/src/module/instance.rs    (+20 lines) - Fuel budget, OutOfFuel trap handling
```

### graphite (modified files)

```
graphite-macros/src/lib.rs         (~100 lines changed) - Handler macro with panic hook + decode error logging
graphite/src/decode.rs             (+50 lines)  - FromWasmBytes trait
graphite/src/primitives.rs         (+10 lines)  - BigInt LE serialization
graphite/src/wasm/host.rs          (+30 lines)  - BigInt BE → LE fix, store_get retry on buffer overflow
graphite/src/wasm/alloc.rs         (+10 lines)  - Allocator bounds checking (4MB limit)
graphite-cli/src/codegen/abi.rs    (+80 lines)  - Generate FromWasmBytes
```

### graphite (new files)

```
graphite/src/wasm/panic.rs         - Panic hook forwarding to graph-node abort FFI
graphite-cli/src/deploy.rs         - CLI deploy command (IPFS upload + JSON-RPC)
tests/integration/                 - WASM integration test crate (wasmtime-based)
scripts/live-test.sh               - Live deployment script
examples/erc20/schema-live.graphql - Simplified schema for live test
examples/erc20/subgraph-live.yaml  - Live test manifest config
```

**Estimated total:** ~1,500 lines new code in graph-node, ~600 lines in graphite SDK

---

## Timeline

| Phase | Description | Effort | Status |
|-------|-------------|--------|--------|
| 1 | Define Rust ABI Protocol | 1 day | **Designed** (formal spec doc remaining) |
| 2 | Implement rust_abi/ in graph-node | 3-4 days | **Done** (~1,450 LOC) |
| 3 | Manifest parsing and dispatch | 1-2 days | **Done** |
| 4 | Complete Graphite SDK | 2 days | **Done** |
| 5 | Integration testing | 2-3 days | **Done** |

**Remaining:** Formal spec documentation (Phase 1 docs) and minor cleanup. All implementation, gas metering, error handling, and integration testing is complete. Draft PR open: [#6462](https://github.com/graphprotocol/graph-node/pull/6462).

---

## Open Questions

1. **Error handling:** Resolved. Handlers return error codes (0 = success, non-zero = error). Panics are caught by a custom panic hook that forwards the message, file, and line number to graph-node via the `abort` FFI. Decode errors are logged via `log_log` before returning code 1.

2. **Gas metering:** Resolved. Wasmtime fuel metering (`config.consume_fuel(true)`, `store.set_fuel(10B)`) provides deterministic per-instruction gas accounting. `Trap::OutOfFuel` is caught and reported as a deterministic error. Committed in graph-node PR [#6462](https://github.com/graphprotocol/graph-node/pull/6462).

3. **API versioning:** Start at `0.0.1` or align with AS versions? (Current manifest uses `apiVersion: 0.0.7` to match AS). Decision deferred to upstream review.

4. **WASM features:** Resolved — Rust modules use standard WASM features (bulk-memory, reference-types, multivalue, sign-ext). These are handled natively by wasmtime 38; the parity_wasm bypass ensures they aren't stripped.

5. **Debugging:** Resolved. Custom panic hook in `graphite/src/wasm/panic.rs` calls graph-node's `abort` with the full panic message including file and line number. Decode errors are logged with type name, handler name, and error details.

---

## Next Steps

All implementation phases (1-5) are functionally complete. Gas metering, error handling, and the CLI deploy command are all implemented. The SDK, graph-node fork, and integration tests all work end-to-end with real Ethereum mainnet data.

### Done since last update

- [x] **Gas metering** — wasmtime fuel metering for Rust modules (10B fuel budget, `Trap::OutOfFuel` as deterministic error). Pushed to graph-node PR [#6462](https://github.com/graphprotocol/graph-node/pull/6462).
- [x] **CLI deploy command** — full `graphite deploy` implementation: IPFS upload, manifest rewriting, JSON-RPC subgraph_create + subgraph_deploy.
- [x] **Error handling** — panic hook (forwards panic message + file + line to graph-node via `abort`), allocator bounds checking (4MB limit), decode error logging (type, handler, error details surfaced via `log_log`), `store_get` retry on buffer overflow (16KB → 256KB).
- [x] **Graph-node fork pushed** — `cargopete/graph-node`, branch `rust-abi-support`, draft PR [#6462](https://github.com/graphprotocol/graph-node/pull/6462).

### Towards upstream merge

1. **Write formal ABI spec** (`docs/rust-abi-spec.md`) — document the protocol for the upstream PR reviewers
2. **Performance comparison** — benchmark Rust vs AS subgraphs (binary size, indexing speed, memory)
3. **Address PR review feedback** — respond to any comments on [#6462](https://github.com/graphprotocol/graph-node/pull/6462)

### Bugs found & fixed during live testing

These are worth documenting for anyone working on the graph-node integration:

1. **parity_wasm opcode 252** — `parity_wasm` (used for AS gas injection) can't parse modern WASM features like `memory.copy` (bulk-memory proposal). Fix in `mapping.rs`: detect Rust modules by scanning for `"graphite"` in raw bytes, skip parity_wasm pipeline entirely.
2. **`id_of_type` not found** — AS-specific export required during module instantiation. Fix in `module/mod.rs`: `AscHeapCtx::new()` accepts `MappingLanguage` parameter, sets `id_of_type = None` for Rust.
3. **`_start` not found** — AS entry point called after instantiation. Fix in `module/instance.rs`: skip `_start` call for Rust modules.
4. **BigInt endianness mismatch** — SDK serialized BigInt as signed big-endian, graph-node deserialized as signed little-endian. Fix: SDK changed to little-endian (`to_signed_bytes_le`).

### Cleanup (nice to have, not blocking)

- [x] ~~Implement CLI `deploy` command~~ — done
- [ ] Fix unused `Vec` import warning in ERC20 example codegen
- [ ] Implement offchain/subgraph trigger serialization (currently stubbed as empty bytes)
- [ ] Add more unit tests + `proptest` property-based testing
- [ ] Consider a shared `graphite-abi` crate for TLV tag constants (used by both SDK and graph-node)
