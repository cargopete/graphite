//! WASM ABI layer for graph-node integration.
//!
//! This module provides the FFI bindings between Rust handlers and
//! graph-node's WASM runtime using a **Rust-native ABI**.
//!
//! # Design
//!
//! Unlike AssemblyScript subgraphs which use AS memory layout (TypedMap,
//! UTF-16 strings, etc.), Rust subgraphs use a clean ptr+len ABI with
//! bincode serialization. This requires graph-node modifications to detect
//! `language: wasm/rust` in the manifest and use the Rust ABI host functions.
//!
//! # ABI Protocol
//!
//! - Strings: UTF-8, passed as (ptr, len) pairs
//! - Entities: bincode-serialized, passed as (ptr, len)
//! - Return values: written to a caller-provided buffer
//! - Memory: bump allocator with arena reset after each handler

#[cfg(target_arch = "wasm32")]
pub mod ffi;

#[cfg(target_arch = "wasm32")]
pub mod host;

#[cfg(target_arch = "wasm32")]
pub mod alloc;

#[cfg(target_arch = "wasm32")]
pub use host::WasmHost;
