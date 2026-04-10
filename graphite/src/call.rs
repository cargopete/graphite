//! Ethereum contract call builder.
//!
//! `ContractCall` encodes calldata and dispatches through the `HostFunctions`
//! trait, which means it works both in WASM (via graph-node's `ethereum.call`
//! host import) and in native tests (via `MockHost`).
//!
//! # Example
//!
//! ```rust,ignore
//! use graphite::call::ContractCall;
//! use graphite::testing::addr;
//!
//! let price = ContractCall::new(oracle_address, "latestAnswer()(int256)")
//!     .call(&host)?;
//! ```
//!
//! For calls with arguments, ABI-encode them with `alloy-sol-types` and pass
//! the encoded bytes (without the selector) to `with_args`:
//!
//! ```rust,ignore
//! use alloy_sol_types::{sol_data, SolType};
//!
//! let encoded = sol_data::Address::abi_encode(&wallet_address);
//! let balance_bytes = ContractCall::new(token, "balanceOf(address)(uint256)")
//!     .with_args(&encoded)
//!     .call(&host)?;
//! ```

use crate::crypto;
use crate::host::{EthereumCallError, HostFunctions};
use crate::primitives::{Address, Bytes};

/// A builder for an Ethereum read-only contract call.
///
/// Construct with [`ContractCall::new`], optionally append encoded arguments
/// via [`ContractCall::with_args`], then dispatch with [`ContractCall::call`].
pub struct ContractCall {
    address: Address,
    /// ABI-encoded calldata: 4-byte selector followed by encoded arguments.
    calldata: alloc::vec::Vec<u8>,
}

impl ContractCall {
    /// Create a new call targeting `address` with the given Solidity `signature`.
    ///
    /// The selector is computed from the input parameter list of `signature`.
    /// Return types (e.g. the `(int256)` in `"latestAnswer()(int256)"`) are
    /// stripped before hashing, so either form is accepted.
    pub fn new(address: Address, signature: &str) -> Self {
        let sel = crypto::selector(signature);
        let mut calldata = alloc::vec::Vec::with_capacity(4);
        calldata.extend_from_slice(&sel);
        Self { address, calldata }
    }

    /// Append ABI-encoded arguments to the calldata.
    ///
    /// Use `alloy_sol_types` to encode arguments. Pass only the parameter
    /// bytes — the selector is already included from [`ContractCall::new`].
    pub fn with_args(mut self, encoded_args: &[u8]) -> Self {
        self.calldata.extend_from_slice(encoded_args);
        self
    }

    /// Execute the call through the host and return the raw response bytes.
    ///
    /// Returns `Err` if the call reverts or graph-node reports an error.
    pub fn call(self, host: &impl HostFunctions) -> Result<Bytes, EthereumCallError> {
        host.ethereum_call_raw(self.address, &self.calldata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockHost;

    fn zero_address() -> Address {
        Address::from([0u8; 20])
    }

    #[test]
    fn selector_encoded_correctly() {
        // transfer(address,uint256) selector = 0xa9059cbb
        let call = ContractCall::new(zero_address(), "transfer(address,uint256)");
        assert_eq!(&call.calldata[..4], &[0xa9, 0x05, 0x9c, 0xbb]);
    }

    #[test]
    fn with_args_appends_after_selector() {
        let args = [0xde, 0xad, 0xbe, 0xef];
        let call = ContractCall::new(zero_address(), "transfer(address,uint256)")
            .with_args(&args);
        assert_eq!(call.calldata.len(), 8);
        assert_eq!(&call.calldata[4..], &args);
    }

    #[test]
    fn call_returns_mock_response() {
        let mut host = MockHost::new();
        let addr = zero_address();
        let sel = crate::crypto::selector("latestAnswer()(int256)");
        let expected: Bytes = alloc::vec![0x00, 0x00, 0x00, 0x42].into();
        host.mock_eth_call_raw(addr, sel.to_vec(), Ok(expected.clone()));

        let result = ContractCall::new(addr, "latestAnswer()(int256)")
            .call(&host)
            .expect("call should succeed");
        assert_eq!(result, expected);
    }

    #[test]
    fn call_returns_err_when_no_mock() {
        let host = MockHost::new();
        let result = ContractCall::new(zero_address(), "nonexistent()").call(&host);
        assert!(result.is_err());
    }
}
