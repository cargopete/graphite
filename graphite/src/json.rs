//! JSON parsing for subgraph handlers.
//!
//! The primary use case is decoding NFT metadata fetched via `ipfs.cat`:
//!
//! ```rust,ignore
//! let bytes = host.ipfs_cat("QmHash").unwrap_or_default();
//! if let Some(meta) = json::from_bytes(&bytes) {
//!     if let Some(name) = meta.get("name").and_then(|v| v.as_str()) {
//!         token.name = name.to_string();
//!     }
//! }
//! ```
//!
//! # WASM vs native behaviour
//!
//! On WASM, `from_bytes` calls the `json.fromBytes` graph-node host function and
//! decodes the returned AS `JSONValue` object from linear memory.
//!
//! On native (`cargo test`), it uses `serde_json` directly — no host required.

pub use graph_as_runtime::json::JsonValue;

/// Parse JSON from raw bytes.
///
/// Returns `None` if the input is not valid JSON.
/// On WASM, graph-node does the parsing; on native, `serde_json` is used.
pub fn from_bytes(data: &[u8]) -> Option<JsonValue> {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_as_runtime::{as_types, ffi, json::read_json_value};
        let data_ptr = as_types::new_asc_bytes(data);
        let result_ptr = unsafe { ffi::json_from_bytes(data_ptr) };
        if result_ptr == 0 {
            return None;
        }
        Some(unsafe { read_json_value(result_ptr) })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let v: serde_json::Value = serde_json::from_slice(data).ok()?;
        Some(serde_to_json_value(v))
    }
}

/// Parse JSON from a UTF-8 string slice.
///
/// Convenience wrapper around [`from_bytes`].
pub fn from_str(s: &str) -> Option<JsonValue> {
    from_bytes(s.as_bytes())
}

// ============================================================================
// Native: convert serde_json::Value → JsonValue
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
fn serde_to_json_value(v: serde_json::Value) -> JsonValue {
    match v {
        serde_json::Value::Null => JsonValue::Null,
        serde_json::Value::Bool(b) => JsonValue::Bool(b),
        serde_json::Value::Number(n) => JsonValue::Number(n.to_string()),
        serde_json::Value::String(s) => JsonValue::String(s),
        serde_json::Value::Array(arr) => {
            JsonValue::Array(arr.into_iter().map(serde_to_json_value).collect())
        }
        serde_json::Value::Object(obj) => {
            JsonValue::Object(obj.into_iter().map(|(k, v)| (k, serde_to_json_value(v))).collect())
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_object() {
        let json = from_str(r#"{"name":"CryptoPunk #1","description":"A punk","id":1}"#).unwrap();
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("CryptoPunk #1"));
        assert_eq!(json.get("description").and_then(|v| v.as_str()), Some("A punk"));
        assert_eq!(json.get("id").and_then(|v| v.as_number_str()), Some("1"));
        assert!(json.get("missing").is_none());
    }

    #[test]
    fn parse_nested_object() {
        let json = from_str(r#"{"attributes":[{"trait_type":"Eyes","value":"Laser"}]}"#).unwrap();
        let attrs = json.get("attributes").and_then(|v| v.as_array()).unwrap();
        assert_eq!(attrs.len(), 1);
        assert_eq!(
            attrs[0].get("trait_type").and_then(|v| v.as_str()),
            Some("Eyes")
        );
    }

    #[test]
    fn parse_array_at_root() {
        let json = from_str(r#"[1, 2, 3]"#).unwrap();
        assert_eq!(json.get_index(0).and_then(|v| v.as_number_str()), Some("1"));
        assert_eq!(json.get_index(2).and_then(|v| v.as_number_str()), Some("3"));
        assert!(json.get_index(3).is_none());
    }

    #[test]
    fn parse_bools_and_null() {
        let json = from_str(r#"{"ok":true,"fail":false,"nothing":null}"#).unwrap();
        assert_eq!(json.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(json.get("fail").and_then(|v| v.as_bool()), Some(false));
        assert!(json.get("nothing").map(|v| v.is_null()).unwrap_or(false));
    }

    #[test]
    fn invalid_json_returns_none() {
        assert!(from_str("not json at all {{{{").is_none());
    }

    #[test]
    fn from_bytes_works() {
        let data = br#"{"key":"value"}"#;
        let json = from_bytes(data).unwrap();
        assert_eq!(json.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn nft_metadata_pattern() {
        let metadata = r#"{
            "name": "Bored Ape #1234",
            "description": "A bored ape.",
            "image": "ipfs://QmImageHash",
            "attributes": [
                {"trait_type": "Background", "value": "Blue"},
                {"trait_type": "Eyes",       "value": "Laser Eyes"}
            ]
        }"#;

        let json = from_str(metadata).unwrap();

        let name = json.get("name").and_then(|v| v.as_str()).unwrap();
        assert_eq!(name, "Bored Ape #1234");

        let attrs = json.get("attributes").and_then(|v| v.as_array()).unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(
            attrs[1].get("value").and_then(|v| v.as_str()),
            Some("Laser Eyes")
        );
    }
}
