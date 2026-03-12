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

    /// Mock responses for ethereum calls.
    pub eth_calls: HashMap<(Address, String), Result<Vec<Value>, EthereumCallError>>,

    /// Mock responses for IPFS fetches.
    pub ipfs_content: HashMap<String, Bytes>,

    /// Captured log messages.
    pub logs: Vec<(LogLevel, String)>,

    /// Created data sources.
    pub created_data_sources: Vec<(String, Vec<String>)>,

    /// Current data source address.
    pub current_address: Address,

    /// Current network name.
    pub current_network: String,
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

    /// Mock an ethereum call response.
    pub fn mock_eth_call(
        &mut self,
        address: Address,
        signature: impl Into<String>,
        response: Result<Vec<Value>, EthereumCallError>,
    ) {
        self.eth_calls.insert((address, signature.into()), response);
    }

    /// Mock IPFS content.
    pub fn mock_ipfs(&mut self, hash: impl Into<String>, content: impl Into<Bytes>) {
        self.ipfs_content.insert(hash.into(), content.into());
    }

    /// Get all log messages at a specific level.
    pub fn logs_at(&self, level: LogLevel) -> Vec<&str> {
        self.logs
            .iter()
            .filter(|(l, _)| *l == level)
            .map(|(_, msg)| msg.as_str())
            .collect()
    }

    /// Clear all logs.
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
}

impl HostFunctions for MockHost {
    fn store_set(&mut self, entity_type: &str, id: &str, entity: Entity) {
        self.store.set(entity_type, id, entity);
    }

    fn store_get(&self, entity_type: &str, id: &str) -> Option<Entity> {
        self.store.get(entity_type, id)
    }

    fn store_remove(&mut self, entity_type: &str, id: &str) {
        self.store.remove(entity_type, id);
    }

    fn ethereum_call(
        &self,
        address: Address,
        function_signature: &str,
        _params: &[Value],
    ) -> Result<Vec<Value>, EthereumCallError> {
        self.eth_calls
            .get(&(address, function_signature.to_string()))
            .cloned()
            .unwrap_or(Err(EthereumCallError::Failed(
                "no mock configured".to_string(),
            )))
    }

    fn crypto_keccak256(&self, input: &[u8]) -> [u8; 32] {
        use alloy_primitives::keccak256;
        keccak256(input).0
    }

    fn log(&self, level: LogLevel, message: &str) {
        // Note: We need interior mutability here for a cleaner API.
        // For now, logs won't be captured in the basic implementation.
        // Users should use a RefCell wrapper or we redesign.
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
