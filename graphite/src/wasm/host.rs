//! WASM host implementation using graph-node Rust ABI.
//!
//! Implements `HostFunctions` by calling through to graph-node's
//! Rust-native host functions (ptr+len protocol, bincode serialization).

use crate::host::{EthereumCallError, HostFunctions, IpfsError, LogLevel};
use crate::primitives::{Address, Bytes};
use crate::store::{Entity, Value};
use crate::wasm::alloc::{alloc_slice, alloc_str, allocate};
use crate::wasm::ffi;
use alloc::string::String;
use alloc::vec::Vec;

/// Host implementation for WASM runtime (Rust ABI).
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
        let entity_type_ptr = alloc_str(entity_type);
        let id_ptr = alloc_str(id);

        // Serialize entity to bincode
        let entity_bytes = serialize_entity(&entity);
        let data_ptr = alloc_slice(&entity_bytes);

        unsafe {
            ffi::store_set(
                entity_type_ptr,
                entity_type.len() as u32,
                id_ptr,
                id.len() as u32,
                data_ptr,
                entity_bytes.len() as u32,
            );
        }
    }

    fn store_get(&self, entity_type: &str, id: &str) -> Option<Entity> {
        let entity_type_ptr = alloc_str(entity_type);
        let id_ptr = alloc_str(id);

        // First attempt with 16KB buffer
        const INITIAL_CAP: u32 = 16 * 1024;
        let out_ptr = allocate(INITIAL_CAP);

        let len = unsafe {
            ffi::store_get(
                entity_type_ptr,
                entity_type.len() as u32,
                id_ptr,
                id.len() as u32,
                out_ptr,
                INITIAL_CAP,
            )
        };

        if len == 0 {
            return None; // Entity not found
        }

        if len == u32::MAX {
            // Buffer too small — retry with a larger buffer (256KB)
            const RETRY_CAP: u32 = 256 * 1024;
            let retry_ptr = allocate(RETRY_CAP);
            let retry_type_ptr = alloc_str(entity_type);
            let retry_id_ptr = alloc_str(id);

            let retry_len = unsafe {
                ffi::store_get(
                    retry_type_ptr,
                    entity_type.len() as u32,
                    retry_id_ptr,
                    id.len() as u32,
                    retry_ptr,
                    RETRY_CAP,
                )
            };

            if retry_len == 0 || retry_len == u32::MAX {
                return None;
            }

            let bytes =
                unsafe { core::slice::from_raw_parts(retry_ptr as *const u8, retry_len as usize) };
            return deserialize_entity(bytes);
        }

        let bytes = unsafe { core::slice::from_raw_parts(out_ptr as *const u8, len as usize) };
        deserialize_entity(bytes)
    }

    fn store_remove(&mut self, entity_type: &str, id: &str) {
        let entity_type_ptr = alloc_str(entity_type);
        let id_ptr = alloc_str(id);

        unsafe {
            ffi::store_remove(
                entity_type_ptr,
                entity_type.len() as u32,
                id_ptr,
                id.len() as u32,
            );
        }
    }

    fn ethereum_call_raw(
        &self,
        address: Address,
        calldata: &[u8],
    ) -> Result<Bytes, EthereumCallError> {
        let address_ptr = alloc_slice(address.as_slice());
        let calldata_ptr = alloc_slice(calldata);

        // Allocate output buffer (4KB for most returns)
        const OUT_CAP: u32 = 4 * 1024;
        let out_ptr = allocate(OUT_CAP);

        let len = unsafe {
            ffi::ethereum_call(
                address_ptr,
                address.as_slice().len() as u32,
                calldata_ptr,
                calldata.len() as u32,
                out_ptr,
                OUT_CAP,
            )
        };

        if len == 0 {
            // 0 indicates revert
            return Err(EthereumCallError::Reverted);
        }

        if len == u32::MAX {
            // u32::MAX indicates buffer too small or error
            return Err(EthereumCallError::Failed("response too large".into()));
        }

        let bytes = unsafe { core::slice::from_raw_parts(out_ptr as *const u8, len as usize) };
        Ok(Bytes::from_slice(bytes))
    }

    fn crypto_keccak256(&self, input: &[u8]) -> [u8; 32] {
        let input_ptr = alloc_slice(input);
        let out_ptr = allocate(32);

        unsafe {
            ffi::crypto_keccak256(input_ptr, input.len() as u32, out_ptr);
            let mut hash = [0u8; 32];
            core::ptr::copy_nonoverlapping(out_ptr as *const u8, hash.as_mut_ptr(), 32);
            hash
        }
    }

    fn log(&self, level: LogLevel, message: &str) {
        let level_int = match level {
            LogLevel::Debug => ffi::LOG_LEVEL_DEBUG,
            LogLevel::Info => ffi::LOG_LEVEL_INFO,
            LogLevel::Warning => ffi::LOG_LEVEL_WARNING,
            LogLevel::Error => ffi::LOG_LEVEL_ERROR,
            LogLevel::Critical => ffi::LOG_LEVEL_CRITICAL,
        };

        let message_ptr = alloc_str(message);

        unsafe {
            ffi::log_log(level_int, message_ptr, message.len() as u32);
        }
    }

    fn ipfs_cat(&self, hash: &str) -> Result<Bytes, IpfsError> {
        let hash_ptr = alloc_str(hash);

        // Allocate output buffer (1MB max for IPFS content)
        const OUT_CAP: u32 = 1024 * 1024;
        let out_ptr = allocate(OUT_CAP);

        let len = unsafe { ffi::ipfs_cat(hash_ptr, hash.len() as u32, out_ptr, OUT_CAP) };

        if len == u32::MAX {
            return Err(IpfsError::NotFound(hash.to_string()));
        }

        let bytes = unsafe { core::slice::from_raw_parts(out_ptr as *const u8, len as usize) };
        Ok(Bytes::from_slice(bytes))
    }

    fn data_source_create(&mut self, name: &str, params: &[String]) {
        let name_ptr = alloc_str(name);

        // Serialize params
        let params_bytes = serialize_string_vec(params);
        let params_ptr = alloc_slice(&params_bytes);

        unsafe {
            ffi::data_source_create(
                name_ptr,
                name.len() as u32,
                params_ptr,
                params_bytes.len() as u32,
            );
        }
    }

    fn data_source_address(&self) -> Address {
        let out_ptr = allocate(20);

        unsafe {
            ffi::data_source_address(out_ptr);
            Address::from_slice(core::slice::from_raw_parts(out_ptr as *const u8, 20))
        }
    }

    fn data_source_network(&self) -> String {
        const OUT_CAP: u32 = 256;
        let out_ptr = allocate(OUT_CAP);

        let len = unsafe { ffi::data_source_network(out_ptr, OUT_CAP) };

        let bytes = unsafe { core::slice::from_raw_parts(out_ptr as *const u8, len as usize) };
        String::from_utf8_lossy(bytes).into_owned()
    }
}

// ============================================================================
// Serialization helpers
// ============================================================================

/// Serialize an Entity to bytes (simple format for now).
///
/// Format: [field_count: u32] [field_name_len: u32, field_name: bytes, value_type: u8, value_data...]
fn serialize_entity(entity: &Entity) -> Vec<u8> {
    use alloc::vec;

    let mut buf = vec![];

    // Field count
    let count = entity.len() as u32;
    buf.extend_from_slice(&count.to_le_bytes());

    for (key, value) in entity.iter() {
        // Key length + key
        let key_bytes = key.as_bytes();
        buf.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(key_bytes);

        // Value
        serialize_value(&mut buf, value);
    }

    buf
}

fn serialize_value(buf: &mut Vec<u8>, value: &Value) {
    match value {
        Value::String(s) => {
            buf.push(0x01);
            let bytes = s.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(bytes);
        }
        Value::Int(n) => {
            buf.push(0x02);
            buf.extend_from_slice(&n.to_le_bytes());
        }
        Value::Int8(n) => {
            buf.push(0x03);
            buf.extend_from_slice(&n.to_le_bytes());
        }
        Value::BigInt(n) => {
            buf.push(0x04);
            let bytes = n.to_signed_bytes_le();
            buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(&bytes);
        }
        Value::BigDecimal(_) => {
            buf.push(0x05);
            // TODO: proper BigDecimal serialization
            buf.extend_from_slice(&0u32.to_le_bytes());
        }
        Value::Bool(b) => {
            buf.push(0x06);
            buf.push(if *b { 1 } else { 0 });
        }
        Value::Bytes(b) => {
            buf.push(0x07);
            buf.extend_from_slice(&(b.len() as u32).to_le_bytes());
            buf.extend_from_slice(b.as_slice());
        }
        Value::Address(a) => {
            buf.push(0x08);
            buf.extend_from_slice(a.as_slice());
        }
        Value::Array(arr) => {
            buf.push(0x09);
            buf.extend_from_slice(&(arr.len() as u32).to_le_bytes());
            for v in arr {
                serialize_value(buf, v);
            }
        }
        Value::Null => {
            buf.push(0x00);
        }
    }
}

/// Deserialize an Entity from bytes.
fn deserialize_entity(bytes: &[u8]) -> Option<Entity> {
    if bytes.len() < 4 {
        return None;
    }

    let mut entity = Entity::new();
    let mut pos = 0;

    // Field count
    let count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
    pos += 4;

    for _ in 0..count {
        // Key
        let key_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;
        let key = core::str::from_utf8(&bytes[pos..pos + key_len]).ok()?;
        pos += key_len;

        // Value
        let (value, consumed) = deserialize_value(&bytes[pos..])?;
        pos += consumed;

        entity.set(key, value);
    }

    Some(entity)
}

fn deserialize_value(bytes: &[u8]) -> Option<(Value, usize)> {
    use crate::primitives::BigInt;

    if bytes.is_empty() {
        return None;
    }

    let tag = bytes[0];
    let mut pos = 1;

    let value = match tag {
        0x00 => Value::Null,
        0x01 => {
            // String
            let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            let s = core::str::from_utf8(&bytes[pos..pos + len]).ok()?;
            pos += len;
            Value::String(s.into())
        }
        0x02 => {
            // Int
            let n = i32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?);
            pos += 4;
            Value::Int(n)
        }
        0x03 => {
            // Int8
            let n = i64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
            pos += 8;
            Value::Int8(n)
        }
        0x04 => {
            // BigInt
            let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            let n = BigInt::from_signed_bytes_be(&bytes[pos..pos + len]);
            pos += len;
            Value::BigInt(n)
        }
        0x06 => {
            // Bool
            let b = bytes[pos] != 0;
            pos += 1;
            Value::Bool(b)
        }
        0x07 => {
            // Bytes
            let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            let b = Bytes::from_slice(&bytes[pos..pos + len]);
            pos += len;
            Value::Bytes(b)
        }
        0x08 => {
            // Address
            let addr = Address::from_slice(&bytes[pos..pos + 20]);
            pos += 20;
            Value::Address(addr)
        }
        _ => return None,
    };

    Some((value, pos))
}

fn serialize_string_vec(strings: &[String]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    for s in strings {
        let bytes = s.as_bytes();
        buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(bytes);
    }
    buf
}
