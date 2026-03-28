//! FFI declarations for graph-node Rust ABI.
//!
//! These host functions use a clean ptr+len protocol, NOT the
//! AssemblyScript AscPtr<T> memory layout.
//!
//! # Protocol
//!
//! - Strings are passed as (ptr: u32, len: u32) pointing to UTF-8 bytes
//! - Byte arrays are passed as (ptr: u32, len: u32)
//! - Entities are bincode-serialized and passed as (ptr: u32, len: u32)
//! - For return values, caller provides a buffer and receives actual length
//!
//! # Graph-node Requirements
//!
//! Graph-node must be modified to:
//! 1. Detect `language: wasm/rust` in subgraph.yaml
//! 2. Register these Rust ABI host functions instead of AS ABI functions
//! 3. Handle bincode serialization for entities

// ============================================================================
// Graph-node host function imports (Rust ABI)
// ============================================================================

#[link(wasm_import_module = "graphite")]
unsafe extern "C" {
    // ========== Store Operations ==========

    /// Store an entity.
    ///
    /// - entity_type_ptr/len: UTF-8 entity type name
    /// - id_ptr/len: UTF-8 entity ID
    /// - data_ptr/len: bincode-serialized entity fields
    pub fn store_set(
        entity_type_ptr: u32,
        entity_type_len: u32,
        id_ptr: u32,
        id_len: u32,
        data_ptr: u32,
        data_len: u32,
    );

    /// Load an entity.
    ///
    /// - entity_type_ptr/len: UTF-8 entity type name
    /// - id_ptr/len: UTF-8 entity ID
    /// - out_ptr: buffer to write bincode-serialized entity
    /// - out_cap: capacity of output buffer
    ///
    /// Returns: actual length written, or 0 if not found, or u32::MAX if buffer too small
    pub fn store_get(
        entity_type_ptr: u32,
        entity_type_len: u32,
        id_ptr: u32,
        id_len: u32,
        out_ptr: u32,
        out_cap: u32,
    ) -> u32;

    /// Remove an entity.
    pub fn store_remove(
        entity_type_ptr: u32,
        entity_type_len: u32,
        id_ptr: u32,
        id_len: u32,
    );

    // ========== Ethereum Operations ==========

    /// Make a raw read-only contract call.
    ///
    /// - addr_ptr/len: 20-byte address
    /// - data_ptr/len: raw calldata (pre-ABI-encoded by SDK)
    /// - out_ptr/out_cap: buffer for raw return data
    ///
    /// Returns:
    /// - actual length on success
    /// - 0 on revert (call reverted, no data)
    /// - u32::MAX if buffer too small or error
    pub fn ethereum_call(
        addr_ptr: u32,
        addr_len: u32,
        data_ptr: u32,
        data_len: u32,
        out_ptr: u32,
        out_cap: u32,
    ) -> u32;

    // ========== Crypto Operations ==========

    /// Compute keccak256 hash.
    ///
    /// - input_ptr/len: data to hash
    /// - out_ptr: 32-byte buffer for hash output
    pub fn crypto_keccak256(input_ptr: u32, input_len: u32, out_ptr: u32);

    // ========== Logging ==========

    /// Log a message.
    ///
    /// - level: 0=debug, 1=info, 2=warning, 3=error, 4=critical
    /// - message_ptr/len: UTF-8 message
    pub fn log_log(level: u32, message_ptr: u32, message_len: u32);

    // ========== Data Source Operations ==========

    /// Create a new data source from a template.
    ///
    /// - name_ptr/len: UTF-8 template name
    /// - params_ptr/len: bincode-serialized Vec<String>
    pub fn data_source_create(
        name_ptr: u32,
        name_len: u32,
        params_ptr: u32,
        params_len: u32,
    );

    /// Get the address of the current data source.
    ///
    /// - out_ptr: 20-byte buffer for address
    pub fn data_source_address(out_ptr: u32);

    /// Get the network name.
    ///
    /// - out_ptr/out_cap: buffer for UTF-8 network name
    ///
    /// Returns: actual length
    pub fn data_source_network(out_ptr: u32, out_cap: u32) -> u32;

    // ========== IPFS Operations ==========

    /// Fetch content from IPFS.
    ///
    /// - hash_ptr/len: UTF-8 IPFS hash
    /// - out_ptr/out_cap: buffer for content
    ///
    /// Returns: actual length, or u32::MAX on error
    pub fn ipfs_cat(
        hash_ptr: u32,
        hash_len: u32,
        out_ptr: u32,
        out_cap: u32,
    ) -> u32;

    // ========== Abort ==========

    /// Abort execution with a message.
    pub fn abort(message_ptr: u32, message_len: u32, file_ptr: u32, file_len: u32, line: u32) -> !;
}

// ============================================================================
// Log levels
// ============================================================================

pub const LOG_LEVEL_DEBUG: u32 = 0;
pub const LOG_LEVEL_INFO: u32 = 1;
pub const LOG_LEVEL_WARNING: u32 = 2;
pub const LOG_LEVEL_ERROR: u32 = 3;
pub const LOG_LEVEL_CRITICAL: u32 = 4;
