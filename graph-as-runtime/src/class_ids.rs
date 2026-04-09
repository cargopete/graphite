//! AssemblyScript class IDs for graph-ts types.
//!
//! These constants are derived from compiling graph-ts v0.31 and reading the
//! `__rtti_base` table that AssemblyScript emits. They correspond to the
//! `AscTypeId` enum in graph-node's `graph/src/runtime/asc_abi/`.
//!
//! If store.set calls fail silently, verify these against graph-node's
//! `runtime/src/host_exports.rs` or the AscTypeId enum. They change only
//! when the graph-ts class hierarchy changes.
//!
//! Source: gnosis/subgraph-rs, verified against graph-node graph-ts@0.31 output.

/// `Array<TypedMapEntry<string,Value>>` — the entries array inside TypedMap.
pub const ARRAY_TYPED_MAP_ENTRY: u32 = 28;

/// `TypedMap<string, Value>` — the entity object passed to store.set.
pub const TYPED_MAP: u32 = 30;

/// `TypedMapEntry<string, Value>` — one key/value pair in a TypedMap.
pub const TYPED_MAP_ENTRY: u32 = 31;

/// `Value` — the discriminated union that holds a single entity field value.
pub const VALUE: u32 = 35;
