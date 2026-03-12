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

pub mod decode;
pub mod host;
pub mod primitives;
pub mod store;
pub mod testing;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// Prelude module — import everything you need with `use graphite::prelude::*`
pub mod prelude {
    pub use crate::decode::{DecodeError, EventDecode, FromWasmBytes, RawLog, TlvReader};
    pub use crate::host::HostFunctions;
    pub use crate::primitives::{Address, BigDecimal, BigInt, Bytes, B256, U256};
    pub use crate::store::{Entity, FromValue, Store, Value};
    pub use graphite_macros::{handler, Entity};

    #[cfg(feature = "std")]
    pub use crate::testing::MockHost;

    #[cfg(target_arch = "wasm32")]
    pub use crate::wasm::WasmHost;
}

// Re-export key types at crate root
pub use primitives::{Address, BigDecimal, BigInt, Bytes, B256, U256};
