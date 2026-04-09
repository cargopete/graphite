//! Reading AS TypedMap objects returned by `store.get`.
//!
//! When graph-node returns data from `store.get`, it writes AS objects into
//! our WASM linear memory — the same layout we use when writing via
//! `store.set`. This module walks that layout to decode it back into Rust.
//!
//! # Layout recap
//!
//! TypedMap → entries: AscPtr<Array<TypedMapEntry>>
//! Array    → buffer: AscPtr<ArrayBuffer>, length: u32
//! ArrayBuffer → [entry_ptr_0: u32, entry_ptr_1: u32, ...]
//! TypedMapEntry → key: AscPtr<String>, value: AscPtr<Value>
//! String   → [len: u32 (in header rt_size)] [UTF-16LE bytes...]
//! Value    → kind: u32, _pad: u32, payload: u64

use alloc::string::String;
use alloc::vec::Vec;

/// A decoded store field value.
#[derive(Debug, Clone)]
pub enum StoreValue {
    String(String),
    Bytes(Vec<u8>),
    BigInt(Vec<u8>),
    Bool(bool),
    Int(i32),
    Null,
}

impl StoreValue {
    pub fn as_string(&self) -> Option<&str> {
        if let StoreValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_bytes(&self) -> Option<Vec<u8>> {
        if let StoreValue::Bytes(b) | StoreValue::BigInt(b) = self {
            Some(b.clone())
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let StoreValue::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        if let StoreValue::Int(n) = self {
            Some(*n)
        } else {
            None
        }
    }
}

/// Decode an AS String (UTF-16LE) at the given AscPtr into a Rust String.
///
/// # Safety
/// `ptr` must be a valid AscPtr<AscString> as returned by graph-node.
pub unsafe fn read_asc_string(ptr: u32) -> String {
    if ptr == 0 {
        return String::new();
    }
    // rt_size is 4 bytes before the header start.
    // Header is 16 bytes before ptr; rt_size is at header+8, so ptr-8.
    let rt_size_ptr = (ptr - 8) as *const u32;
    let byte_len = unsafe { rt_size_ptr.read_unaligned() } as usize;

    if byte_len == 0 {
        return String::new();
    }

    // UTF-16LE bytes start at ptr.
    let data_ptr = ptr as *const u16;
    let char_count = byte_len / 2;
    let utf16: Vec<u16> = (0..char_count)
        .map(|i| unsafe { data_ptr.add(i).read_unaligned() })
        .collect();

    String::from_utf16_lossy(&utf16)
}

/// Decode the raw bytes of an AS Uint8Array-like object (Bytes/BigInt).
///
/// Layout: [buffer: u32][dataStart: u32][length: u32]
///
/// # Safety
/// `ptr` must be a valid AscPtr<Bytes> or AscPtr<BigInt>.
unsafe fn read_bytes(ptr: u32) -> Vec<u8> {
    if ptr == 0 {
        return Vec::new();
    }
    let fields = ptr as *const u32;
    // dataStart points to the actual byte data in the ArrayBuffer.
    let data_start = unsafe { fields.add(1).read_unaligned() };
    let length = unsafe { fields.add(2).read_unaligned() } as usize;

    if length == 0 {
        return Vec::new();
    }

    let slice = unsafe { core::slice::from_raw_parts(data_start as *const u8, length) };
    slice.to_vec()
}

/// Decode an AS Value (AscEnum) at `ptr` into a `StoreValue`.
///
/// Layout: kind(u32) + _pad(u32) + payload(u64)
///
/// # Safety
/// `ptr` must be a valid AscPtr<Value>.
unsafe fn read_value(ptr: u32) -> StoreValue {
    if ptr == 0 {
        return StoreValue::Null;
    }
    let fields = ptr as *const u32;
    let kind = unsafe { fields.read_unaligned() };
    // payload is at offset 8 (after kind u32 + pad u32).
    let payload_ptr = (ptr + 8) as *const u64;
    let payload = unsafe { payload_ptr.read_unaligned() };

    match kind {
        0 => {
            // String
            let s = unsafe { read_asc_string(payload as u32) };
            StoreValue::String(s)
        }
        1 => {
            // Int
            StoreValue::Int(payload as i32)
        }
        3 => {
            // Bool
            StoreValue::Bool(payload != 0)
        }
        6 => {
            // Bytes
            let b = unsafe { read_bytes(payload as u32) };
            StoreValue::Bytes(b)
        }
        7 => {
            // BigInt
            let b = unsafe { read_bytes(payload as u32) };
            StoreValue::BigInt(b)
        }
        _ => StoreValue::Null,
    }
}

/// Decode a TypedMap returned by `store.get` into a flat vec of (key, value) pairs.
///
/// # Safety
/// `map_ptr` must be a valid AscPtr<TypedMap<String, Value>> as returned by
/// graph-node's `store.get`. Returns an empty vec if `map_ptr` is 0.
pub unsafe fn read_typed_map(map_ptr: u32) -> Vec<(String, StoreValue)> {
    if map_ptr == 0 {
        return Vec::new();
    }

    // TypedMap payload: [entries: u32 AscPtr<Array<TypedMapEntry>>]
    let entries_arr_ptr = unsafe { (map_ptr as *const u32).read_unaligned() };
    if entries_arr_ptr == 0 {
        return Vec::new();
    }

    // Array<TypedMapEntry> payload (ArrayBufferView, 16 bytes):
    // [buffer: u32][buffer_data_start: u32][buffer_data_length: u32][length: u32]
    let arr_fields = entries_arr_ptr as *const u32;
    let buf_data_start = unsafe { arr_fields.add(1).read_unaligned() };
    let length = unsafe { arr_fields.add(3).read_unaligned() } as usize;

    if length == 0 {
        return Vec::new();
    }

    // ArrayBuffer data contains `length` u32 entry pointers.
    let entry_ptrs = buf_data_start as *const u32;
    let mut result = Vec::with_capacity(length);

    for i in 0..length {
        let entry_ptr = unsafe { entry_ptrs.add(i).read_unaligned() };
        if entry_ptr == 0 {
            continue;
        }

        // TypedMapEntry payload: [key: u32][value: u32]
        let entry_fields = entry_ptr as *const u32;
        let key_ptr = unsafe { entry_fields.read_unaligned() };
        let value_ptr = unsafe { entry_fields.add(1).read_unaligned() };

        let key = unsafe { read_asc_string(key_ptr) };
        let value = unsafe { read_value(value_ptr) };

        result.push((key, value));
    }

    result
}
