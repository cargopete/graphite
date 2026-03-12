//! Code generation for Graphite subgraphs.
//!
//! This module handles:
//! - ABI → Rust event structs
//! - Schema.graphql → Rust entity structs (future)

pub mod abi;

pub use abi::generate_abi_bindings;
