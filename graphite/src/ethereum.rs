//! Ethereum ABI utilities.

use alloc::vec::Vec;
use graph_as_runtime::ethereum::EthereumValue;

/// ABI-encode a single `EthereumValue`.
///
/// Encodes `value` as a one-element ABI tuple, matching the behaviour of
/// `ethabi::encode(&[token])` in graph-node (and graph-ts `ethereum.encode`).
///
/// # Encoding rules (per ABI spec)
///
/// - **Static types** (address, bool, uintN, intN, bytesN): 32-byte head, no tail.
/// - **Dynamic types** (bytes, string): 32-byte offset in head, then length + data in tail.
/// - **Arrays / tuples**: encoded recursively; arrays and tuples with dynamic
///   elements use offset-based encoding.
///
/// Returns `None` for `EthereumValue::Unknown`.
pub fn encode(value: &EthereumValue) -> Option<Vec<u8>> {
    if matches!(value, EthereumValue::Unknown(_)) {
        return None;
    }
    if is_dynamic(value) {
        // head: offset = 32 (pointing past the head)
        let mut out = Vec::new();
        out.extend_from_slice(&u256_from_usize(32)); // offset
        encode_inner(value, &mut out);
        Some(out)
    } else {
        let mut out = Vec::new();
        encode_inner(value, &mut out);
        Some(out)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Returns `true` if the value requires dynamic (offset-based) encoding.
fn is_dynamic(value: &EthereumValue) -> bool {
    match value {
        EthereumValue::Bytes(_) | EthereumValue::String(_) => true,
        EthereumValue::Array(_) => true,
        EthereumValue::FixedArray(items) => items.iter().any(is_dynamic),
        EthereumValue::Tuple(items) => items.iter().any(is_dynamic),
        _ => false,
    }
}

/// Encode `value` and append to `out`.
///
/// For static types: appends exactly 32 bytes.
/// For dynamic types: appends length (u256) + data (padded to multiple of 32).
fn encode_inner(value: &EthereumValue, out: &mut Vec<u8>) {
    match value {
        EthereumValue::Address(bytes) => {
            // 12 zero bytes + 20 address bytes
            out.extend_from_slice(&[0u8; 12]);
            out.extend_from_slice(bytes);
        }
        EthereumValue::Bool(b) => {
            out.extend_from_slice(&[0u8; 31]);
            out.push(*b as u8);
        }
        EthereumValue::Uint(le_bytes) => {
            // Little-endian → big-endian, zero-padded on the left to 32 bytes.
            let be = le_to_be_padded(le_bytes, 32, false);
            out.extend_from_slice(&be);
        }
        EthereumValue::Int(le_bytes) => {
            // Sign-extended little-endian → big-endian, 32 bytes.
            let be = le_to_be_padded(le_bytes, 32, true);
            out.extend_from_slice(&be);
        }
        EthereumValue::FixedBytes(bytes) => {
            // Right-padded (data is left-aligned in the 32-byte slot).
            let len = bytes.len().min(32);
            out.extend_from_slice(&bytes[..len]);
            for _ in len..32 {
                out.push(0);
            }
        }
        EthereumValue::Bytes(bytes) => {
            // length (u256) + data + padding
            out.extend_from_slice(&u256_from_usize(bytes.len()));
            out.extend_from_slice(bytes);
            pad_to_32(out, bytes.len());
        }
        EthereumValue::String(s) => {
            let bytes = s.as_bytes();
            out.extend_from_slice(&u256_from_usize(bytes.len()));
            out.extend_from_slice(bytes);
            pad_to_32(out, bytes.len());
        }
        EthereumValue::Array(items) => {
            // Dynamic array: length (u256) + encoded elements (with offset table for dynamic elements)
            out.extend_from_slice(&u256_from_usize(items.len()));
            encode_sequence(items, out);
        }
        EthereumValue::FixedArray(items) => {
            encode_sequence(items, out);
        }
        EthereumValue::Tuple(items) => {
            encode_sequence(items, out);
        }
        EthereumValue::Unknown(_) => {}
    }
}

/// Encode a sequence of values (elements of a fixed array or tuple) using
/// the standard ABI head/tail scheme.
fn encode_sequence(items: &[EthereumValue], out: &mut Vec<u8>) {
    // Build head + tail.
    // Head: for each item, 32 bytes — either the value (static) or an offset (dynamic).
    // Tail: dynamic items appended in order.
    let head_size = items.len() * 32;
    let mut tail: Vec<u8> = Vec::new();
    let mut head: Vec<u8> = Vec::with_capacity(head_size);

    for item in items {
        if is_dynamic(item) {
            // Write offset: head_size + current tail length
            let offset = head_size + tail.len();
            head.extend_from_slice(&u256_from_usize(offset));
            encode_inner(item, &mut tail);
        } else {
            encode_inner(item, &mut head);
        }
    }

    out.extend_from_slice(&head);
    out.extend_from_slice(&tail);
}

/// Convert `n` to a big-endian 32-byte uint256.
fn u256_from_usize(n: usize) -> [u8; 32] {
    let mut out = [0u8; 32];
    let be = (n as u64).to_be_bytes();
    out[24..].copy_from_slice(&be);
    out
}

/// Convert little-endian bytes to a big-endian 32-byte slice.
/// If `sign_extend` is true, the high bits are filled based on the sign of the input.
fn le_to_be_padded(le: &[u8], width: usize, sign_extend: bool) -> Vec<u8> {
    let is_negative = sign_extend
        && le.last().map(|&b| b & 0x80 != 0).unwrap_or(false);

    let fill = if is_negative { 0xFF } else { 0x00 };
    let mut padded_le = alloc::vec![fill; width];
    let copy_len = le.len().min(width);
    padded_le[..copy_len].copy_from_slice(&le[..copy_len]);
    padded_le.reverse(); // LE → BE
    padded_le
}

/// Pad `out` so total length of the last appended `data_len` bytes is a multiple of 32.
fn pad_to_32(out: &mut Vec<u8>, data_len: usize) {
    let remainder = data_len % 32;
    if remainder != 0 {
        let pad = 32 - remainder;
        for _ in 0..pad {
            out.push(0);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_address() {
        let addr = [0xABu8; 20];
        let enc = encode(&EthereumValue::Address(addr)).unwrap();
        assert_eq!(enc.len(), 32);
        assert_eq!(&enc[..12], &[0u8; 12]);
        assert_eq!(&enc[12..], &addr);
    }

    #[test]
    fn encode_bool_true() {
        let enc = encode(&EthereumValue::Bool(true)).unwrap();
        assert_eq!(enc.len(), 32);
        assert_eq!(enc[31], 1);
        assert_eq!(&enc[..31], &[0u8; 31]);
    }

    #[test]
    fn encode_uint256_42() {
        // 42 in little-endian
        let enc = encode(&EthereumValue::Uint(vec![42])).unwrap();
        assert_eq!(enc.len(), 32);
        assert_eq!(enc[31], 42);
        assert_eq!(&enc[..31], &[0u8; 31]);
    }

    #[test]
    fn encode_int256_negative_one() {
        // -1 in little-endian two's complement is [0xFF]
        let enc = encode(&EthereumValue::Int(vec![0xFF])).unwrap();
        assert_eq!(enc.len(), 32);
        // Should be sign-extended: all 0xFF
        assert_eq!(enc, [0xFFu8; 32].to_vec());
    }

    #[test]
    fn encode_bytes_hello() {
        let data = b"hello".to_vec();
        let enc = encode(&EthereumValue::Bytes(data)).unwrap();
        // offset(32) + length(5) + data(32 bytes padded)
        assert_eq!(enc.len(), 96);
        assert_eq!(enc[31], 32); // offset = 32
        assert_eq!(enc[63], 5);  // length = 5
        assert_eq!(&enc[64..69], b"hello");
        assert_eq!(&enc[69..96], &[0u8; 27]);
    }

    #[test]
    fn encode_string() {
        let s = "hi".to_string();
        let enc = encode(&EthereumValue::String(s)).unwrap();
        assert_eq!(enc.len(), 96);
        assert_eq!(enc[31], 32); // offset = 32
        assert_eq!(enc[63], 2);  // length = 2
        assert_eq!(&enc[64..66], b"hi");
    }

    #[test]
    fn encode_fixed_bytes4() {
        let enc = encode(&EthereumValue::FixedBytes(vec![0xDE, 0xAD, 0xBE, 0xEF])).unwrap();
        assert_eq!(enc.len(), 32);
        assert_eq!(&enc[..4], &[0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(&enc[4..], &[0u8; 28]);
    }

    #[test]
    fn encode_unknown_returns_none() {
        assert!(encode(&EthereumValue::Unknown(0)).is_none());
    }
}
