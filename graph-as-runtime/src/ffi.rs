//! FFI declarations for graph-node host functions (AS ABI).
//!
//! Graph-node registers these under `(module="env", name="store.set")` etc.
//! WASM field names can contain dots — LLD passes them through correctly on
//! wasm32-unknown-unknown.
//!
//! All pointers here are AscPtr values: u32 offsets into WASM linear memory
//! pointing at the *payload* of an AS object (16 bytes past the object header).
//!
//! # Store operations
//!
//! `store.set(entity: u32, id: u32, data: u32)`
//!   - entity: AscPtr<AscString> — entity type name (e.g. "Transfer")
//!   - id:     AscPtr<AscString> — entity ID
//!   - data:   AscPtr<TypedMap<string, Value>> — entity fields
//!
//! `store.get(entity: u32, id: u32) -> u32`
//!   Returns AscPtr<TypedMap<string, Value>> or 0 if not found.
//!
//! `store.remove(entity: u32, id: u32)`

// ============================================================================
// graph-node host function imports (AS ABI, module = "env")
// ============================================================================

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    // ========== Store ==========

    #[link_name = "store.set"]
    pub fn store_set(entity: u32, id: u32, data: u32);

    #[link_name = "store.get"]
    pub fn store_get(entity: u32, id: u32) -> u32;

    #[link_name = "store.remove"]
    pub fn store_remove(entity: u32, id: u32);

    // ========== Logging ==========

    /// `log.log(level: u32, message: u32)`
    /// level: 0=CRITICAL 1=ERROR 2=WARNING 3=INFO 4=DEBUG
    /// message: AscPtr<AscString>
    #[link_name = "log.log"]
    pub fn log_log(level: u32, message: u32);

    // ========== Ethereum ==========

    /// `ethereum.call(call: u32) -> u32`
    /// call:   AscPtr<SmartContractCall>
    /// returns AscPtr<Array<EthereumValue>> or 0 on revert
    #[link_name = "ethereum.call"]
    pub fn ethereum_call(call: u32) -> u32;

    // ========== Crypto ==========

    /// `crypto.keccak256(input: u32) -> u32`
    /// input:   AscPtr<Bytes>
    /// returns: AscPtr<Bytes> (32 bytes)
    #[link_name = "crypto.keccak256"]
    pub fn crypto_keccak256(input: u32) -> u32;

    // ========== IPFS ==========

    /// `ipfs.cat(hash: u32) -> u32`
    /// hash:    AscPtr<AscString>
    /// returns: AscPtr<Bytes> or 0 on error
    #[link_name = "ipfs.cat"]
    pub fn ipfs_cat(hash: u32) -> u32;

    // ========== Data Source ==========

    /// `dataSource.create(name: u32, params: u32)`
    /// name:   AscPtr<AscString>
    /// params: AscPtr<Array<AscString>>
    #[link_name = "dataSource.create"]
    pub fn data_source_create(name: u32, params: u32);

    /// `dataSource.address() -> u32` — AscPtr<Address>
    #[link_name = "dataSource.address"]
    pub fn data_source_address() -> u32;

    /// `dataSource.network() -> u32` — AscPtr<AscString>
    #[link_name = "dataSource.network"]
    pub fn data_source_network() -> u32;

    // ========== Abort ==========

    /// AS abort signature: `abort(msg: u32, file: u32, line: u32, col: u32)`
    /// All pointers are AscPtr<AscString> (or 0).
    pub fn abort(msg: u32, file: u32, line: u32, col: u32);
}

// ============================================================================
// Log level constants (match graph-node's LogLevel enum order)
// ============================================================================

pub const LOG_CRITICAL: u32 = 0;
pub const LOG_ERROR: u32 = 1;
pub const LOG_WARNING: u32 = 2;
pub const LOG_INFO: u32 = 3;
pub const LOG_DEBUG: u32 = 4;
