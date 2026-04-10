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
