//! EntityBuilder — constructs AS TypedMap<string, Value> objects for store.set.
//!
//! Usage:
//! ```rust,ignore
//! let mut b = EntityBuilder::new();
//! b.set_string("id", &self.id);
//! b.set_bytes("from", &self.from);
//! b.set_bigint("value", &self.value);
//! unsafe { ffi::store_set(entity_type_ptr, id_ptr, b.build()); }
//! ```

use crate::as_types::{
    new_asc_big_int, new_asc_bytes, new_asc_string, new_typed_map, new_value_big_int,
    new_value_bool, new_value_bytes, new_value_int, new_value_int8, new_value_string,
};
use alloc::vec::Vec;

/// Builder for an AS TypedMap<string, Value> to pass to store.set.
pub struct EntityBuilder {
    fields: Vec<(&'static str, u32)>,
}

impl EntityBuilder {
    /// Create a new, empty builder.
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Set a string field.
    pub fn set_string(&mut self, key: &'static str, value: &str) {
        let str_ptr = new_asc_string(value);
        let val_ptr = new_value_string(str_ptr);
        self.fields.push((key, val_ptr));
    }

    /// Set a bytes field from a slice.
    pub fn set_bytes(&mut self, key: &'static str, value: &[u8]) {
        let bytes_ptr = new_asc_bytes(value);
        let val_ptr = new_value_bytes(bytes_ptr);
        self.fields.push((key, val_ptr));
    }

    /// Set a BigInt field from little-endian bytes.
    pub fn set_bigint(&mut self, key: &'static str, value: &[u8]) {
        let bi_ptr = new_asc_big_int(value);
        let val_ptr = new_value_big_int(bi_ptr);
        self.fields.push((key, val_ptr));
    }

    /// Set a boolean field.
    pub fn set_bool(&mut self, key: &'static str, value: bool) {
        let val_ptr = new_value_bool(value);
        self.fields.push((key, val_ptr));
    }

    /// Set an i32 field.
    pub fn set_i32(&mut self, key: &'static str, value: i32) {
        let val_ptr = new_value_int(value);
        self.fields.push((key, val_ptr));
    }

    /// Set an i64 field (Int8 / Timestamp scalars).
    pub fn set_i64(&mut self, key: &'static str, value: i64) {
        let val_ptr = new_value_int8(value);
        self.fields.push((key, val_ptr));
    }

    /// Build and return the AscPtr<TypedMap<string, Value>>.
    pub fn build(self) -> u32 {
        new_typed_map(&self.fields)
    }
}
