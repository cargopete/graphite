//! Event log decoding utilities.
//!
//! Provides traits and helpers for decoding Ethereum event logs
//! into typed Rust structs.

use crate::primitives::{Address, BigInt, Bytes, B256};
use alloc::string::String;
use alloc::vec::Vec;

/// Trait for types that can be decoded from event log data.
///
/// Implemented by generated event structs.
pub trait EventDecode: Sized {
    /// The event selector (keccak256 of the event signature).
    const SELECTOR: [u8; 32];

    /// Decode from log topics and data.
    ///
    /// - `topics`: The log topics (topic[0] is the selector for non-anonymous events)
    /// - `data`: The ABI-encoded non-indexed parameters
    fn decode(topics: &[B256], data: &[u8]) -> Result<Self, DecodeError>;
}

/// Errors that can occur during event decoding.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("selector mismatch: expected {expected:?}, got {got:?}")]
    SelectorMismatch { expected: [u8; 32], got: [u8; 32] },

    #[error("not enough topics: expected {expected}, got {got}")]
    NotEnoughTopics { expected: usize, got: usize },

    #[error("data too short: expected at least {expected} bytes, got {got}")]
    DataTooShort { expected: usize, got: usize },

    #[error("invalid data: {0}")]
    InvalidData(String),
}

/// Raw log data from an Ethereum event.
#[derive(Debug, Clone)]
pub struct RawLog {
    /// Contract address that emitted the event
    pub address: Address,
    /// Log topics (first is usually the event selector)
    pub topics: Vec<B256>,
    /// ABI-encoded event data
    pub data: Bytes,
    /// Transaction hash
    pub tx_hash: B256,
    /// Log index within the block
    pub log_index: u64,
    /// Block number
    pub block_number: u64,
    /// Block timestamp
    pub block_timestamp: u64,
}

// ============ Primitive Decoding Helpers ============

/// Decode an address from a 32-byte topic (right-aligned, left-padded with zeros).
pub fn decode_address_from_topic(topic: &B256) -> Address {
    Address::from_slice(&topic.as_slice()[12..])
}

/// Decode a uint256 from a 32-byte topic.
pub fn decode_uint256_from_topic(topic: &B256) -> BigInt {
    BigInt::from_unsigned_bytes_be(topic.as_slice())
}

/// Decode a bytes32 from a topic (identity).
pub fn decode_bytes32_from_topic(topic: &B256) -> B256 {
    *topic
}

/// Decode a bool from a topic.
pub fn decode_bool_from_topic(topic: &B256) -> bool {
    topic.as_slice()[31] != 0
}

// ============ ABI Data Decoding ============

/// Decode a uint256 from ABI-encoded data at the given offset.
pub fn decode_uint256(data: &[u8], offset: usize) -> Result<BigInt, DecodeError> {
    if data.len() < offset + 32 {
        return Err(DecodeError::DataTooShort {
            expected: offset + 32,
            got: data.len(),
        });
    }
    Ok(BigInt::from_unsigned_bytes_be(&data[offset..offset + 32]))
}

/// Decode an address from ABI-encoded data at the given offset.
pub fn decode_address(data: &[u8], offset: usize) -> Result<Address, DecodeError> {
    if data.len() < offset + 32 {
        return Err(DecodeError::DataTooShort {
            expected: offset + 32,
            got: data.len(),
        });
    }
    Ok(Address::from_slice(&data[offset + 12..offset + 32]))
}

/// Decode a bool from ABI-encoded data at the given offset.
pub fn decode_bool(data: &[u8], offset: usize) -> Result<bool, DecodeError> {
    if data.len() < offset + 32 {
        return Err(DecodeError::DataTooShort {
            expected: offset + 32,
            got: data.len(),
        });
    }
    Ok(data[offset + 31] != 0)
}

/// Decode bytes32 from ABI-encoded data at the given offset.
pub fn decode_bytes32(data: &[u8], offset: usize) -> Result<B256, DecodeError> {
    if data.len() < offset + 32 {
        return Err(DecodeError::DataTooShort {
            expected: offset + 32,
            got: data.len(),
        });
    }
    Ok(B256::from_slice(&data[offset..offset + 32]))
}

/// Decode a dynamic bytes value from ABI-encoded data.
///
/// The offset points to the location in data where the bytes pointer is stored.
pub fn decode_bytes(data: &[u8], offset: usize) -> Result<Bytes, DecodeError> {
    // First, read the offset to the actual data
    let data_offset = decode_uint256(data, offset)?;
    let data_offset: usize = data_offset
        .to_u64()
        .ok_or_else(|| DecodeError::InvalidData("bytes offset too large".into()))?
        as usize;

    // Then read the length
    let length = decode_uint256(data, data_offset)?;
    let length: usize = length
        .to_u64()
        .ok_or_else(|| DecodeError::InvalidData("bytes length too large".into()))?
        as usize;

    // Finally read the data
    let start = data_offset + 32;
    let end = start + length;
    if data.len() < end {
        return Err(DecodeError::DataTooShort {
            expected: end,
            got: data.len(),
        });
    }

    Ok(Bytes::from_slice(&data[start..end]))
}

/// Decode a dynamic string from ABI-encoded data.
pub fn decode_string(data: &[u8], offset: usize) -> Result<String, DecodeError> {
    let bytes = decode_bytes(data, offset)?;
    String::from_utf8(bytes.to_vec()).map_err(|e| DecodeError::InvalidData(e.to_string()))
}

// ============ TLV Decoding for WASM ============

/// Trait for deserializing types from graph-node's TLV WASM format.
///
/// This is used to deserialize events passed from graph-node to handlers.
/// Generated event structs implement this trait via codegen.
pub trait FromWasmBytes: Sized {
    /// Deserialize from TLV-encoded bytes.
    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;
}

/// Value tag constants matching graph-node's rust_abi/types.rs
pub mod value_tag {
    pub const NULL: u8 = 0x00;
    pub const STRING: u8 = 0x01;
    pub const INT: u8 = 0x02;
    pub const INT8: u8 = 0x03;
    pub const BIGINT: u8 = 0x04;
    pub const BIGDECIMAL: u8 = 0x05;
    pub const BOOL: u8 = 0x06;
    pub const BYTES: u8 = 0x07;
    pub const ADDRESS: u8 = 0x08;
    pub const ARRAY: u8 = 0x09;
}

/// TLV reader helper for deserializing graph-node's format.
pub struct TlvReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> TlvReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn read_u32(&mut self) -> Result<u32, DecodeError> {
        if self.remaining() < 4 {
            return Err(DecodeError::DataTooShort {
                expected: self.pos + 4,
                got: self.data.len(),
            });
        }
        let val = u32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(val)
    }

    pub fn read_i32(&mut self) -> Result<i32, DecodeError> {
        if self.remaining() < 4 {
            return Err(DecodeError::DataTooShort {
                expected: self.pos + 4,
                got: self.data.len(),
            });
        }
        let val = i32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(val)
    }

    pub fn read_i64(&mut self) -> Result<i64, DecodeError> {
        if self.remaining() < 8 {
            return Err(DecodeError::DataTooShort {
                expected: self.pos + 8,
                got: self.data.len(),
            });
        }
        let val = i64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(val)
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        if self.remaining() < 1 {
            return Err(DecodeError::DataTooShort {
                expected: self.pos + 1,
                got: self.data.len(),
            });
        }
        let val = self.data[self.pos];
        self.pos += 1;
        Ok(val)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        if self.remaining() < len {
            return Err(DecodeError::DataTooShort {
                expected: self.pos + len,
                got: self.data.len(),
            });
        }
        let val = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(val)
    }

    pub fn read_string(&mut self) -> Result<String, DecodeError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| DecodeError::InvalidData(e.to_string()))
    }

    pub fn read_bigint(&mut self) -> Result<BigInt, DecodeError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(BigInt::from_signed_bytes_be(bytes))
    }

    pub fn read_address(&mut self) -> Result<Address, DecodeError> {
        let bytes = self.read_bytes(20)?;
        Ok(Address::from_slice(bytes))
    }

    pub fn read_bytes_value(&mut self) -> Result<Bytes, DecodeError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(Bytes::from_slice(bytes))
    }

    pub fn read_b256(&mut self) -> Result<B256, DecodeError> {
        let bytes = self.read_bytes(32)?;
        Ok(B256::from_slice(bytes))
    }

    /// Skip a tagged value (reads tag first, then skips value data).
    pub fn skip_value(&mut self) -> Result<(), DecodeError> {
        let tag = self.read_u8()?;
        self.skip_value_data(tag)
    }

    /// Skip value data when tag has already been read.
    pub fn skip_value_data(&mut self, tag: u8) -> Result<(), DecodeError> {
        match tag {
            value_tag::NULL => Ok(()),
            value_tag::STRING | value_tag::BYTES | value_tag::BIGINT => {
                let len = self.read_u32()? as usize;
                self.read_bytes(len)?;
                Ok(())
            }
            value_tag::INT => {
                self.read_bytes(4)?;
                Ok(())
            }
            value_tag::INT8 => {
                self.read_bytes(8)?;
                Ok(())
            }
            value_tag::BIGDECIMAL => {
                // BigDecimal is scale:i64 + BigInt
                self.read_bytes(8)?;
                let len = self.read_u32()? as usize;
                self.read_bytes(len)?;
                Ok(())
            }
            value_tag::BOOL => {
                self.read_bytes(1)?;
                Ok(())
            }
            value_tag::ADDRESS => {
                self.read_bytes(20)?;
                Ok(())
            }
            value_tag::ARRAY => {
                let count = self.read_u32()?;
                for _ in 0..count {
                    self.skip_value()?; // Array elements have their own tags
                }
                Ok(())
            }
            _ => Err(DecodeError::InvalidData(format!("unknown tag: 0x{:02x}", tag))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_address_from_topic_works() {
        // Topic with address 0xdead...beef right-aligned
        let mut topic_bytes = [0u8; 32];
        topic_bytes[12..].copy_from_slice(&[
            0xde, 0xad, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xbe, 0xef,
        ]);
        let topic = B256::from(topic_bytes);

        let addr = decode_address_from_topic(&topic);
        assert_eq!(addr.as_slice()[0], 0xde);
        assert_eq!(addr.as_slice()[1], 0xad);
        assert_eq!(addr.as_slice()[18], 0xbe);
        assert_eq!(addr.as_slice()[19], 0xef);
    }

    #[test]
    fn decode_uint256_works() {
        // Value: 1000 in big-endian
        let mut data = [0u8; 32];
        data[31] = 0xe8; // 1000 = 0x3e8
        data[30] = 0x03;

        let value = decode_uint256(&data, 0).unwrap();
        assert_eq!(value, BigInt::from(1000u64));
    }

    #[test]
    fn decode_address_from_data_works() {
        // Address right-aligned in 32 bytes
        let mut data = [0u8; 32];
        data[12..].copy_from_slice(&[
            0xca, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xba, 0xbe,
        ]);

        let addr = decode_address(&data, 0).unwrap();
        assert_eq!(addr.as_slice()[0], 0xca);
        assert_eq!(addr.as_slice()[1], 0xfe);
    }
}
