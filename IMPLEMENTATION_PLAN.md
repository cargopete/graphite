# Rust Subgraph Implementation Plan

This document outlines the changes required to enable Rust subgraphs in The Graph ecosystem. It covers modifications to both graph-node and the Graphite SDK.

---

## Overview

**Goal:** Enable `language: wasm/rust` subgraphs that compile Rust handlers to WASM and run on graph-node with a clean, Rust-native ABI.

**Key Insight:** graph-node's `host_exports.rs` is already language-agnostic. The AS coupling exists only in the serialization layer (`asc_abi/`). We add a parallel `rust_abi/` and dispatch based on manifest language.

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
  0x04 = BigInt (len:u32, signed big-endian bytes)
  0x05 = BigDecimal (scale:i64, len:u32, unscaled big-endian bytes)
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

- [ ] `docs/rust-abi-spec.md` in graph-node repo
- [ ] Shared constants for value tags
- [ ] Test vectors for serialization

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

- [ ] `rust_abi/mod.rs` - module structure, Language enum
- [ ] `rust_abi/types.rs` - RustWasmType trait and impls
- [ ] `rust_abi/entity.rs` - Entity serialization
- [ ] `rust_abi/event.rs` - Event/trigger serialization
- [ ] `rust_abi/host.rs` - Linker function wrappers
- [ ] Unit tests for serialization roundtrips

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

- [ ] Update manifest parsing for `kind: wasm/rust`
- [ ] Add `MappingKind` enum
- [ ] Dispatch in `build_linker()`
- [ ] Update handler invocation for Rust calling convention
- [ ] Integration test with minimal Rust WASM module

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

- [ ] Complete `#[handler]` macro with proper WASM wrapper
- [ ] Add `FromWasmBytes` trait and implementations
- [ ] Update codegen to generate `FromWasmBytes`
- [ ] Verify entity serialization matches spec
- [ ] Add `reset_arena` export to alloc.rs (already done)
- [ ] Unit tests for serialization

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

- [ ] Add Rust subgraph to graph-node integration tests
- [ ] Test against mainnet fork with real contract events
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
graph/src/data_source/mod.rs       (+50 lines) - MappingKind enum
graph/src/data_source/manifest.rs  (+30 lines) - Parse language
runtime/wasm/src/module/mod.rs     (+20 lines) - Linker dispatch
runtime/wasm/src/module/instance.rs (+40 lines) - Handler invocation
runtime/wasm/src/lib.rs            (+5 lines)  - Export rust_abi
```

### graphite (modified files)

```
graphite-macros/src/lib.rs         (~100 lines changed) - Handler macro
graphite/src/decode.rs             (+50 lines) - FromWasmBytes trait
graphite-cli/src/codegen/abi.rs    (+80 lines) - Generate FromWasmBytes
```

**Estimated total:** ~1,400 lines new code, ~200 lines modified

---

## Timeline

| Phase | Description | Effort |
|-------|-------------|--------|
| 1 | Define Rust ABI Protocol | 1 day |
| 2 | Implement rust_abi/ in graph-node | 3-4 days |
| 3 | Manifest parsing and dispatch | 1-2 days |
| 4 | Complete Graphite SDK | 2 days |
| 5 | Integration testing | 2-3 days |

**Total:** ~10-12 days of focused work

---

## Open Questions

1. **Error handling:** Should handler return error codes or panic? (Suggest: return codes, panic as fallback)

2. **Gas metering:** Use same gas model as AS or define Rust-specific costs?

3. **API versioning:** Start at `0.0.1` or align with AS versions?

4. **WASM features:** Require specific WASM features (bulk-memory, etc.)?

5. **Debugging:** How to surface Rust panic messages to users?

---

## Next Steps

1. Review this plan
2. Create graph-node fork
3. Start with Phase 1 (ABI spec) to lock down the protocol
4. Parallel work: Phase 2 (graph-node) and Phase 4 (SDK) can proceed together once spec is stable
