//! Graphite — Rust SDK for building subgraphs on The Graph
//!
//! This crate provides ergonomic Rust bindings for writing subgraph mappings,
//! eliminating AssemblyScript's pain points while delivering type safety and
//! access to Rust's ecosystem.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use graphite::prelude::*;
//!
//! #[derive(Entity)]
//! pub struct Transfer {
//!     #[id]
//!     id: Bytes,
//!     from: Address,
//!     to: Address,
//!     value: BigInt,
//! }
//!
//! #[handler]
//! pub fn handle_transfer(event: TransferEvent) {
//!     let mut transfer = Transfer::new(&event.id());
//!     transfer.from = event.params.from;
//!     transfer.to = event.params.to;
//!     transfer.value = event.params.value.into();
//!     transfer.save();
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod call;
pub mod crypto;
pub mod data_source;
pub mod host;
pub mod primitives;
pub mod store;
pub mod testing;

/// Mock host for native `cargo test` — no WASM, no graph-node required.
#[cfg(not(target_arch = "wasm32"))]
pub mod mock;

/// Prelude module — import everything you need with `use graphite::prelude::*`
pub mod prelude {
    pub use crate::host::HostFunctions;
    pub use crate::primitives::{Address, AddressExt, B256, BigDecimal, BigInt, Bytes, U256};
    pub use crate::store::{Entity, FromValue, Store, Value};
    pub use graphite_macros::{Entity, handler};

    #[cfg(feature = "std")]
    pub use crate::testing::MockHost;
}

// Re-export key types at crate root
pub use primitives::{Address, AddressExt, B256, BigDecimal, BigInt, Bytes, U256};
/// Transaction receipt exposed to event handlers. Present only when the manifest
/// sets `receipt: true` on the data source mapping.
pub use graph_as_runtime::ethereum::EthereumTransactionReceipt as TransactionReceipt;

/// Context passed to every event handler alongside the decoded event.
///
/// Contains block and transaction metadata extracted from the EthereumEvent
/// AS object by `graph_as_runtime::ethereum::read_ethereum_event`.
pub struct EventContext {
    /// Block number as little-endian BigInt bytes.
    pub block_number: alloc::vec::Vec<u8>,
    /// Block timestamp as little-endian BigInt bytes.
    pub block_timestamp: alloc::vec::Vec<u8>,
    /// Transaction hash (32 bytes).
    pub tx_hash: [u8; 32],
    /// Log index as little-endian BigInt bytes.
    pub log_index: alloc::vec::Vec<u8>,
    /// Contract address that emitted the event (20 bytes).
    pub address: [u8; 20],
    /// Transaction receipt, if the manifest enables `receipt: true`.
    pub receipt: Option<crate::TransactionReceipt>,
}

/// Context passed to every call handler alongside the decoded call inputs.
///
/// Contains block and transaction metadata extracted from the EthereumCall
/// AS object by `graph_as_runtime::ethereum::read_ethereum_call`.
pub struct CallContext {
    /// Block number as little-endian BigInt bytes.
    pub block_number: alloc::vec::Vec<u8>,
    /// Block timestamp as little-endian BigInt bytes.
    pub block_timestamp: alloc::vec::Vec<u8>,
    /// Transaction hash (32 bytes).
    pub tx_hash: [u8; 32],
    /// Contract address that was called (20 bytes).
    pub address: [u8; 20],
    /// Transaction sender address (20 bytes).
    pub from: [u8; 20],
}
