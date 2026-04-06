//! ABI cross-validation test vectors.
//!
//! These tests encode known binary payloads by hand — matching graph-node's
//! ToRustWasm/FromRustWasm implementations — and assert that the SDK codec
//! decodes them to the expected Rust values, and that re-encoding produces
//! identical bytes.
//!
//! The vectors are the source of truth for cross-validation; the same raw
//! bytes are tested in graph-node's entity.rs unit tests.

#[cfg(test)]
mod tests {
    use crate::primitives::{BigDecimal, BigInt, Bytes};
    use crate::store::{Entity, Value};
    use crate::wasm::codec::{deserialize_entity, deserialize_value, serialize_entity, serialize_value};
    use alloc::vec;
    use alloc::vec::Vec;

    fn le32(n: u32) -> [u8; 4] {
        n.to_le_bytes()
    }

    fn le64(n: u64) -> [u8; 8] {
        n.to_le_bytes()
    }

    // -------------------------------------------------------------------------
    // Value::Null  (tag 0x00, no body)
    // -------------------------------------------------------------------------
    #[test]
    fn null_decode() {
        let bytes = [0x00u8];
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Null);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn null_encode() {
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Null);
        assert_eq!(buf, [0x00u8]);
    }

    #[test]
    fn null_roundtrip() {
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Null);
        let (val, _) = deserialize_value(&buf).unwrap();
        assert_eq!(val, Value::Null);
    }

    // -------------------------------------------------------------------------
    // Value::String  (tag 0x01, len:u32 LE, utf-8 bytes)
    // -------------------------------------------------------------------------
    #[test]
    fn string_decode() {
        // "hi" — 2 bytes
        let mut bytes = vec![0x01u8];
        bytes.extend_from_slice(&le32(2));
        bytes.extend_from_slice(b"hi");

        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::String("hi".into()));
        assert_eq!(consumed, 1 + 4 + 2);
    }

    #[test]
    fn string_encode() {
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::String("hi".into()));
        let mut expected = vec![0x01u8];
        expected.extend_from_slice(&le32(2));
        expected.extend_from_slice(b"hi");
        assert_eq!(buf, expected);
    }

    #[test]
    fn string_roundtrip_unicode() {
        let s = "héllo wörld";
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::String(s.into()));
        let (val, _) = deserialize_value(&buf).unwrap();
        assert_eq!(val, Value::String(s.into()));
    }

    // -------------------------------------------------------------------------
    // Value::Int  (tag 0x02, i32 LE 4 bytes)
    // -------------------------------------------------------------------------
    #[test]
    fn int_decode_positive() {
        let mut bytes = vec![0x02u8];
        bytes.extend_from_slice(&42i32.to_le_bytes());
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Int(42));
        assert_eq!(consumed, 5);
    }

    #[test]
    fn int_decode_negative() {
        let mut bytes = vec![0x02u8];
        bytes.extend_from_slice(&(-1i32).to_le_bytes());
        let (val, _) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Int(-1));
    }

    #[test]
    fn int_encode() {
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Int(42));
        let mut expected = vec![0x02u8];
        expected.extend_from_slice(&42i32.to_le_bytes());
        assert_eq!(buf, expected);
    }

    // -------------------------------------------------------------------------
    // Value::Int8  (tag 0x03, i64 LE 8 bytes)
    // -------------------------------------------------------------------------
    #[test]
    fn int8_decode() {
        let mut bytes = vec![0x03u8];
        bytes.extend_from_slice(&le64(9_000_000_000_000_000_000u64));
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Int8(9_000_000_000_000_000_000i64));
        assert_eq!(consumed, 9);
    }

    #[test]
    fn int8_decode_negative() {
        let n: i64 = -42;
        let mut bytes = vec![0x03u8];
        bytes.extend_from_slice(&n.to_le_bytes());
        let (val, _) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Int8(-42));
    }

    #[test]
    fn int8_encode() {
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Int8(-42));
        let mut expected = vec![0x03u8];
        expected.extend_from_slice(&(-42i64).to_le_bytes());
        assert_eq!(buf, expected);
    }

    // -------------------------------------------------------------------------
    // Value::BigInt  (tag 0x04, len:u32 LE, signed-LE bytes)
    // -------------------------------------------------------------------------

    #[test]
    fn bigint_decode_zero() {
        // Zero BigInt: len=0, no bytes
        let mut bytes = vec![0x04u8];
        bytes.extend_from_slice(&le32(0));
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::BigInt(BigInt::zero()));
        assert_eq!(consumed, 5);
    }

    #[test]
    fn bigint_decode_positive() {
        // 1000 in signed-LE: 0xe8 0x03
        let mut bytes = vec![0x04u8];
        bytes.extend_from_slice(&le32(2));
        bytes.push(0xe8);
        bytes.push(0x03);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::BigInt(BigInt::from(1000u64)));
        assert_eq!(consumed, 7);
    }

    #[test]
    fn bigint_decode_negative() {
        // -1 in signed-LE: single byte 0xff
        let mut bytes = vec![0x04u8];
        bytes.extend_from_slice(&le32(1));
        bytes.push(0xff);
        let (val, _) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::BigInt(BigInt::from(-1i64)));
    }

    #[test]
    fn bigint_encode_uses_le() {
        let n = BigInt::from(1000u64);
        let le_bytes = n.to_signed_bytes_le();
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::BigInt(n));
        // tag
        assert_eq!(buf[0], 0x04);
        // length
        let len = u32::from_le_bytes(buf[1..5].try_into().unwrap()) as usize;
        assert_eq!(len, le_bytes.len());
        // body matches LE bytes
        assert_eq!(&buf[5..5 + len], le_bytes.as_slice());
    }

    #[test]
    fn bigint_roundtrip_large() {
        let n = BigInt::from(u64::MAX);
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::BigInt(n.clone()));
        let (val, _) = deserialize_value(&buf).unwrap();
        assert_eq!(val, Value::BigInt(n));
    }

    #[test]
    fn bigint_roundtrip_negative_large() {
        let n = BigInt::from(i64::MIN);
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::BigInt(n.clone()));
        let (val, _) = deserialize_value(&buf).unwrap();
        assert_eq!(val, Value::BigInt(n));
    }

    // -------------------------------------------------------------------------
    // Value::BigDecimal  (tag 0x05, len:u32 LE, UTF-8 string)
    // -------------------------------------------------------------------------

    #[test]
    fn bigdecimal_decode() {
        let s = b"3.14";
        let mut bytes = vec![0x05u8];
        bytes.extend_from_slice(&le32(s.len() as u32));
        bytes.extend_from_slice(s);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, 1 + 4 + s.len());
        // Should parse without error
        assert!(matches!(val, Value::BigDecimal(_)));
    }

    #[test]
    fn bigdecimal_encode_is_string() {
        let d = BigDecimal::from_str("3.14").unwrap();
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::BigDecimal(d));
        // tag
        assert_eq!(buf[0], 0x05);
        // len
        let len = u32::from_le_bytes(buf[1..5].try_into().unwrap()) as usize;
        // body should be valid UTF-8
        let s = core::str::from_utf8(&buf[5..5 + len]).unwrap();
        // must contain a dot — it's a decimal string
        assert!(s.contains('.') || s.chars().all(|c| c.is_ascii_digit() || c == '-'),
            "BigDecimal should serialize as decimal string, got: {}", s);
    }

    #[test]
    fn bigdecimal_roundtrip() {
        let d = BigDecimal::from_str("123.456").unwrap();
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::BigDecimal(d.clone()));
        let (val, _) = deserialize_value(&buf).unwrap();
        // Re-encode the decoded value and compare bytes
        let mut buf2 = Vec::new();
        serialize_value(&mut buf2, &val);
        assert_eq!(buf, buf2);
    }

    // -------------------------------------------------------------------------
    // Value::Bool  (tag 0x06, 1 byte: 0x00/0x01)
    // -------------------------------------------------------------------------

    #[test]
    fn bool_decode_true() {
        let bytes = [0x06u8, 0x01];
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Bool(true));
        assert_eq!(consumed, 2);
    }

    #[test]
    fn bool_decode_false() {
        let bytes = [0x06u8, 0x00];
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Bool(false));
        assert_eq!(consumed, 2);
    }

    #[test]
    fn bool_encode() {
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Bool(true));
        assert_eq!(buf, [0x06u8, 0x01]);

        buf.clear();
        serialize_value(&mut buf, &Value::Bool(false));
        assert_eq!(buf, [0x06u8, 0x00]);
    }

    // -------------------------------------------------------------------------
    // Value::Bytes  (tag 0x07, len:u32 LE, raw bytes)
    // -------------------------------------------------------------------------

    #[test]
    fn bytes_decode() {
        let payload = [0xde, 0xad, 0xbe, 0xef];
        let mut bytes = vec![0x07u8];
        bytes.extend_from_slice(&le32(4));
        bytes.extend_from_slice(&payload);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Bytes(Bytes::from_slice(&payload)));
        assert_eq!(consumed, 9);
    }

    #[test]
    fn bytes_encode() {
        let payload = [0xca, 0xfe];
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Bytes(Bytes::from_slice(&payload)));
        let mut expected = vec![0x07u8];
        expected.extend_from_slice(&le32(2));
        expected.extend_from_slice(&payload);
        assert_eq!(buf, expected);
    }

    #[test]
    fn bytes_roundtrip_empty() {
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Bytes(Bytes::new()));
        let (val, _) = deserialize_value(&buf).unwrap();
        assert_eq!(val, Value::Bytes(Bytes::new()));
    }

    // -------------------------------------------------------------------------
    // Value::Address  (tag 0x08, 20 raw bytes, NO length prefix)
    // -------------------------------------------------------------------------

    #[test]
    fn address_decode() {
        let addr_bytes = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
            0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33,
        ];
        let mut bytes = vec![0x08u8];
        bytes.extend_from_slice(&addr_bytes);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, 21); // tag + 20 bytes, no length prefix
        if let Value::Address(a) = val {
            assert_eq!(a.as_slice(), &addr_bytes);
        } else {
            panic!("expected Address, got {:?}", val);
        }
    }

    #[test]
    fn address_encode_no_length_prefix() {
        let addr_bytes = [0xabu8; 20];
        let addr = crate::primitives::Address::from_slice(&addr_bytes);
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Address(addr));
        // Must be exactly tag + 20 bytes = 21 bytes (no length prefix)
        assert_eq!(buf.len(), 21);
        assert_eq!(buf[0], 0x08);
        assert_eq!(&buf[1..], &addr_bytes);
    }

    #[test]
    fn address_roundtrip() {
        let addr_bytes = [0x42u8; 20];
        let addr = crate::primitives::Address::from_slice(&addr_bytes);
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Address(addr.clone()));
        let (val, _) = deserialize_value(&buf).unwrap();
        assert_eq!(val, Value::Address(addr));
    }

    // -------------------------------------------------------------------------
    // Value::Array  (tag 0x09, len:u32 LE, then len tagged Values)
    // -------------------------------------------------------------------------

    #[test]
    fn array_decode_empty() {
        let mut bytes = vec![0x09u8];
        bytes.extend_from_slice(&le32(0));
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Array(vec![]));
        assert_eq!(consumed, 5);
    }

    #[test]
    fn array_decode_mixed() {
        // [Int(1), Bool(true)]
        let mut bytes = vec![0x09u8];
        bytes.extend_from_slice(&le32(2)); // count = 2
        // Int(1)
        bytes.push(0x02);
        bytes.extend_from_slice(&1i32.to_le_bytes());
        // Bool(true)
        bytes.push(0x06);
        bytes.push(0x01);

        let (val, _) = deserialize_value(&bytes).unwrap();
        assert_eq!(val, Value::Array(vec![Value::Int(1), Value::Bool(true)]));
    }

    #[test]
    fn array_encode() {
        let arr = vec![Value::Int(1), Value::Bool(true)];
        let mut buf = Vec::new();
        serialize_value(&mut buf, &Value::Array(arr.clone()));

        assert_eq!(buf[0], 0x09);
        let count = u32::from_le_bytes(buf[1..5].try_into().unwrap());
        assert_eq!(count, 2);
    }

    #[test]
    fn array_roundtrip_nested() {
        let inner = Value::Array(vec![Value::Int(1), Value::Int(2)]);
        let outer = Value::Array(vec![inner, Value::String("x".into())]);
        let mut buf = Vec::new();
        serialize_value(&mut buf, &outer);
        let (val, _) = deserialize_value(&buf).unwrap();
        assert_eq!(val, outer);
    }

    // -------------------------------------------------------------------------
    // Entity round-trips
    // -------------------------------------------------------------------------

    #[test]
    fn entity_roundtrip_basic() {
        // spec worked example: { id: "tx-1", value: 42, active: true }
        let mut entity = Entity::new();
        entity.set("id", Value::String("tx-1".into()));
        entity.set("value", Value::Int(42));
        entity.set("active", Value::Bool(true));

        let bytes = serialize_entity(&entity);
        let recovered = deserialize_entity(&bytes).unwrap();

        assert_eq!(recovered.get("id"), Some(&Value::String("tx-1".into())));
        assert_eq!(recovered.get("value"), Some(&Value::Int(42)));
        assert_eq!(recovered.get("active"), Some(&Value::Bool(true)));
    }

    #[test]
    fn entity_roundtrip_all_value_types() {
        let addr_bytes = [0x01u8; 20];
        let mut entity = Entity::new();
        entity.set("s", Value::String("hello".into()));
        entity.set("i", Value::Int(-99));
        entity.set("i8", Value::Int8(i64::MAX));
        entity.set("bi", Value::BigInt(BigInt::from(u64::MAX)));
        entity.set("bd", Value::BigDecimal(BigDecimal::from_str("1.5").unwrap()));
        entity.set("b", Value::Bool(false));
        entity.set("by", Value::Bytes(Bytes::from_slice(&[0xde, 0xad])));
        entity.set("addr", Value::Address(crate::primitives::Address::from_slice(&addr_bytes)));
        entity.set("arr", Value::Array(vec![Value::Int(1), Value::Int(2)]));
        entity.set("null", Value::Null);

        let bytes = serialize_entity(&entity);
        let recovered = deserialize_entity(&bytes).unwrap();

        assert_eq!(recovered.get("s"), entity.get("s"));
        assert_eq!(recovered.get("i"), entity.get("i"));
        assert_eq!(recovered.get("i8"), entity.get("i8"));
        assert_eq!(recovered.get("bi"), entity.get("bi"));
        assert_eq!(recovered.get("b"), entity.get("b"));
        assert_eq!(recovered.get("by"), entity.get("by"));
        assert_eq!(recovered.get("addr"), entity.get("addr"));
        assert_eq!(recovered.get("arr"), entity.get("arr"));
        assert_eq!(recovered.get("null"), entity.get("null"));
        // BigDecimal: compare via re-serialization
        let mut bd_buf1 = Vec::new();
        let mut bd_buf2 = Vec::new();
        serialize_value(&mut bd_buf1, entity.get("bd").unwrap());
        serialize_value(&mut bd_buf2, recovered.get("bd").unwrap());
        assert_eq!(bd_buf1, bd_buf2);
    }

    #[test]
    fn entity_field_count_in_header() {
        let mut entity = Entity::new();
        entity.set("a", Value::Int(1));
        entity.set("b", Value::Int(2));
        entity.set("c", Value::Int(3));

        let bytes = serialize_entity(&entity);
        let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        assert_eq!(count, 3);
    }

    // -------------------------------------------------------------------------
    // Known-bad BE vector: verify the fix (old BE code would produce wrong value)
    // -------------------------------------------------------------------------

    #[test]
    fn bigint_rejects_be_interpretation() {
        // Encode 256 as signed-LE: bytes [0x00, 0x01] (little-endian)
        // If decoded as BE it would be 1 instead of 256.
        let n = BigInt::from(256u64);
        let le = n.to_signed_bytes_le();
        assert_eq!(le, vec![0x00, 0x01]);

        let mut bytes = vec![0x04u8];
        bytes.extend_from_slice(&le32(le.len() as u32));
        bytes.extend_from_slice(&le);

        let (val, _) = deserialize_value(&bytes).unwrap();
        // Must decode as 256, NOT 1 (which BE would give)
        assert_eq!(val, Value::BigInt(BigInt::from(256u64)));
    }

    // -------------------------------------------------------------------------
    // TlvReader::skip_value — verify BigDecimal skip is len:u32 + bytes
    // -------------------------------------------------------------------------

    #[test]
    fn skip_bigdecimal_uses_string_format() {
        use crate::decode::TlvReader;

        // Build: [BigDecimal("9.99"), Int(7)]
        let s = b"9.99";
        let mut bytes = vec![0x05u8];
        bytes.extend_from_slice(&le32(s.len() as u32));
        bytes.extend_from_slice(s);
        bytes.push(0x02); // Int tag
        bytes.extend_from_slice(&7i32.to_le_bytes());

        let mut reader = TlvReader::new(&bytes);
        reader.skip_value().unwrap(); // skip the BigDecimal (tag + len + "9.99")
        // Now positioned at the Int tag; read tag then value
        let tag = reader.read_u8().unwrap();
        assert_eq!(tag, 0x02);
        let n = reader.read_i32().unwrap();
        assert_eq!(n, 7);
    }
}
