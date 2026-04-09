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
//! This crate is `no_std`. Use `alloc::string::String` and `alloc::vec::Vec`
//! as usual — they are re-exported from the crate root for convenience.

#![no_std]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

pub mod alloc_impl;
pub mod as_types;
pub mod class_ids;
pub mod ethereum;
pub mod ffi;
pub mod panic_handler;

pub use alloc::{string::String, vec::Vec};
