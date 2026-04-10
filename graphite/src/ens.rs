//! ENS name resolution for subgraph handlers.
//!
//! # Example
//!
//! ```rust,ignore
//! if let Some(name) = ens::name_by_address(event.params.from) {
//!     account.ens_name = Some(name);
//! }
//! ```
//!
//! # WASM vs native behaviour
//!
//! On WASM, `name_by_address` calls the `ens.nameByAddress` graph-node host function.
//!
//! On native (`cargo test`), it checks the thread-local mock registry populated by
//! `mock::set_ens_name`.

use crate::primitives::Address;
use alloc::string::String;

/// Look up the ENS name registered for an Ethereum address.
///
/// Returns `None` if no name is registered for the address, or when running
/// on a network where ENS is not deployed (e.g., non-mainnet).
pub fn name_by_address(address: Address) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_as_runtime::{as_types::new_asc_bytes, ffi, store_read::read_asc_string};
        let addr_ptr = new_asc_bytes(address.as_slice());
        let result_ptr = unsafe { ffi::ens_name_by_address(addr_ptr) };
        if result_ptr == 0 {
            return None;
        }
        let name = unsafe { read_asc_string(result_ptr) };
        if name.is_empty() { None } else { Some(name) }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let addr: [u8; 20] = address.as_slice().try_into().ok()?;
        crate::mock::get_ens_name(&addr)
    }
}
