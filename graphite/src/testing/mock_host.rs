//! Mock implementation of HostFunctions for native testing.

use crate::host::{EthereumCallError, HostFunctions, IpfsError, LogLevel};
use crate::primitives::{Address, Bytes};
use crate::store::{Entity, Value};
use std::collections::HashMap;

/// A mock implementation of graph-node host functions for testing.
///
/// Provides an in-memory store and configurable mocks for ethereum calls,
/// IPFS fetches, and other host operations.
///
/// # Example
///
/// ```rust
/// use graphite::testing::MockHost;
///
/// let mut host = MockHost::default();
/// // Run your handler with &mut host
/// // Then inspect host.store for assertions
/// ```
#[derive(Debug, Default)]
pub struct MockHost {
    /// The in-memory entity store.
    pub store: MockStore,

    /// Mock responses for raw ethereum calls.
    /// Key is (address, calldata), value is raw response bytes or error.
    pub eth_calls: HashMap<(Address, Vec<u8>), Result<Bytes, EthereumCallError>>,

    /// Record of every `ethereum_call_raw` invocation: (address, calldata).
    /// Uses interior mutability so it can be updated through `&self`.
    pub eth_calls_made: std::cell::RefCell<Vec<(Address, Vec<u8>)>>,

    /// Mock responses for IPFS fetches.
    pub ipfs_content: HashMap<String, Bytes>,

    /// Captured log messages (use `get_logs()` or `logs_at()` to inspect).
    pub logs: std::cell::RefCell<Vec<(LogLevel, String)>>,

    /// Created data sources.
    pub created_data_sources: Vec<(String, Vec<String>)>,

    /// Current data source address.
    pub current_address: Address,

    /// Current network name.
    pub current_network: String,

    /// Mock ENS name-by-address registry.
    pub ens_names: HashMap<Address, String>,

    /// Current data source context (key-value pairs set via createWithContext).
    pub current_context: Entity,

    /// Current data source ID (defaults to empty string).
    pub current_id: String,
}

impl MockHost {
    /// Create a new mock host with default settings.
    pub fn new() -> Self {
        Self {
            current_network: "mainnet".to_string(),
            ..Default::default()
        }
    }

    /// Set the current data source address.
    pub fn with_address(mut self, address: Address) -> Self {
        self.current_address = address;
        self
    }

    /// Set the current network.
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.current_network = network.into();
        self
    }

    /// Set the current data source ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.current_id = id.into();
        self
    }

    /// Set the current data source context.
    pub fn with_context(mut self, context: Entity) -> Self {
        self.current_context = context;
        self
    }

    /// Mock a raw ethereum call response.
    /// Use alloy-sol-types to encode the expected calldata and response.
    pub fn mock_eth_call_raw(
        &mut self,
        address: Address,
        calldata: impl Into<Vec<u8>>,
        response: Result<Bytes, EthereumCallError>,
    ) {
        self.eth_calls.insert((address, calldata.into()), response);
    }

    /// Mock IPFS content.
    pub fn mock_ipfs(&mut self, hash: impl Into<String>, content: impl Into<Bytes>) {
        self.ipfs_content.insert(hash.into(), content.into());
    }

    /// Register a mock ENS name for an address.
    pub fn mock_ens_name(&mut self, address: Address, name: impl Into<String>) {
        self.ens_names.insert(address, name.into());
    }

    // ---- Call tracking helpers ----

    /// Assert that `address` was called with the selector for `signature`.
    ///
    /// Panics if no matching call was recorded during the test.
    ///
    /// # Example
    /// ```rust,ignore
    /// host.assert_called(token_addr, "balanceOf(address)");
    /// ```
    pub fn assert_called(&self, address: Address, signature: &str) {
        let sel = crate::crypto::selector(signature);
        let made = self.eth_calls_made.borrow();
        let found = made.iter().any(|(addr, data)| {
            *addr == address && data.starts_with(&sel)
        });
        assert!(
            found,
            "Expected a call to {:?} with selector {:?} ({}), but none was recorded.\nCalls made: {}",
            address,
            sel,
            signature,
            made.iter()
                .map(|(a, d)| {
                    let hex: String = d.iter().map(|b| format!("{:02x}", b)).collect();
                    format!("{:?} data=0x{}", a, hex)
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    /// Return how many times `address` was called with the selector for `signature`.
    pub fn call_count(&self, address: Address, signature: &str) -> usize {
        let sel = crate::crypto::selector(signature);
        self.eth_calls_made
            .borrow()
            .iter()
            .filter(|(addr, data)| *addr == address && data.starts_with(&sel))
            .count()
    }

    /// Assert that a data source was created with the given name and address.
    ///
    /// Panics if no matching `data_source_create` call was recorded.
    pub fn assert_data_source_created(&self, name: &str, address: Address) {
        let hex: String = core::iter::once("0x".to_string())
            .chain(address.as_slice().iter().map(|b| format!("{:02x}", b)))
            .collect();
        let found = self.created_data_sources.iter().any(|(n, params)| {
            n == name && params.first().map(|p| p == &hex).unwrap_or(false)
        });
        assert!(
            found,
            "Expected data source '{}' to be created for address {}, but it wasn't.\nCreated: {:?}",
            name, hex, self.created_data_sources
        );
    }

    /// Get all captured log messages as a borrowed slice.
    pub fn get_logs(&self) -> std::cell::Ref<'_, Vec<(LogLevel, String)>> {
        self.logs.borrow()
    }

    /// Get all log messages at a specific level.
    pub fn logs_at(&self, level: LogLevel) -> Vec<String> {
        self.logs
            .borrow()
            .iter()
            .filter(|(l, _)| *l == level)
            .map(|(_, msg)| msg.clone())
            .collect()
    }

    /// Clear all logs.
    pub fn clear_logs(&mut self) {
        self.logs.borrow_mut().clear();
    }
}

impl HostFunctions for MockHost {
    fn store_set(&mut self, entity_type: &str, id: &str, entity: Entity) {
        self.store.set(entity_type, id, entity);
    }

    fn store_get(&self, entity_type: &str, id: &str) -> Option<Entity> {
        self.store.get(entity_type, id)
    }

    fn store_get_in_block(&self, entity_type: &str, id: &str) -> Option<Entity> {
        // Native tests have no block/committed distinction — delegate to store_get.
        self.store.get(entity_type, id)
    }

    fn store_remove(&mut self, entity_type: &str, id: &str) {
        self.store.remove(entity_type, id);
    }

    fn ethereum_call_raw(
        &self,
        address: Address,
        calldata: &[u8],
    ) -> Result<Bytes, EthereumCallError> {
        self.eth_calls_made
            .borrow_mut()
            .push((address, calldata.to_vec()));
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
        self.logs.borrow_mut().push((level, message.to_string()));
        eprintln!("[{:?}] {}", level, message);
    }

    fn ipfs_cat(&self, hash: &str) -> Result<Bytes, IpfsError> {
        self.ipfs_content
            .get(hash)
            .cloned()
            .ok_or_else(|| IpfsError::NotFound(hash.to_string()))
    }

    fn ens_name_by_address(&self, address: Address) -> Option<String> {
        self.ens_names.get(&address).cloned()
    }

    fn data_source_create(&mut self, name: &str, params: &[String]) {
        self.created_data_sources
            .push((name.to_string(), params.to_vec()));
    }

    fn data_source_create_with_context(&mut self, name: &str, params: &[String], context: Entity) {
        self.created_data_sources
            .push((name.to_string(), params.to_vec()));
        self.current_context = context;
    }

    fn data_source_address(&self) -> Address {
        self.current_address
    }

    fn data_source_network(&self) -> String {
        self.current_network.clone()
    }

    fn data_source_context(&self) -> Entity {
        self.current_context.clone()
    }

    fn data_source_id(&self) -> String {
        self.current_id.clone()
    }
}

/// In-memory entity store for testing.
#[derive(Debug, Default)]
pub struct MockStore {
    /// Nested map: entity_type -> id -> entity
    entities: HashMap<String, HashMap<String, Entity>>,
}

impl MockStore {
    /// Store an entity.
    pub fn set(&mut self, entity_type: &str, id: &str, entity: Entity) {
        self.entities
            .entry(entity_type.to_string())
            .or_default()
            .insert(id.to_string(), entity);
    }

    /// Load an entity.
    pub fn get(&self, entity_type: &str, id: &str) -> Option<Entity> {
        self.entities
            .get(entity_type)
            .and_then(|m| m.get(id))
            .cloned()
    }

    /// Remove an entity.
    pub fn remove(&mut self, entity_type: &str, id: &str) {
        if let Some(m) = self.entities.get_mut(entity_type) {
            m.remove(id);
        }
    }

    /// Count entities of a given type.
    pub fn entity_count(&self, entity_type: &str) -> usize {
        self.entities.get(entity_type).map(|m| m.len()).unwrap_or(0)
    }

    /// Get all entities of a given type.
    pub fn all_of_type(&self, entity_type: &str) -> Vec<(&String, &Entity)> {
        self.entities
            .get(entity_type)
            .map(|m| m.iter().collect())
            .unwrap_or_default()
    }

    /// Assert a field has a specific value.
    ///
    /// # Panics
    /// Panics if the entity doesn't exist or the field doesn't match.
    pub fn assert_field_equals(&self, entity_type: &str, id: &str, field: &str, expected: &Value) {
        let entity = self
            .get(entity_type, id)
            .unwrap_or_else(|| panic!("Entity {}/{} not found", entity_type, id));
        let actual = entity
            .get(field)
            .unwrap_or_else(|| panic!("Field {} not found on {}/{}", field, entity_type, id));
        assert_eq!(
            actual, expected,
            "Field {} on {}/{} mismatch",
            field, entity_type, id
        );
    }

    /// Clear all entities.
    pub fn clear(&mut self) {
        self.entities.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_store_basic_operations() {
        let mut host = MockHost::new();

        // Store an entity
        let mut entity = Entity::new();
        entity.set("name", "Alice");
        entity.set("balance", crate::primitives::BigInt::from(1000));
        host.store_set("User", "1", entity);

        // Retrieve it
        assert_eq!(host.store.entity_count("User"), 1);
        let loaded = host.store_get("User", "1").unwrap();
        assert_eq!(loaded.get("name"), Some(&Value::String("Alice".into())));

        // Remove it
        host.store_remove("User", "1");
        assert_eq!(host.store.entity_count("User"), 0);
    }

    #[test]
    fn mock_keccak256() {
        let host = MockHost::new();
        let hash = host.crypto_keccak256(b"hello");
        // Known keccak256 of "hello"
        assert_eq!(
            hex::encode(hash),
            "1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
    }
}
