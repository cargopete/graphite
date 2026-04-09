//! AssemblyScript type constructors.
//!
//! These functions build the exact in-memory layout that graph-node's AS ABI
//! layer expects when it reads entity data out of WASM memory.
//!
//! # Memory layout reference
//!
//! **AS String** (class ID = 1, built-in):
//! ```text
//! [16-byte header] [UTF-16LE code units...]
//! ```
//! `rt_size` = byte count of the UTF-16LE payload.
//! The AscPtr<AscString> stored in parent objects points at the payload start.
//!
//! **TypedMapEntry<K,V>** (class ID = TYPED_MAP_ENTRY):
//! ```text
//! [16-byte header] [key: u32 AscPtr<String>] [value: u32 AscPtr<Value>]
//! ```
//!
//! **Array<TypedMapEntry>** (class ID = ARRAY_TYPED_MAP_ENTRY):
//! ```text
//! [16-byte header] [buffer: u32 AscPtr<ArrayBuffer>] [length: u32]
//! ```
//! The buffer is a raw ArrayBuffer holding `length` u32 pointers.
//!
//! **ArrayBuffer** (class ID = 4, built-in):
//! ```text
//! [16-byte header] [ptr_0: u32] [ptr_1: u32] ...
//! ```
//!
//! **TypedMap<K,V>** (class ID = TYPED_MAP):
//! ```text
//! [16-byte header] [entries: u32 AscPtr<Array<TypedMapEntry>>]
//! ```
//!
//! **Value** (class ID = VALUE):
//! ```text
//! [16-byte header] [kind: u32] [payload: u32 OR u64]
//! ```
//! The `kind` discriminant and payload encoding follow graph-ts's ValueKind enum.
//!
//! # ValueKind discriminants (graph-ts ValueKind enum)
//!
//! | Value | Kind          | Payload                                         |
//! |-------|---------------|-------------------------------------------------|
//! | 0     | String        | AscPtr<AscString>                               |
//! | 1     | Int           | i32 (lower 32 bits of the 8-byte payload slot)  |
//! | 2     | BigDecimal    | AscPtr<BigDecimal> (not implemented here)        |
//! | 3     | Bool          | 1 = true, 0 = false                             |
//! | 4     | Array         | AscPtr<Array<Value>>                            |
//! | 5     | Null          | 0                                               |
//! | 6     | Bytes         | AscPtr<Bytes>                                   |
//! | 7     | BigInt        | AscPtr<BigInt>                                  |

use crate::alloc_impl::alloc_as_obj;
use crate::class_ids;
use alloc::vec::Vec;

// ============================================================================
// Built-in AS class IDs
// ============================================================================

/// Built-in AS string class ID.
const CLASS_STRING: u32 = 1;
/// Built-in ArrayBuffer class ID.
const CLASS_ARRAY_BUFFER: u32 = 4;

// ============================================================================
// ValueKind discriminants
// ============================================================================

pub const VALUE_KIND_STRING: u32 = 0;
pub const VALUE_KIND_INT: u32 = 1;
pub const VALUE_KIND_BOOL: u32 = 3;
pub const VALUE_KIND_BYTES: u32 = 6;
pub const VALUE_KIND_BIG_INT: u32 = 7;

// ============================================================================
// AscString
// ============================================================================

/// Encode a Rust `&str` as an AS String object (UTF-16LE).
///
/// Returns the AscPtr — pointer past the 16-byte header.
pub fn new_asc_string(s: &str) -> u32 {
    // Encode to UTF-16LE.
    let mut utf16: Vec<u8> = Vec::with_capacity(s.len() * 2);
    for ch in s.encode_utf16() {
        let bytes = ch.to_le_bytes();
        utf16.push(bytes[0]);
        utf16.push(bytes[1]);
    }

    let payload_len = utf16.len() as u32;
    let ptr = alloc_as_obj(CLASS_STRING, payload_len);

    // Write UTF-16LE bytes into payload.
    unsafe {
        core::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr as *mut u8, utf16.len());
    }

    ptr
}

// ============================================================================
// Bytes / BigInt (AS Uint8Array / Bytes layout)
// ============================================================================

/// AS Bytes / BigInt on-heap layout:
/// ```text
/// [16-byte header]
/// [buffer: u32 AscPtr<ArrayBuffer>]
/// [dataStart: u32]   -- byte offset into buffer (always 0 for us)
/// [length: u32]      -- element count
/// ```
///
/// The ArrayBuffer itself:
/// ```text
/// [16-byte header]
/// [raw bytes...]
/// ```

/// ArrayBuffer class ID for a Uint8Array-backed value (Bytes/BigInt).
const CLASS_BYTES: u32 = 26; // Bytes in graph-ts compiled output
const CLASS_BIG_INT: u32 = 21; // BigInt in graph-ts compiled output

/// Build an AS `Bytes` object from raw bytes.
pub fn new_asc_bytes(data: &[u8]) -> u32 {
    new_uint8array_like(CLASS_BYTES, data)
}

/// Build an AS `BigInt` object from raw bytes.
/// graph-ts BigInt stores bytes in little-endian two's-complement.
pub fn new_asc_big_int(data: &[u8]) -> u32 {
    new_uint8array_like(CLASS_BIG_INT, data)
}

fn new_uint8array_like(class_id: u32, data: &[u8]) -> u32 {
    // 1. Allocate ArrayBuffer for the raw bytes.
    let buf_ptr = alloc_as_obj(CLASS_ARRAY_BUFFER, data.len() as u32);
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr(), buf_ptr as *mut u8, data.len());
    }

    // 2. Allocate the Uint8Array wrapper — 3 u32 fields = 12 bytes.
    let wrapper_ptr = alloc_as_obj(class_id, 12);
    let fields = wrapper_ptr as *mut u32;
    unsafe {
        fields.write(buf_ptr);           // buffer
        fields.add(1).write(buf_ptr);    // dataStart (same as buffer ptr for us)
        fields.add(2).write(data.len() as u32); // length
    }

    wrapper_ptr
}

// ============================================================================
// Value
// ============================================================================

/// AS Value (AscEnum<StoreValueKind>) layout:
///
/// ```text
/// [kind:    u32]  -- discriminant (StoreValueKind)
/// [_pad:    u32]  -- explicit 4-byte padding (graph-node AscEnum has this field)
/// [payload: u64]  -- 8-byte EnumPayload (pointer or scalar)
/// ```
/// Total payload = 16 bytes.
///
/// The explicit padding between `kind` and `payload` is required by
/// graph-node's `AscEnum<D>` struct definition (`_padding: u32` field).
/// Previously this was 12 bytes which caused "Size does not match" errors.

/// Build a `Value` of kind String.
pub fn new_value_string(str_ptr: u32) -> u32 {
    new_value(VALUE_KIND_STRING, str_ptr as u64)
}

/// Build a `Value` of kind Bytes.
pub fn new_value_bytes(bytes_ptr: u32) -> u32 {
    new_value(VALUE_KIND_BYTES, bytes_ptr as u64)
}

/// Build a `Value` of kind BigInt.
pub fn new_value_big_int(big_int_ptr: u32) -> u32 {
    new_value(VALUE_KIND_BIG_INT, big_int_ptr as u64)
}

/// Build a `Value` of kind Int (i32).
pub fn new_value_int(n: i32) -> u32 {
    new_value(VALUE_KIND_INT, n as u64)
}

/// Build a `Value` of kind Bool.
pub fn new_value_bool(b: bool) -> u32 {
    new_value(VALUE_KIND_BOOL, b as u64)
}

fn new_value(kind: u32, payload: u64) -> u32 {
    // Layout: kind(u32) + padding(u32) + payload(u64) = 16 bytes.
    let ptr = alloc_as_obj(class_ids::VALUE, 16);
    let fields = ptr as *mut u32;
    unsafe {
        fields.write(kind);            // kind
        fields.add(1).write(0);        // _padding (explicit, matches AscEnum._padding)
        // payload as two u32 words, little-endian
        let payload_ptr = fields.add(2) as *mut u64;
        payload_ptr.write_unaligned(payload);
    }
    ptr
}

// ============================================================================
// TypedMapEntry
// ============================================================================

/// Build a `TypedMapEntry<string, Value>`.
///
/// Layout: [key: u32 AscPtr<String>] [value: u32 AscPtr<Value>]
/// Total payload = 8 bytes.
pub fn new_typed_map_entry(key_ptr: u32, value_ptr: u32) -> u32 {
    let ptr = alloc_as_obj(class_ids::TYPED_MAP_ENTRY, 8);
    let fields = ptr as *mut u32;
    unsafe {
        fields.write(key_ptr);
        fields.add(1).write(value_ptr);
    }
    ptr
}

// ============================================================================
// Array<TypedMapEntry> and TypedMap
// ============================================================================

/// Build an `Array<AscPtr<TypedMapEntry>>` from a slice of entry pointers.
///
/// For apiVersion >= 0.0.5, the AS `Array<T>` (ArrayBufferView) layout is:
/// ```text
/// [buffer:            u32]  -- AscPtr<ArrayBuffer> (the backing buffer)
/// [buffer_data_start: u32]  -- absolute pointer to data start within buffer
/// [buffer_data_length:u32]  -- byte length of the active data region
/// [length:            i32]  -- element count (mutable user-visible length)
/// ```
/// Total Array payload = 16 bytes.
///
/// The ArrayBuffer payload = length * sizeof(element) bytes.
pub fn new_typed_map_entry_array(entries: &[u32]) -> u32 {
    // 1. Build the ArrayBuffer (raw u32 pointers, each 4 bytes).
    let buf_bytes = (entries.len() * 4) as u32;
    let buf_ptr = alloc_as_obj(CLASS_ARRAY_BUFFER, buf_bytes);
    let buf_data = buf_ptr as *mut u32;
    for (i, &entry_ptr) in entries.iter().enumerate() {
        unsafe {
            buf_data.add(i).write(entry_ptr);
        }
    }

    // 2. Build the Array object with the v0.0.5 ArrayBufferView layout (16 bytes).
    //    buffer_data_start = buf_ptr (data starts at the very beginning of the buffer).
    //    buffer_data_length = buf_bytes (all bytes of the buffer are in use).
    //    length = number of elements.
    let arr_ptr = alloc_as_obj(class_ids::ARRAY_TYPED_MAP_ENTRY, 16);
    let arr_fields = arr_ptr as *mut u32;
    unsafe {
        arr_fields.write(buf_ptr);                      // buffer
        arr_fields.add(1).write(buf_ptr);               // buffer_data_start (= buffer start)
        arr_fields.add(2).write(buf_bytes);             // buffer_data_length (bytes)
        arr_fields.add(3).write(entries.len() as u32);  // length (element count)
    }

    arr_ptr
}

/// Build a `TypedMap<string, Value>` from a list of (key, value_ptr) pairs.
///
/// AS TypedMap layout:
/// ```text
/// [entries: u32 AscPtr<Array<TypedMapEntry>>]
/// ```
/// Total payload = 4 bytes.
pub fn new_typed_map(fields: &[(&str, u32)]) -> u32 {
    // Build each entry.
    let mut entry_ptrs: Vec<u32> = Vec::with_capacity(fields.len());
    for &(key, value_ptr) in fields {
        let key_ptr = new_asc_string(key);
        let entry_ptr = new_typed_map_entry(key_ptr, value_ptr);
        entry_ptrs.push(entry_ptr);
    }

    // Build the entries array.
    let entries_ptr = new_typed_map_entry_array(&entry_ptrs);

    // Build the TypedMap itself.
    let map_ptr = alloc_as_obj(class_ids::TYPED_MAP, 4);
    let map_fields = map_ptr as *mut u32;
    unsafe {
        map_fields.write(entries_ptr);
    }

    map_ptr
}
