//! WASM ABI layer for graph-node integration.
//!
//! This module provides the FFI bindings between Rust handlers and
//! graph-node's WASM runtime. It defines:
//!
//! - External host functions imported from graph-node
//! - Memory allocation functions exported to graph-node
//! - A global host context for handler code to use
//!
//! # Architecture
//!
//! Graph-node expects WASM modules to:
//! 1. Export `memory` (the linear memory)
//! 2. Export allocation functions (`allocate`, `deallocate`)
//! 3. Export handler functions that graph-node calls with event data
//! 4. Import host functions for store operations, ethereum calls, etc.

#[cfg(target_arch = "wasm32")]
pub mod ffi;

#[cfg(target_arch = "wasm32")]
pub mod host;

#[cfg(target_arch = "wasm32")]
pub mod alloc;

#[cfg(target_arch = "wasm32")]
pub use host::WasmHost;

// Re-export the AscPtr type for generated code
#[cfg(target_arch = "wasm32")]
pub use ffi::AscPtr;
