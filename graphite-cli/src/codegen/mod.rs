//! Code generation for Graphite subgraphs.
//!
//! This module handles:
//! - ABI → Rust event structs
//! - Schema.graphql → Rust entity structs

pub mod abi;
pub mod schema;

pub use abi::generate_abi_bindings;
pub use schema::generate_schema_entities;
