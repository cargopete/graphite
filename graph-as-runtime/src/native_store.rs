//! Thread-local in-memory store for native (non-WASM) unit tests.
//!
//! Subgraph handlers that use the `graphite` SDK's `HostFunctions` trait
//! should prefer `graphite::mock::MockHost`. This module provides the lower-
//! level storage that the assertion helpers in `graphite::mock` read from.

use std::cell::RefCell;
use std::collections::HashMap;

/// A single stored entity field value.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    String(std::string::String),
    Bytes(Vec<u8>),
    /// Little-endian two's-complement bytes (graph-ts BigInt).
    BigInt(Vec<u8>),
    Bool(bool),
    Int(i32),
    /// 64-bit signed integer (Int8 / Timestamp scalars).
    Int8(i64),
    /// IEEE 754 double (Float scalar).
    Float(f64),
    Null,
}

/// In-memory store: entity_type -> id -> (field_name -> FieldValue).
#[derive(Default, Debug)]
pub struct NativeStore {
    pub entities: HashMap<
        std::string::String,
        HashMap<std::string::String, HashMap<std::string::String, FieldValue>>,
    >,
    /// Captured log messages: (level, message).
    pub logs: Vec<(u32, std::string::String)>,
}

impl NativeStore {
    /// Retrieve a single entity's fields, if it exists.
    pub fn get_entity(
        &self,
        entity_type: &str,
        id: &str,
    ) -> Option<&HashMap<std::string::String, FieldValue>> {
        self.entities.get(entity_type)?.get(id)
    }

    /// Store (upsert) an entity.
    pub fn set_entity(
        &mut self,
        entity_type: &str,
        id: &str,
        fields: HashMap<std::string::String, FieldValue>,
    ) {
        self.entities
            .entry(entity_type.to_string())
            .or_default()
            .insert(id.to_string(), fields);
    }

    /// Remove an entity.
    pub fn remove_entity(&mut self, entity_type: &str, id: &str) {
        if let Some(m) = self.entities.get_mut(entity_type) {
            m.remove(id);
        }
    }

    /// Count entities of a given type.
    pub fn entity_count(&self, entity_type: &str) -> usize {
        self.entities.get(entity_type).map(|m| m.len()).unwrap_or(0)
    }

    /// Check if an entity exists.
    pub fn has_entity(&self, entity_type: &str, id: &str) -> bool {
        self.get_entity(entity_type, id).is_some()
    }
}

thread_local! {
    /// The global test store. Each test thread gets its own isolated instance.
    pub static STORE: RefCell<NativeStore> = RefCell::new(NativeStore::default());
}

/// Reset the store to an empty state. Call at the start of each test.
pub fn reset() {
    STORE.with(|s| *s.borrow_mut() = NativeStore::default());
}

/// Run a closure with a shared reference to the store.
pub fn with_store<F, R>(f: F) -> R
where
    F: FnOnce(&NativeStore) -> R,
{
    STORE.with(|s| f(&s.borrow()))
}

/// Run a closure with a mutable reference to the store.
pub fn with_store_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut NativeStore) -> R,
{
    STORE.with(|s| f(&mut s.borrow_mut()))
}
