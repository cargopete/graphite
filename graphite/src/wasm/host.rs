//! WASM host implementation using graph-node FFI.
//!
//! This implements the `HostFunctions` trait by calling through to
//! graph-node's host functions.

use crate::host::{EthereumCallError, HostFunctions, IpfsError, LogLevel};
use crate::primitives::{Address, Bytes};
use crate::store::{Entity, Value};
use crate::wasm::alloc::{alloc_bytes, alloc_string, read_bytes, read_string};
use crate::wasm::ffi::{self, AscPtr};
use alloc::string::String;
use alloc::vec::Vec;

/// Host implementation for WASM runtime.
///
/// This is a zero-sized type that calls through to graph-node's
/// imported host functions.
pub struct WasmHost;

impl WasmHost {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WasmHost {
    fn default() -> Self {
        Self::new()
    }
}

impl HostFunctions for WasmHost {
    fn store_set(&mut self, entity_type: &str, id: &str, entity: Entity) {
        let entity_type_ptr = AscPtr(alloc_string(entity_type));
        let id_ptr = AscPtr(alloc_string(id));
        let data_ptr = entity_to_asc(&entity);

        unsafe {
            ffi::store_set(entity_type_ptr, id_ptr, data_ptr);
        }
    }

    fn store_get(&self, entity_type: &str, id: &str) -> Option<Entity> {
        let entity_type_ptr = AscPtr(alloc_string(entity_type));
        let id_ptr = AscPtr(alloc_string(id));

        let result = unsafe { ffi::store_get(entity_type_ptr, id_ptr) };

        if result.is_null() {
            None
        } else {
            Some(asc_to_entity(result))
        }
    }

    fn store_remove(&mut self, entity_type: &str, id: &str) {
        let entity_type_ptr = AscPtr(alloc_string(entity_type));
        let id_ptr = AscPtr(alloc_string(id));

        unsafe {
            ffi::store_remove(entity_type_ptr, id_ptr);
        }
    }

    fn ethereum_call(
        &self,
        _address: Address,
        _function_signature: &str,
        _params: &[Value],
    ) -> Result<Vec<Value>, EthereumCallError> {
        // TODO: Implement ethereum_call
        // This requires marshalling the call parameters into AS format
        Err(EthereumCallError::Failed(
            "ethereum_call not yet implemented".into(),
        ))
    }

    fn crypto_keccak256(&self, input: &[u8]) -> [u8; 32] {
        let input_ptr = AscPtr(alloc_bytes(input));

        let result_ptr = unsafe { ffi::crypto_keccak256(input_ptr) };

        let result_bytes = unsafe { read_bytes(result_ptr.0) };
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result_bytes[..32]);
        hash
    }

    fn log(&self, level: LogLevel, message: &str) {
        let level_int = match level {
            LogLevel::Debug => ffi::LOG_LEVEL_DEBUG,
            LogLevel::Info => ffi::LOG_LEVEL_INFO,
            LogLevel::Warning => ffi::LOG_LEVEL_WARNING,
            LogLevel::Error => ffi::LOG_LEVEL_ERROR,
            LogLevel::Critical => ffi::LOG_LEVEL_CRITICAL,
        };

        let message_ptr = AscPtr(alloc_string(message));

        unsafe {
            ffi::log_log(level_int, message_ptr);
        }
    }

    fn ipfs_cat(&self, hash: &str) -> Result<Bytes, IpfsError> {
        let hash_ptr = AscPtr(alloc_string(hash));

        let result_ptr = unsafe { ffi::ipfs_cat(hash_ptr) };

        if result_ptr.is_null() {
            Err(IpfsError::NotFound(hash.to_string()))
        } else {
            let bytes = unsafe { read_bytes(result_ptr.0) };
            Ok(Bytes::from_slice(bytes))
        }
    }

    fn data_source_create(&mut self, name: &str, params: &[String]) {
        let name_ptr = AscPtr(alloc_string(name));
        let params_ptr = string_array_to_asc(params);

        unsafe {
            ffi::data_source_create(name_ptr, params_ptr);
        }
    }

    fn data_source_address(&self) -> Address {
        let result_ptr = unsafe { ffi::data_source_address() };
        let bytes = unsafe { read_bytes(result_ptr.0) };

        if bytes.len() >= 20 {
            Address::from_slice(&bytes[..20])
        } else {
            Address::ZERO
        }
    }

    fn data_source_network(&self) -> String {
        let result_ptr = unsafe { ffi::data_source_network() };
        let s = unsafe { read_string(result_ptr.0) };
        s.to_string()
    }
}

// ============================================================================
// AS Memory Marshalling
// ============================================================================

/// Convert an Entity to AS TypedMap format.
fn entity_to_asc(entity: &Entity) -> AscPtr {
    // For now, we allocate a simple representation
    // TODO: Full AS TypedMap marshalling
    //
    // This is a simplified version - real implementation needs to match
    // graph-node's expected TypedMap<String, Value> layout

    // Allocate array for entries
    let entries: Vec<_> = entity.iter().collect();
    let _count = entries.len();

    // For now just allocate a placeholder
    // Real implementation would serialize each key-value pair
    AscPtr(alloc_bytes(&[]))
}

/// Convert AS TypedMap to Entity.
fn asc_to_entity(_ptr: AscPtr) -> Entity {
    // TODO: Full AS TypedMap unmarshalling
    Entity::new()
}

/// Convert a string array to AS Array<String> format.
fn string_array_to_asc(strings: &[String]) -> AscPtr {
    // TODO: Full AS Array marshalling
    let _ = strings;
    AscPtr(alloc_bytes(&[]))
}

// ============================================================================
// Panic handler (only when std is disabled)
// ============================================================================

// When building without std, we need our own panic handler.
// With std enabled, the std panic handler is used.
// To build a no_std subgraph, users would add:
//   #![no_std]
//   #![no_main]
// and enable the wasm feature without std.
