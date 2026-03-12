//! FFI declarations for graph-node host functions.
//!
//! These are the raw `extern "C"` imports that graph-node provides.
//! Higher-level wrappers are in `host.rs`.

/// Pointer to AssemblyScript-style heap object.
/// Graph-node uses AS memory layout for compatibility.
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct AscPtr(pub u32);

impl AscPtr {
    pub const NULL: Self = Self(0);

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

// ============================================================================
// Graph-node host function imports
// ============================================================================
//
// These match the imports expected by graph-node's WASM runtime.
// See: https://github.com/graphprotocol/graph-node/blob/master/runtime/wasm/src/host_exports.rs

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    // ========== Store Operations ==========

    /// Store an entity.
    /// entity_type: AscPtr to String
    /// id: AscPtr to String
    /// data: AscPtr to Entity (TypedMap<String, Value>)
    pub fn store_set(entity_type: AscPtr, id: AscPtr, data: AscPtr);

    /// Load an entity. Returns AscPtr to Entity or null.
    pub fn store_get(entity_type: AscPtr, id: AscPtr) -> AscPtr;

    /// Remove an entity.
    pub fn store_remove(entity_type: AscPtr, id: AscPtr);

    // ========== Ethereum Operations ==========

    /// Make a read-only contract call.
    /// Returns AscPtr to result array or null on failure.
    pub fn ethereum_call(call: AscPtr) -> AscPtr;

    // ========== Crypto Operations ==========

    /// Compute keccak256 hash.
    /// input: AscPtr to Bytes
    /// Returns: AscPtr to Bytes (32 bytes)
    pub fn crypto_keccak256(input: AscPtr) -> AscPtr;

    // ========== Logging ==========

    /// Log a message.
    /// level: i32 (0=debug, 1=info, 2=warning, 3=error, 4=critical)
    /// message: AscPtr to String
    pub fn log_log(level: i32, message: AscPtr);

    // ========== Type Conversions ==========

    /// Convert BigInt to string.
    pub fn big_int_to_string(big_int: AscPtr) -> AscPtr;

    /// Convert string to BigInt.
    pub fn big_int_from_string(s: AscPtr) -> AscPtr;

    /// Convert BigInt to hex string.
    pub fn big_int_to_hex(big_int: AscPtr) -> AscPtr;

    // ========== Data Source Operations ==========

    /// Create a new data source from a template.
    /// name: AscPtr to String
    /// params: AscPtr to Array<String>
    pub fn data_source_create(name: AscPtr, params: AscPtr);

    /// Get the address of the current data source.
    /// Returns: AscPtr to Address (Bytes)
    pub fn data_source_address() -> AscPtr;

    /// Get the network name.
    /// Returns: AscPtr to String
    pub fn data_source_network() -> AscPtr;

    /// Get the context of the current data source.
    /// Returns: AscPtr to DataSourceContext
    pub fn data_source_context() -> AscPtr;

    // ========== IPFS Operations ==========

    /// Fetch content from IPFS.
    /// hash: AscPtr to String (IPFS hash)
    /// Returns: AscPtr to Bytes or null
    pub fn ipfs_cat(hash: AscPtr) -> AscPtr;

    // ========== JSON Operations ==========

    /// Parse a JSON string.
    /// json: AscPtr to String
    /// Returns: AscPtr to JSONValue
    pub fn json_from_bytes(bytes: AscPtr) -> AscPtr;

    /// Convert a JSON value to bytes.
    pub fn json_to_string(json: AscPtr) -> AscPtr;

    // ========== Abort ==========

    /// Abort execution with a message.
    /// message: AscPtr to String
    /// file_name: AscPtr to String
    /// line: i32
    /// column: i32
    pub fn abort(message: AscPtr, file_name: AscPtr, line: i32, column: i32) -> !;
}

// ============================================================================
// Log levels matching graph-node
// ============================================================================

pub const LOG_LEVEL_DEBUG: i32 = 0;
pub const LOG_LEVEL_INFO: i32 = 1;
pub const LOG_LEVEL_WARNING: i32 = 2;
pub const LOG_LEVEL_ERROR: i32 = 3;
pub const LOG_LEVEL_CRITICAL: i32 = 4;
