//! TLV serialization/deserialization for entity values.
//!
//! Pure encode/decode logic with no FFI dependencies — compiled for both
//! wasm32 targets and host tests.

use crate::primitives::{Address, Bytes};
use crate::store::{Entity, Value};
use alloc::vec::Vec;

/// Serialize an Entity to TLV bytes.
///
/// Format: [field_count: u32] [key_len: u32, key: bytes, Value...]*
pub fn serialize_entity(entity: &Entity) -> Vec<u8> {
    let mut buf = Vec::new();
    let count = entity.len() as u32;
    buf.extend_from_slice(&count.to_le_bytes());

    for (key, value) in entity.iter() {
        let key_bytes = key.as_bytes();
        buf.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(key_bytes);
        serialize_value(&mut buf, value);
    }

    buf
}

/// Serialize a single tagged Value into `buf`.
pub fn serialize_value(buf: &mut Vec<u8>, value: &Value) {
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
        Value::BigDecimal(n) => {
            buf.push(0x05);
            let s = n.to_string();
            let bytes = s.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(bytes);
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

/// Deserialize an Entity from TLV bytes.
pub fn deserialize_entity(bytes: &[u8]) -> Option<Entity> {
    if bytes.len() < 4 {
        return None;
    }

    let mut entity = Entity::new();
    let mut pos = 0;

    let count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
    pos += 4;

    for _ in 0..count {
        let key_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;
        let key = core::str::from_utf8(&bytes[pos..pos + key_len]).ok()?;
        pos += key_len;

        let (value, consumed) = deserialize_value(&bytes[pos..])?;
        pos += consumed;

        entity.set(key, value);
    }

    Some(entity)
}

/// Deserialize a single tagged Value from `bytes`.
///
/// Returns `(value, bytes_consumed)` on success, `None` on any parse error.
pub fn deserialize_value(bytes: &[u8]) -> Option<(Value, usize)> {
    use crate::primitives::BigInt;

    if bytes.is_empty() {
        return None;
    }

    let tag = bytes[0];
    let mut pos = 1;

    let value = match tag {
        0x00 => Value::Null,
        0x01 => {
            let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            let s = core::str::from_utf8(&bytes[pos..pos + len]).ok()?;
            pos += len;
            Value::String(s.into())
        }
        0x02 => {
            let n = i32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?);
            pos += 4;
            Value::Int(n)
        }
        0x03 => {
            let n = i64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
            pos += 8;
            Value::Int8(n)
        }
        0x04 => {
            let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            let n = BigInt::from_signed_bytes_le(&bytes[pos..pos + len]);
            pos += len;
            Value::BigInt(n)
        }
        0x05 => {
            // BigDecimal: len:u32 + UTF-8 string bytes
            let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            let s = core::str::from_utf8(&bytes[pos..pos + len]).ok()?;
            pos += len;
            let n = crate::primitives::BigDecimal::from_str(s).ok()?;
            Value::BigDecimal(n)
        }
        0x06 => {
            let b = bytes[pos] != 0;
            pos += 1;
            Value::Bool(b)
        }
        0x07 => {
            let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            let b = Bytes::from_slice(&bytes[pos..pos + len]);
            pos += len;
            Value::Bytes(b)
        }
        0x08 => {
            let addr = Address::from_slice(&bytes[pos..pos + 20]);
            pos += 20;
            Value::Address(addr)
        }
        0x09 => {
            let count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            let mut arr = Vec::with_capacity(count);
            for _ in 0..count {
                let (v, consumed) = deserialize_value(&bytes[pos..])?;
                pos += consumed;
                arr.push(v);
            }
            Value::Array(arr)
        }
        _ => return None,
    };

    Some((value, pos))
}
