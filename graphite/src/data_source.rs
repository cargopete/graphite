//! Dynamic data source helpers.
//!
//! These functions wrap the `HostFunctions` trait methods for dynamic data
//! source creation and introspection, providing a slightly friendlier API
//! than calling the trait methods directly.
//!
//! # Example
//!
//! ```rust,ignore
//! use graphite::data_source;
//! use graphite::prelude::*;
//!
//! // Inside a factory event handler:
//! data_source::create(host, "PairTemplate", pair_address);
//!
//! // Inside a template event handler:
//! let addr = data_source::address(host);
//! let net  = data_source::network(host);
//! ```

use crate::host::HostFunctions;
use crate::primitives::Address;
use crate::store::Entity;
use alloc::string::String;

/// Instantiate a data source template for `address`.
///
/// `name` must match a template name declared in `subgraph.yaml`.
/// The address is passed as a lowercase hex string with `0x` prefix,
/// matching the convention used by graph-ts.
pub fn create(host: &mut impl HostFunctions, name: &str, address: Address) {
    let hex = address_to_hex(address);
    host.data_source_create(name, &[hex]);
}

/// Instantiate a data source template for `address` with a key-value context.
///
/// The context is retrievable inside the template's handlers via
/// `data_source::context_current()`.
pub fn create_with_context(
    host: &mut impl HostFunctions,
    name: &str,
    address: Address,
    context: Entity,
) {
    let hex = address_to_hex(address);
    host.data_source_create_with_context(name, &[hex], context);
}

// ============================================================================
// Hostless API — for use inside `#[handler]` functions
// ============================================================================

/// Instantiate a `file/ipfs` data source template for the given IPFS CID.
///
/// Call this from an event handler to trigger an IPFS content fetch. `name`
/// must match a `file/ipfs` template in the manifest.
///
/// In WASM builds, calls `dataSource.create` directly.
/// In native tests, records the creation in the thread-local mock (cleared by
/// `mock::reset()`). Inspect results via `mock::get_created_data_sources()`.
pub fn create_file(name: &str, cid: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_as_runtime::{
            as_types::{new_asc_string, new_asc_string_array},
            ffi,
        };
        let name_ptr = new_asc_string(name);
        let params_ptr = new_asc_string_array(&[cid]);
        unsafe { ffi::data_source_create(name_ptr, params_ptr) };
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::mock::record_data_source_created(name, &[cid.to_string()]);
    }
}

/// Instantiate a `file/ipfs` data source template with a key-value context.
///
/// The context is a slice of `(key, value)` string pairs. Inside the file
/// handler, retrieve them via `data_source::context_current()`.
///
/// In WASM builds, calls `dataSource.createWithContext` directly.
/// In native tests, sets the context in the thread-local mock so that
/// `context_current()` returns the correct values.
pub fn create_file_with_context(name: &str, cid: &str, context: &[(&str, &str)]) {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_as_runtime::{
            as_types::{new_asc_string, new_asc_string_array, new_typed_map, new_value_string},
            ffi,
        };
        let name_ptr = new_asc_string(name);
        let params_ptr = new_asc_string_array(&[cid]);
        let ctx_entries: alloc::vec::Vec<(&str, u32)> = context
            .iter()
            .map(|(k, v)| {
                let v_ptr = new_asc_string(v);
                let val_ptr = new_value_string(v_ptr);
                (*k, val_ptr)
            })
            .collect();
        let ctx_ptr = new_typed_map(&ctx_entries);
        unsafe { ffi::data_source_create_with_context(name_ptr, params_ptr, ctx_ptr) };
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::mock::record_data_source_created(name, &[cid.to_string()]);
        for (k, v) in context {
            crate::mock::set_data_source_context(*k, *v);
        }
    }
}

/// Return the context of the currently-executing data source (hostless).
///
/// Reads the context that was passed to `dataSource.createWithContext` when
/// this template instance was started. Returns an empty entity if no context
/// was set.
///
/// For use in `#[handler]` and `#[handler(file)]` functions.
pub fn context_current() -> Entity {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_as_runtime::{ffi, store_read::read_typed_map};
        let map_ptr = unsafe { ffi::data_source_context() };
        let entries = unsafe { read_typed_map(map_ptr) };
        let mut entity = Entity::new();
        for (k, v) in entries {
            use graph_as_runtime::store_read::StoreValue;
            let val = match v {
                StoreValue::String(s) => crate::store::Value::String(s),
                StoreValue::Bool(b) => crate::store::Value::Bool(b),
                StoreValue::Int(n) => crate::store::Value::Int(n),
                StoreValue::Int8(n) => crate::store::Value::Int8(n),
                StoreValue::Bytes(b) => {
                    crate::store::Value::Bytes(crate::primitives::Bytes::from_slice(&b))
                }
                StoreValue::BigInt(b) => crate::store::Value::BigInt(
                    crate::primitives::BigInt::from_signed_bytes_le(&b),
                ),
                StoreValue::Null => crate::store::Value::Null,
            };
            entity.set(k, val);
        }
        entity
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::mock::get_data_source_context()
    }
}

/// Look up a single string value from the current data source context (hostless).
///
/// Shorthand for `context_current().get(key).and_then(|v| v.as_str())`.
pub fn context_string(key: &str) -> Option<alloc::string::String> {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_as_runtime::{ffi, store_read::{read_typed_map, StoreValue}};
        let map_ptr = unsafe { ffi::data_source_context() };
        let entries = unsafe { read_typed_map(map_ptr) };
        for (k, v) in entries {
            if k == key {
                if let StoreValue::String(s) = v {
                    return Some(s);
                }
            }
        }
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::mock::get_data_source_context_string(key)
    }
}

/// Return the unique identifier of the current data source (hostless).
///
/// For use in `#[handler]` and `#[handler(file)]` functions.
pub fn id_current() -> alloc::string::String {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_as_runtime::{ffi, store_read::read_asc_string};
        let ptr = unsafe { ffi::data_source_id() };
        unsafe { read_asc_string(ptr) }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        alloc::string::String::new()
    }
}

/// Return the address of the currently-executing data source (hostless).
///
/// For use in `#[handler]` functions on ethereum data source templates.
pub fn address_current() -> [u8; 20] {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_as_runtime::{ffi, store_read::read_asc_bytes};
        let ptr = unsafe { ffi::data_source_address() };
        let bytes = unsafe { read_asc_bytes(ptr) };
        let mut addr = [0u8; 20];
        let len = bytes.len().min(20);
        addr[..len].copy_from_slice(&bytes[..len]);
        addr
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        [0u8; 20]
    }
}

/// Return the network name of the currently-executing data source (hostless).
///
/// For use in `#[handler]` functions.
pub fn network_current() -> alloc::string::String {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_as_runtime::{ffi, store_read::read_asc_string};
        let ptr = unsafe { ffi::data_source_network() };
        unsafe { read_asc_string(ptr) }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        alloc::string::String::from("mainnet")
    }
}

/// Return the address of the currently-executing data source.
///
/// In WASM builds this calls `dataSource.address()`. In native tests the
/// value comes from [`MockHost::current_address`].
pub fn address(host: &impl HostFunctions) -> Address {
    host.data_source_address()
}

/// Return the network name of the currently-executing data source.
///
/// Typical values: `"mainnet"`, `"arbitrum-one"`, `"matic"`.
pub fn network(host: &impl HostFunctions) -> String {
    host.data_source_network()
}

/// Return the context of the currently-executing data source.
///
/// The context is the key-value map passed as the second argument to
/// `dataSource.createWithContext` when this dynamic data source was
/// instantiated. Returns an empty entity if no context was set.
pub fn context(host: &impl HostFunctions) -> Entity {
    host.data_source_context()
}

/// Return the unique identifier of the currently-executing data source.
///
/// For dynamic data sources created from templates, graph-node combines the
/// template name with the address. Static data sources return their manifest
/// name.
pub fn id(host: &impl HostFunctions) -> String {
    host.data_source_id()
}

/// Format an `Address` as a lowercase `0x`-prefixed hex string.
fn address_to_hex(address: Address) -> String {
    let mut s = String::with_capacity(42);
    s.push_str("0x");
    for byte in address.as_slice() {
        let hi = (byte >> 4) as usize;
        let lo = (byte & 0xf) as usize;
        s.push(HEX_CHARS[hi]);
        s.push(HEX_CHARS[lo]);
    }
    s
}

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockHost;

    fn make_addr(byte: u8) -> Address {
        Address::from([byte; 20])
    }

    #[test]
    fn address_to_hex_zero() {
        let hex = address_to_hex(Address::from([0u8; 20]));
        assert_eq!(hex, "0x0000000000000000000000000000000000000000");
    }

    #[test]
    fn address_to_hex_ff() {
        let hex = address_to_hex(Address::from([0xffu8; 20]));
        assert_eq!(hex, "0xffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn create_records_in_mock_host() {
        let mut host = MockHost::new();
        let addr = make_addr(0xab);
        create(&mut host, "PairTemplate", addr);

        assert_eq!(host.created_data_sources.len(), 1);
        let (name, params) = &host.created_data_sources[0];
        assert_eq!(name, "PairTemplate");
        // 20 bytes all 0xab → 40 hex chars after "0x"
        assert_eq!(params[0], "0xabababababababababababababababababababab");
    }

    #[test]
    fn address_and_network_from_mock() {
        let addr = make_addr(0x01);
        let host = MockHost::new()
            .with_address(addr)
            .with_network("arbitrum-one");

        assert_eq!(address(&host), addr);
        assert_eq!(network(&host), "arbitrum-one");
    }
}
