//! Host function abstractions.
//!
//! This module defines the trait interface for graph-node host functions,
//! allowing the same handler code to run against real host functions (in WASM)
//! or mock implementations (in native tests).

use crate::primitives::{Address, BigInt, Bytes};
use crate::store::Entity;
use alloc::string::String;

/// The core trait abstracting all graph-node host functions.
///
/// In WASM builds, this is implemented by direct FFI calls to graph-node.
/// In native test builds, this is implemented by `MockHost`.
pub trait HostFunctions {
    // ============ Store Operations ============

    /// Store an entity in the database.
    fn store_set(&mut self, entity_type: &str, id: &str, entity: Entity);

    /// Load an entity from the database.
    fn store_get(&self, entity_type: &str, id: &str) -> Option<Entity>;

    /// Remove an entity from the database.
    fn store_remove(&mut self, entity_type: &str, id: &str);

    // ============ Ethereum Operations ============

    /// Make a raw read-only call to a smart contract.
    /// Takes pre-encoded calldata and returns raw bytes.
    /// Use alloy-sol-types for ABI encoding/decoding.
    fn ethereum_call_raw(
        &self,
        address: Address,
        calldata: &[u8],
    ) -> Result<Bytes, EthereumCallError>;

    // ============ Crypto Operations ============

    /// Compute the Keccak-256 hash of the input.
    fn crypto_keccak256(&self, input: &[u8]) -> [u8; 32];

    // ============ Logging ============

    /// Log a message at the specified level.
    fn log(&self, level: LogLevel, message: &str);

    // ============ IPFS Operations ============

    /// Fetch content from IPFS by hash.
    fn ipfs_cat(&self, hash: &str) -> Result<Bytes, IpfsError>;

    // ============ Data Source Operations ============

    /// Create a new data source from a template.
    fn data_source_create(&mut self, name: &str, params: &[String]);

    /// Get the address of the current data source.
    fn data_source_address(&self) -> Address;

    /// Get the network name of the current data source.
    fn data_source_network(&self) -> String;

    // ============ Type Conversions ============
    // Note: Most conversions are handled natively in Rust, but some
    // may need host function calls for compatibility.

    /// Convert a BigInt to a string representation.
    fn big_int_to_string(&self, n: &BigInt) -> String {
        n.to_string()
    }
}

/// Log levels matching graph-node's log system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Error from an Ethereum contract call.
#[derive(Debug, Clone, thiserror::Error)]
pub enum EthereumCallError {
    #[error("contract call reverted")]
    Reverted,
    #[error("contract call failed: {0}")]
    Failed(String),
}

/// Error from IPFS operations.
#[derive(Debug, thiserror::Error)]
pub enum IpfsError {
    #[error("IPFS content not found: {0}")]
    NotFound(String),
    #[error("IPFS timeout")]
    Timeout,
    #[error("IPFS error: {0}")]
    Other(String),
}

/// Convenience macros for logging (available when a host context is in scope).
#[macro_export]
macro_rules! log_debug {
    ($host:expr, $($arg:tt)*) => {
        $host.log($crate::host::LogLevel::Debug, &alloc::format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($host:expr, $($arg:tt)*) => {
        $host.log($crate::host::LogLevel::Info, &alloc::format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warning {
    ($host:expr, $($arg:tt)*) => {
        $host.log($crate::host::LogLevel::Warning, &alloc::format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($host:expr, $($arg:tt)*) => {
        $host.log($crate::host::LogLevel::Error, &alloc::format!($($arg)*))
    };
}
