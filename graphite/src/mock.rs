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
use std::cell::RefCell;
use std::collections::HashMap;

// Thread-local tracking of ethereum calls made during a test.
thread_local! {
    static ETH_CALLS_MADE: RefCell<Vec<([u8; 20], Vec<u8>)>> = RefCell::new(Vec::new());
    /// Thread-local IPFS content for AS-ABI-style handlers. See `set_ipfs_result`.
    static IPFS_CONTENT: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
    /// Thread-local ENS registry for AS-ABI-style handlers. See `set_ens_name`.
    static ENS_NAMES: RefCell<HashMap<[u8; 20], String>> = RefCell::new(HashMap::new());
    /// Thread-local data source context. See `set_data_source_context`.
    static DATA_SOURCE_CONTEXT: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    /// Data sources created by hostless `data_source::create_file` etc. in #[handler] code.
    static CREATED_DATA_SOURCES: RefCell<Vec<(String, Vec<String>)>> = RefCell::new(Vec::new());
    /// Current data source address, returned by `data_source::address_current()` on native.
    static CURRENT_ADDRESS: RefCell<[u8; 20]> = RefCell::new([0u8; 20]);
}

// ============================================================================
// Top-level helpers
// ============================================================================

/// Reset the in-memory store, logs, call records, IPFS content, and ENS names. Call at the top of every test.
pub fn reset() {
    native_store::reset();
    ETH_CALLS_MADE.with(|c| c.borrow_mut().clear());
    IPFS_CONTENT.with(|c| c.borrow_mut().clear());
    ENS_NAMES.with(|c| c.borrow_mut().clear());
    DATA_SOURCE_CONTEXT.with(|c| c.borrow_mut().clear());
    CREATED_DATA_SOURCES.with(|c| c.borrow_mut().clear());
    CURRENT_ADDRESS.with(|c| *c.borrow_mut() = [0u8; 20]);
}

/// Register mock IPFS content for the given CID.
///
/// When `MockHost::ipfs_cat` is called with this `cid` it will return `content`.
/// This stores the content in a thread-local so it works without a `MockHost` reference.
pub fn set_ipfs_result(cid: impl Into<String>, content: impl Into<Vec<u8>>) {
    IPFS_CONTENT.with(|c| c.borrow_mut().insert(cid.into(), content.into()));
}

/// Register a mock ENS name for the given address.
///
/// When `ens::name_by_address` is called with this address (in an AS-ABI-style handler)
/// it will return `name`. Thread-local, cleared by `mock::reset()`.
pub fn set_ens_name(address: [u8; 20], name: impl Into<String>) {
    ENS_NAMES.with(|c| c.borrow_mut().insert(address, name.into()));
}

/// Look up a mocked ENS name — used by `graphite::ens::name_by_address` on native.
pub(crate) fn get_ens_name(address: &[u8; 20]) -> Option<String> {
    ENS_NAMES.with(|c| c.borrow().get(address).cloned())
}

/// Record a data source creation from hostless `data_source::create_file` etc.
/// Used by `graphite::data_source` on native when there is no host reference.
pub(crate) fn record_data_source_created(name: &str, params: &[String]) {
    CREATED_DATA_SOURCES.with(|c| {
        c.borrow_mut().push((name.to_string(), params.to_vec()));
    });
}

/// Return all data sources created via the hostless API during this test.
///
/// Each entry is `(template_name, params)`. For `create_file`, `params[0]` is the IPFS CID.
/// Cleared by `mock::reset()`.
pub fn get_created_data_sources() -> Vec<(String, Vec<String>)> {
    CREATED_DATA_SOURCES.with(|c| c.borrow().clone())
}

/// Assert that a file/ipfs data source was created for the given template and CID.
///
/// # Panics
/// Panics if no matching `create_file` call was recorded.
pub fn assert_file_data_source_created(template: &str, cid: &str) {
    let found = CREATED_DATA_SOURCES.with(|c| {
        c.borrow()
            .iter()
            .any(|(n, params)| n == template && params.first().map(|p| p == cid).unwrap_or(false))
    });
    assert!(
        found,
        "Expected file data source '{}' for CID '{}' to be created, but it wasn't.\nCreated: {:?}",
        template,
        cid,
        CREATED_DATA_SOURCES.with(|c| c.borrow().clone()),
    );
}

/// Assert that an ethereum/contract data source was created for the given template and address.
///
/// Checks the thread-local `CREATED_DATA_SOURCES` list populated by
/// `data_source::create_contract`. Cleared by `mock::reset()`.
///
/// # Panics
/// Panics if no matching creation was recorded.
pub fn assert_contract_data_source_created(template: &str, address: [u8; 20]) {
    let hex = format!(
        "0x{}",
        address.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    );
    let found = CREATED_DATA_SOURCES.with(|c| {
        c.borrow()
            .iter()
            .any(|(n, params)| n == template && params.first().map(|p| p == &hex).unwrap_or(false))
    });
    assert!(
        found,
        "Expected contract data source '{}' for address {} to be created, but it wasn't.\nCreated: {:?}",
        template,
        hex,
        CREATED_DATA_SOURCES.with(|c| c.borrow().clone()),
    );
}

/// Set the mock data source address returned by `data_source::address_current()`.
///
/// Use in tests that exercise handlers reading their data source address.
/// Cleared by `mock::reset()`.
pub fn set_current_address(address: [u8; 20]) {
    CURRENT_ADDRESS.with(|c| *c.borrow_mut() = address);
}

/// Return the current mock data source address — used by `data_source::address_current()` on native.
pub(crate) fn get_current_address() -> [u8; 20] {
    CURRENT_ADDRESS.with(|c| *c.borrow())
}

/// Retrieve a single string value from the mock data source context.
pub(crate) fn get_data_source_context_string(key: &str) -> Option<String> {
    DATA_SOURCE_CONTEXT.with(|c| c.borrow().get(key).cloned())
}

/// Set a key-value pair in the mock data source context.
///
/// Use before calling a handler that reads `dataSource.context()`.
/// Cleared by `mock::reset()`.
pub fn set_data_source_context(key: impl Into<String>, value: impl Into<String>) {
    DATA_SOURCE_CONTEXT.with(|c| {
        c.borrow_mut().insert(key.into(), value.into());
    });
}

/// Retrieve the mock data source context as an entity.
pub(crate) fn get_data_source_context() -> crate::store::Entity {
    DATA_SOURCE_CONTEXT.with(|c| {
        let mut entity = crate::store::Entity::new();
        for (k, v) in c.borrow().iter() {
            entity.set(k.clone(), v.clone());
        }
        entity
    })
}

/// Return the number of stored entities of the given type.
pub fn entity_count(entity_type: &str) -> usize {
    native_store::with_store(|s| s.entity_count(entity_type))
}

/// Return true if an entity of the given type and id exists in the store.
pub fn has_entity(entity_type: &str, id: &str) -> bool {
    native_store::with_store(|s| s.has_entity(entity_type, id))
}

/// Assert that a contract at `address` was called with the selector for `signature`.
///
/// Only tracks calls made through [`MockHost::ethereum_call_raw`] in this module.
///
/// # Panics
/// Panics if no matching call was recorded.
pub fn assert_called(address: [u8; 20], signature: &str) {
    let sel = crate::crypto::selector(signature);
    let found = ETH_CALLS_MADE.with(|c| {
        c.borrow().iter().any(|(addr, data)| {
            *addr == address && data.starts_with(&sel)
        })
    });
    assert!(
        found,
        "Expected a call to {} with selector {:02x?} ({}), but none was recorded.",
        hex_addr(address),
        sel,
        signature,
    );
}

/// Return the number of times `address` was called with the selector for `signature`.
pub fn call_count(address: [u8; 20], signature: &str) -> usize {
    let sel = crate::crypto::selector(signature);
    ETH_CALLS_MADE.with(|c| {
        c.borrow()
            .iter()
            .filter(|(addr, data)| *addr == address && data.starts_with(&sel))
            .count()
    })
}

/// Assert that a data source template was created with the given name and address.
///
/// Checks the [`MockHost::created_data_sources`] list. If you are using the
/// `graphite::testing::MockHost` (not this module's `MockHost`), use
/// `host.assert_data_source_created()` instead.
///
/// # Panics
/// Panics if no matching `data_source_create` call was recorded.
pub fn assert_data_source_created(host: &MockHost, name: &str, address: [u8; 20]) {
    let hex = format!(
        "0x{}",
        address.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    );
    let found = host.created_data_sources.iter().any(|(n, params)| {
        n == name && params.first().map(|p| p == &hex).unwrap_or(false)
    });
    assert!(
        found,
        "Expected data source '{}' to be created for address {}, but it wasn't.\nCreated: {:?}",
        name, hex, host.created_data_sources
    );
}

fn hex_addr(addr: [u8; 20]) -> String {
    format!(
        "0x{}",
        addr.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    )
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
                b.as_slice(),
                expected,
                "Entity {}/{} field '{}': bytes mismatch",
                self.entity_type,
                self.id,
                field
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
                b.as_slice(),
                expected,
                "Entity {}/{} field '{}': BigInt bytes mismatch",
                self.entity_type,
                self.id,
                field
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

    /// Data source context (string key-value pairs).
    pub current_context: HashMap<String, String>,

    /// Current data source ID.
    pub current_id: String,
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
fn entity_to_native_fields(entity: &Entity) -> HashMap<String, FieldValue> {
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
        Value::Int8(n) => FieldValue::Int8(*n),
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
                        FieldValue::Int8(n) => Value::Int8(*n),
                        FieldValue::Null => Value::Null,
                    };
                    e.set(k.clone(), v);
                }
                e
            })
        })
    }

    fn store_get_in_block(&self, entity_type: &str, id: &str) -> Option<Entity> {
        // Native tests have no block/committed distinction — delegate to store_get.
        self.store_get(entity_type, id)
    }

    fn store_remove(&mut self, entity_type: &str, id: &str) {
        native_store::with_store_mut(|s| s.remove_entity(entity_type, id));
    }

    fn ethereum_call_raw(
        &self,
        address: Address,
        calldata: &[u8],
    ) -> Result<Bytes, EthereumCallError> {
        let addr_bytes: [u8; 20] = address.as_slice().try_into().expect("address is 20 bytes");
        ETH_CALLS_MADE.with(|c| c.borrow_mut().push((addr_bytes, calldata.to_vec())));
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
        // Check instance content first, then the module-level thread-local.
        if let Some(content) = self.ipfs_content.get(hash) {
            return Ok(content.clone());
        }
        IPFS_CONTENT.with(|c| {
            c.borrow()
                .get(hash)
                .cloned()
                .map(Bytes::from)
                .ok_or_else(|| IpfsError::NotFound(hash.to_string()))
        })
    }

    fn ens_name_by_address(&self, address: Address) -> Option<String> {
        let addr: [u8; 20] = address.as_slice().try_into().expect("address is 20 bytes");
        get_ens_name(&addr)
    }

    fn data_source_create(&mut self, name: &str, params: &[String]) {
        self.created_data_sources
            .push((name.to_string(), params.to_vec()));
    }

    fn data_source_create_with_context(&mut self, name: &str, params: &[String], context: Entity) {
        self.created_data_sources
            .push((name.to_string(), params.to_vec()));
        // Store context so the next data_source_context() call returns it.
        self.current_context.clear();
        for (k, v) in context.iter() {
            if let Value::String(s) = v {
                self.current_context.insert(k.clone(), s.clone());
            }
        }
    }

    fn data_source_address(&self) -> Address {
        self.current_address
    }

    fn data_source_network(&self) -> String {
        self.current_network.clone()
    }

    fn data_source_context(&self) -> Entity {
        let mut entity = Entity::new();
        for (k, v) in &self.current_context {
            entity.set(k.clone(), v.clone());
        }
        entity
    }

    fn data_source_id(&self) -> String {
        self.current_id.clone()
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
        assert_entity("User", "0x01").field_string("name", "Alice");
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
