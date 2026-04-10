//! graph-as-runtime — AssemblyScript ABI layer for graph-node compatibility.
//!
//! This crate provides the glue between Rust subgraph handlers and the real,
//! unmodified graph-node runtime. Graph-node expects WASM modules that speak
//! the AssemblyScript (AS) memory model: UTF-16LE strings, TypedMap objects,
//! reference-counted heap objects with a 16-byte header, etc.
//!
//! # Architecture
//!
//! ```text
//!   Rust handler code
//!        |
//!   [ graph-as-runtime ]   <-- this crate
//!        |  - bump allocator (AS-compatible __new/__pin/__unpin exports)
//!        |  - AS type constructors (String, TypedMap, Value, Array)
//!        |  - FFI shim: store_set(entity_ptr, id_ptr, data_ptr) via "env" module
//!        |
//!   [ graph-node AS runtime ]
//! ```
//!
//! # no_std
//!
//! This crate is `no_std` when targeting `wasm32`. For native builds (tests),
//! it links against the standard library so thread-locals and the system
//! allocator are available.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

/// WASM-only: bump allocator and AS runtime exports.
#[cfg(target_arch = "wasm32")]
pub mod alloc_impl;

/// WASM-only: AS type constructors (string, bytes, TypedMap, Value, ...).
#[cfg(target_arch = "wasm32")]
pub mod as_types;

/// WASM-only: EntityBuilder for constructing AS TypedMap objects.
#[cfg(target_arch = "wasm32")]
pub mod entity;

/// WASM-only: read AS TypedMap objects returned by store.get.
#[cfg(target_arch = "wasm32")]
pub mod store_read;

/// WASM-only: panic handler that forwards to graph-node's abort.
#[cfg(target_arch = "wasm32")]
pub mod panic_handler;

pub mod class_ids;
pub mod ethereum;
pub mod ffi;
pub mod json;

/// Native-only: thread-local in-memory store for unit tests.
#[cfg(not(target_arch = "wasm32"))]
pub mod native_store;

pub use alloc::{string::String, vec::Vec};
