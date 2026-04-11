//! AS JSONValue decoder — reads the JSONValue AscEnum that graph-node writes
//! into WASM linear memory after a `json.fromBytes` host call.
//!
//! # AS JSONValueKind discriminants
//!
//! ```text
//! NULL   = 0
//! BOOL   = 1
//! NUMBER = 2   data → AscPtr<AscString>  (the raw number text)
//! STRING = 3   data → AscPtr<AscString>
//! ARRAY  = 4   data → AscPtr<Array<JSONValue>>
//! OBJECT = 5   data → AscPtr<TypedMap<AscString, JSONValue>>
//! ```
//!
//! JSONValue AscEnum layout (16 bytes):
//!   offset 0   kind   u32
//!   offset 4   _pad   u32
//!   offset 8   data   u64   (lower 32 bits used as AscPtr)
//!
//! TypedMap<K,V> payload:
//!   offset 0   entries   AscPtr<Array<TypedMapEntry<K,V>>>
//!
//! TypedMapEntry<K,V> payload:
//!   offset 0   key   AscPtr<K>
//!   offset 4   value AscPtr<V>
//!
//! Array<T> payload:
//!   offset 0   buffer         AscPtr<ArrayBuffer>
//!   offset 4   buffer_data_start  AscPtr   (absolute ptr to first element)
//!   offset 8   buffer_data_len    u32      (byte length)
//!   offset 12  length         u32          (element count)

use alloc::{string::String, vec::Vec};

// ============================================================================
// Public type
// ============================================================================

/// A decoded JSON value returned by `json.fromBytes`.
#[derive(Clone, Debug)]
pub enum JsonValue {
    Null,
    Bool(bool),
    /// JSON number, stored as its raw text representation.
    /// Use `.as_str()` for the text, or convert via `str::parse::<f64>()`.
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    /// Key-value pairs in declaration order.
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Look up a key in a JSON object. Returns `None` for non-objects or missing keys.
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if let JsonValue::Object(entries) = self {
            entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
        } else {
            None
        }
    }

    /// Get element at index in a JSON array. Returns `None` for non-arrays or out-of-bounds.
    pub fn get_index(&self, i: usize) -> Option<&JsonValue> {
        if let JsonValue::Array(arr) = self {
            arr.get(i)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let JsonValue::String(s) = self { Some(s.as_str()) } else { None }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let JsonValue::Bool(b) = self { Some(*b) } else { None }
    }

    /// Return the raw text of a JSON number (e.g. `"3.14"`, `"42"`).
    pub fn as_number_str(&self) -> Option<&str> {
        if let JsonValue::Number(s) = self { Some(s.as_str()) } else { None }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        if let JsonValue::Array(arr) = self { Some(arr) } else { None }
    }

    pub fn as_object(&self) -> Option<&Vec<(String, JsonValue)>> {
        if let JsonValue::Object(entries) = self { Some(entries) } else { None }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }
}

// ============================================================================
// Low-level memory helpers — only needed when running inside WASM
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[inline(always)]
unsafe fn read_u32(ptr: u32) -> u32 {
    unsafe { (ptr as *const u32).read_unaligned() }
}

#[cfg(target_arch = "wasm32")]
#[inline(always)]
unsafe fn read_u64(ptr: u32) -> u64 {
    unsafe { (ptr as *const u64).read_unaligned() }
}

/// Read an AS String (UTF-16LE). `rt_size` is at ptr - 4 (last 4 bytes of the header).
#[cfg(target_arch = "wasm32")]
unsafe fn read_string(ptr: u32) -> String {
    if ptr == 0 {
        return String::new();
    }
    let byte_len = unsafe { read_u32(ptr - 4) } as usize;
    if byte_len == 0 {
        return String::new();
    }
    let char_count = byte_len / 2;
    let raw = ptr as *const u16;
    let slice = unsafe { core::slice::from_raw_parts(raw, char_count) };
    char::decode_utf16(slice.iter().copied())
        .map(|r| r.unwrap_or('\u{FFFD}'))
        .collect()
}

/// Read a JSONValue AscEnum at `ptr` and recursively decode it.
#[cfg(target_arch = "wasm32")]
pub unsafe fn read_json_value(ptr: u32) -> JsonValue {
    if ptr == 0 {
        return JsonValue::Null;
    }
    unsafe {
        let kind = read_u32(ptr);
        let data = read_u64(ptr + 8);
        let data_ptr = data as u32;

        match kind {
            0 => JsonValue::Null,
            1 => JsonValue::Bool(data != 0),
            2 => {
                let s = read_string(data_ptr);
                JsonValue::Number(s)
            }
            3 => {
                let s = read_string(data_ptr);
                JsonValue::String(s)
            }
            4 => {
                let items = read_json_array(data_ptr);
                JsonValue::Array(items)
            }
            5 => {
                let entries = read_json_object(data_ptr);
                JsonValue::Object(entries)
            }
            _ => JsonValue::Null,
        }
    }
}

/// Read an `Array<JSONValue>` — same layout as `Array<EthereumEventParam>`.
#[cfg(target_arch = "wasm32")]
unsafe fn read_json_array(arr_ptr: u32) -> Vec<JsonValue> {
    if arr_ptr == 0 {
        return Vec::new();
    }
    unsafe {
        let data_start = read_u32(arr_ptr + 4);
        let length = read_u32(arr_ptr + 12);
        let mut out = Vec::with_capacity(length as usize);
        for i in 0..length {
            let elem_ptr = read_u32(data_start + i * 4);
            out.push(if elem_ptr != 0 {
                read_json_value(elem_ptr)
            } else {
                JsonValue::Null
            });
        }
        out
    }
}

/// Read a `TypedMap<AscString, JSONValue>` into a vec of `(String, JsonValue)` pairs.
#[cfg(target_arch = "wasm32")]
unsafe fn read_json_object(map_ptr: u32) -> Vec<(String, JsonValue)> {
    if map_ptr == 0 {
        return Vec::new();
    }
    unsafe {
        // TypedMap payload: [entries_arr_ptr: u32]
        let entries_arr_ptr = read_u32(map_ptr);
        if entries_arr_ptr == 0 {
            return Vec::new();
        }
        // Array<TypedMapEntry> layout: [buf: u32][data_start: u32][data_len: u32][length: u32]
        let data_start = read_u32(entries_arr_ptr + 4);
        let length = read_u32(entries_arr_ptr + 12);
        let mut out = Vec::with_capacity(length as usize);
        for i in 0..length {
            let entry_ptr = read_u32(data_start + i * 4);
            if entry_ptr == 0 {
                continue;
            }
            // TypedMapEntry: [key: u32][value: u32]
            let key_ptr = read_u32(entry_ptr);
            let val_ptr = read_u32(entry_ptr + 4);
            let key = read_string(key_ptr);
            let value = read_json_value(val_ptr);
            out.push((key, value));
        }
        out
    }
}
