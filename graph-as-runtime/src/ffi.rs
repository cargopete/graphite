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
// Only available when compiling to WASM — the symbols are provided by graph-node.
// ============================================================================

#[cfg(target_arch = "wasm32")]
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

    /// `ethereum.decode(types: u32, data: u32) -> u32`
    /// types:   AscPtr<AscString>  — Solidity type string e.g. `"(uint256,address)"`
    /// data:    AscPtr<Bytes>      — ABI-encoded bytes to decode
    /// returns: AscPtr<EthereumValue> or 0 on failure
    #[link_name = "ethereum.decode"]
    pub fn ethereum_decode(types: u32, data: u32) -> u32;

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

    // ========== JSON ==========

    /// `json.fromBytes(data: u32) -> u32`
    /// data:    AscPtr<Bytes>
    /// returns: AscPtr<JSONValue> (panics in graph-node on invalid JSON)
    #[link_name = "json.fromBytes"]
    pub fn json_from_bytes(data: u32) -> u32;

    // ========== ENS ==========

    /// `ens.nameByAddress(address: u32) -> u32`
    /// address: AscPtr<Address> (20-byte Uint8Array)
    /// returns: AscPtr<AscString> (nullable — 0 if no name registered)
    #[link_name = "ens.nameByAddress"]
    pub fn ens_name_by_address(address: u32) -> u32;

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
// Native stubs — used when running under `cargo test` (no WASM runtime).
// These forward to the thread-local native_store.
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn store_set(_entity: u32, _id: u32, _data: u32) {
    // The low-level AS-ABI path is not used in native tests.
    // Use graph_as_runtime::native_store directly, or the graphite MockHost.
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn store_get(_entity: u32, _id: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn store_remove(_entity: u32, _id: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn log_log(level: u32, _message: u32) {
    // Print a note so tests can see log calls without needing the WASM runtime.
    let level_str = match level {
        LOG_CRITICAL => "CRITICAL",
        LOG_ERROR => "ERROR",
        LOG_WARNING => "WARNING",
        LOG_INFO => "INFO",
        LOG_DEBUG => "DEBUG",
        _ => "UNKNOWN",
    };
    eprintln!("[graph-node log] level={}", level_str);
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ethereum_call(_call: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ethereum_decode(_types: u32, _data: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn crypto_keccak256(_input: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ipfs_cat(_hash: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn json_from_bytes(_data: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ens_name_by_address(_address: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn data_source_create(_name: u32, _params: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn data_source_address() -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn data_source_network() -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn abort(_msg: u32, _file: u32, _line: u32, _col: u32) {
    panic!("graph-node abort called");
}

// ============================================================================
// Log level constants (match graph-node's LogLevel enum order)
// ============================================================================

pub const LOG_CRITICAL: u32 = 0;
pub const LOG_ERROR: u32 = 1;
pub const LOG_WARNING: u32 = 2;
pub const LOG_INFO: u32 = 3;
pub const LOG_DEBUG: u32 = 4;
