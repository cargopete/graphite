//! Mock host for native `cargo test` — no WASM, no graph-node required.
//!
//! # Usage
//!
//! ```rust,ignore
//! use graphite::mock;
//!
//! #[test]
//! fn my_handler_test() {
//!     mock::reset();
//!
//!     // Call your handler's _impl function directly:
//!     handle_transfer_impl(&my_mock_event());
//!
//!     // Assert what ended up in the store:
//!     mock::assert_entity("Transfer", "abcd...-0")
//!         .field_bytes("from", &[0xaa; 20])
//!         .field_bytes("to",   &[0xbb; 20]);
//! }
//! ```
//!
//! Handlers that write to the store via the `graphite` SDK's `HostFunctions`
//! trait should use `graphite::testing::MockHost` instead.
//!
//! This module is a thin façade over `graph_as_runtime::native_store`.

use graph_as_runtime::native_store::{self, FieldValue};
use std::collections::HashMap;

// ============================================================================
// Top-level helpers
// ============================================================================

/// Reset the in-memory store and logs. Call at the top of every test.
pub fn reset() {
    native_store::reset();
}

/// Return the number of stored entities of the given type.
pub fn entity_count(entity_type: &str) -> usize {
    native_store::with_store(|s| s.entity_count(entity_type))
}

/// Return true if an entity of the given type and id exists in the store.
pub fn has_entity(entity_type: &str, id: &str) -> bool {
    native_store::with_store(|s| s.has_entity(entity_type, id))
}

/// Begin an assertion chain for an entity.
///
/// # Panics
/// Panics immediately if no entity with the given `entity_type` and `id` exists.
pub fn assert_entity(entity_type: &str, id: &str) -> EntityAssert {
    let fields = native_store::with_store(|s| {
        s.get_entity(entity_type, id)
            .cloned()
            .unwrap_or_else(|| panic!("Entity {}/{} not found in mock store", entity_type, id))
    });
    EntityAssert {
        entity_type: entity_type.to_string(),
        id: id.to_string(),
        fields,
    }
}

// ============================================================================
// EntityAssert — fluent assertion builder
// ============================================================================

/// Fluent assertion helper for a single entity in the mock store.
///
/// Each `field_*` method panics immediately with a descriptive message if the
/// assertion fails, then returns `self` so you can chain further assertions.
pub struct EntityAssert {
    entity_type: String,
    id: String,
    fields: HashMap<String, FieldValue>,
}

impl EntityAssert {
    fn get_field(&self, field: &str) -> &FieldValue {
        self.fields.get(field).unwrap_or_else(|| {
            panic!(
                "Field '{}' not found on entity {}/{}. Available fields: {:?}",
                field,
                self.entity_type,
                self.id,
                self.fields.keys().collect::<Vec<_>>()
            )
        })
    }

    /// Assert a string field equals `expected`.
    pub fn field_string(self, field: &str, expected: &str) -> Self {
        match self.get_field(field) {
            FieldValue::String(s) => assert_eq!(
                s, expected,
                "Entity {}/{} field '{}': expected string {:?}, got {:?}",
                self.entity_type, self.id, field, expected, s
            ),
            other => panic!(
                "Entity {}/{} field '{}': expected String, got {:?}",
                self.entity_type, self.id, field, other
            ),
        }
        self
    }

    /// Assert a bytes field equals `expected`.
    pub fn field_bytes(self, field: &str, expected: &[u8]) -> Self {
        match self.get_field(field) {
            FieldValue::Bytes(b) => assert_eq!(
                b.as_slice(), expected,
                "Entity {}/{} field '{}': bytes mismatch",
                self.entity_type, self.id, field
            ),
            other => panic!(
                "Entity {}/{} field '{}': expected Bytes, got {:?}",
                self.entity_type, self.id, field, other
            ),
        }
        self
    }

    /// Assert a BigInt field (little-endian bytes) equals `expected`.
    pub fn field_bigint(self, field: &str, expected: &[u8]) -> Self {
        match self.get_field(field) {
            FieldValue::BigInt(b) => assert_eq!(
                b.as_slice(), expected,
                "Entity {}/{} field '{}': BigInt bytes mismatch",
                self.entity_type, self.id, field
            ),
            other => panic!(
                "Entity {}/{} field '{}': expected BigInt, got {:?}",
                self.entity_type, self.id, field, other
            ),
        }
        self
    }

    /// Assert a bool field equals `expected`.
    pub fn field_bool(self, field: &str, expected: bool) -> Self {
        match self.get_field(field) {
            FieldValue::Bool(b) => assert_eq!(
                *b, expected,
                "Entity {}/{} field '{}': expected bool {}, got {}",
                self.entity_type, self.id, field, expected, b
            ),
            other => panic!(
                "Entity {}/{} field '{}': expected Bool, got {:?}",
                self.entity_type, self.id, field, other
            ),
        }
        self
    }

    /// Assert an i32 field equals `expected`.
    pub fn field_int(self, field: &str, expected: i32) -> Self {
        match self.get_field(field) {
            FieldValue::Int(n) => assert_eq!(
                *n, expected,
                "Entity {}/{} field '{}': expected i32 {}, got {}",
                self.entity_type, self.id, field, expected, n
            ),
            other => panic!(
                "Entity {}/{} field '{}': expected Int, got {:?}",
                self.entity_type, self.id, field, other
            ),
        }
        self
    }

    /// Assert a field exists (any type).
    pub fn field_exists(self, field: &str) -> Self {
        // get_field already panics if missing
        let _ = self.get_field(field);
        self
    }
}

// ============================================================================
// MockHost — graphite::HostFunctions impl backed by native_store
// ============================================================================
//
// This implements the graphite `HostFunctions` trait so handlers written
// against the graphite SDK (using `host.store_set(...)`) can be tested
// natively. The store state is shared with the `assert_entity` helpers above.

use crate::host::{EthereumCallError, HostFunctions, IpfsError, LogLevel};
use crate::primitives::{Address, Bytes};
use crate::store::{Entity, Value};

/// A `HostFunctions` implementation backed by the thread-local `native_store`.
///
/// Use this when your handler accepts a `&mut impl HostFunctions` argument.
/// For raw AS-ABI handlers (those that call `store_set` directly via FFI),
/// the handler's `_impl` variant should accept `host: &mut MockHost` and
/// write to `host.store`.
///
/// After running the handler, inspect the store via the `mock::assert_entity`
/// helpers (which read from the same thread-local) or via `MockHost::store`.
#[derive(Debug, Default)]
pub struct MockHost {
    /// Captured log messages.
    pub logs: Vec<(LogLevel, String)>,
    /// Mock ethereum call responses. Key: (address, calldata).
    pub eth_calls: std::collections::HashMap<(Address, Vec<u8>), Result<Bytes, EthereumCallError>>,
    /// Mock IPFS content. Key: hash string.
    pub ipfs_content: std::collections::HashMap<String, Bytes>,
    /// Created data sources.
    pub created_data_sources: Vec<(String, Vec<String>)>,
    /// Current data source address.
    pub current_address: Address,
    /// Current network name.
    pub current_network: String,
}

impl MockHost {
    /// Create a new `MockHost`.
    pub fn new() -> Self {
        Self {
            current_network: "mainnet".to_string(),
            ..Default::default()
        }
    }

    /// Set a mock ethernet call response.
    pub fn mock_eth_call(
        &mut self,
        address: Address,
        calldata: impl Into<Vec<u8>>,
        response: Result<Bytes, EthereumCallError>,
    ) {
        self.eth_calls.insert((address, calldata.into()), response);
    }

    /// Set mock IPFS content.
    pub fn mock_ipfs(&mut self, hash: impl Into<String>, content: impl Into<Bytes>) {
        self.ipfs_content.insert(hash.into(), content.into());
    }

    /// Return how many entities of `entity_type` are in the store.
    pub fn entity_count(&self, entity_type: &str) -> usize {
        native_store::with_store(|s| s.entity_count(entity_type))
    }

    /// Return true if the entity exists in the store.
    pub fn has_entity(&self, entity_type: &str, id: &str) -> bool {
        native_store::with_store(|s| s.has_entity(entity_type, id))
    }
}

/// Convert a graphite `Entity` (BTreeMap<String, Value>) into the native
/// store's `HashMap<String, FieldValue>` representation.
fn entity_to_native_fields(
    entity: &Entity,
) -> HashMap<String, FieldValue> {
    entity
        .iter()
        .map(|(k, v)| {
            let fv = value_to_field_value(v);
            (k.clone(), fv)
        })
        .collect()
}

fn value_to_field_value(v: &Value) -> FieldValue {
    match v {
        Value::String(s) => FieldValue::String(s.clone()),
        Value::Bytes(b) => FieldValue::Bytes(b.as_slice().to_vec()),
        Value::BigInt(n) => FieldValue::BigInt(n.to_signed_bytes_le()),
        Value::Bool(b) => FieldValue::Bool(*b),
        Value::Int(n) => FieldValue::Int(*n),
        Value::Address(a) => FieldValue::Bytes(a.as_slice().to_vec()),
        Value::Null => FieldValue::Null,
        // Array and BigDecimal: store as Null for now (unsupported in testing).
        _ => FieldValue::Null,
    }
}

impl HostFunctions for MockHost {
    fn store_set(&mut self, entity_type: &str, id: &str, entity: Entity) {
        let fields = entity_to_native_fields(&entity);
        native_store::with_store_mut(|s| s.set_entity(entity_type, id, fields));
    }

    fn store_get(&self, entity_type: &str, id: &str) -> Option<Entity> {
        native_store::with_store(|s| {
            s.get_entity(entity_type, id).map(|fields| {
                let mut e = Entity::new();
                for (k, fv) in fields {
                    let v = match fv {
                        FieldValue::String(s) => Value::String(s.clone()),
                        FieldValue::Bytes(b) => {
                            Value::Bytes(crate::primitives::Bytes::from_slice(b))
                        }
                        FieldValue::BigInt(b) => {
                            Value::BigInt(crate::primitives::BigInt::from_signed_bytes_le(b))
                        }
                        FieldValue::Bool(b) => Value::Bool(*b),
                        FieldValue::Int(n) => Value::Int(*n),
                        FieldValue::Null => Value::Null,
                    };
                    e.set(k.clone(), v);
                }
                e
            })
        })
    }

    fn store_remove(&mut self, entity_type: &str, id: &str) {
        native_store::with_store_mut(|s| s.remove_entity(entity_type, id));
    }

    fn ethereum_call_raw(
        &self,
        address: Address,
        calldata: &[u8],
    ) -> Result<Bytes, EthereumCallError> {
        self.eth_calls
            .get(&(address, calldata.to_vec()))
            .cloned()
            .unwrap_or(Err(EthereumCallError::Failed(
                "no mock configured for this call".to_string(),
            )))
    }

    fn crypto_keccak256(&self, input: &[u8]) -> [u8; 32] {
        use alloy_primitives::keccak256;
        keccak256(input).0
    }

    fn log(&self, level: LogLevel, message: &str) {
        native_store::with_store_mut(|s| {
            let lvl = match level {
                LogLevel::Critical => 0,
                LogLevel::Error => 1,
                LogLevel::Warning => 2,
                LogLevel::Info => 3,
                LogLevel::Debug => 4,
            };
            s.logs.push((lvl, message.to_string()));
        });
        eprintln!("[{:?}] {}", level, message);
    }

    fn ipfs_cat(&self, hash: &str) -> Result<Bytes, IpfsError> {
        self.ipfs_content
            .get(hash)
            .cloned()
            .ok_or_else(|| IpfsError::NotFound(hash.to_string()))
    }

    fn data_source_create(&mut self, name: &str, params: &[String]) {
        self.created_data_sources
            .push((name.to_string(), params.to_vec()));
    }

    fn data_source_address(&self) -> Address {
        self.current_address
    }

    fn data_source_network(&self) -> String {
        self.current_network.clone()
    }
}

// ============================================================================
// Convenience: write an entity directly into the native store (for seeding).
// ============================================================================

/// Insert a pre-built entity into the mock store.
///
/// Useful for seeding state before calling a handler that calls `store.get`.
pub fn seed_entity(entity_type: &str, id: &str, entity: Entity) {
    let fields = entity_to_native_fields(&entity);
    native_store::with_store_mut(|s| s.set_entity(entity_type, id, fields));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Entity;

    #[test]
    fn mock_host_store_set_and_assert() {
        reset();

        let mut host = MockHost::new();
        let mut e = Entity::new();
        e.set("name", "Alice");
        e.set("balance", crate::primitives::BigInt::from(42u64));
        host.store_set("User", "0x01", e);

        assert!(has_entity("User", "0x01"));
        assert_eq!(entity_count("User"), 1);
        assert_entity("User", "0x01")
            .field_string("name", "Alice");
    }

    #[test]
    fn mock_host_store_remove() {
        reset();

        let mut host = MockHost::new();
        let mut e = Entity::new();
        e.set("val", "test");
        host.store_set("Foo", "1", e);
        assert!(has_entity("Foo", "1"));

        host.store_remove("Foo", "1");
        assert!(!has_entity("Foo", "1"));
    }

    #[test]
    fn seed_and_get() {
        reset();

        let mut e = Entity::new();
        e.set("x", 99i32);
        seed_entity("Bar", "abc", e.clone());

        let host = MockHost::new();
        let loaded = host.store_get("Bar", "abc").expect("entity should exist");
        assert_eq!(loaded.get("x"), Some(&crate::store::Value::Int(99)));
    }
}
