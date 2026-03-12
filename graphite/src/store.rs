//! Entity storage types and traits.
//!
//! Defines the `Entity` type used for serializing data to graph-node's store,
//! and the `Value` enum representing field values.

use crate::primitives::{Address, BigDecimal, BigInt, Bytes, B256};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// A stored entity, represented as a map of field names to values.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Entity {
    fields: BTreeMap<String, Value>,
}

impl Entity {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a field value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.fields.insert(key.into(), value.into());
    }

    /// Get a field value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }

    /// Remove a field.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.fields.remove(key)
    }

    /// Iterate over all fields.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.fields.iter()
    }

    /// Get the number of fields.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if the entity has no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// A value that can be stored in an entity field.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Int(i32),
    Int8(i64),
    BigInt(BigInt),
    BigDecimal(BigDecimal),
    Bool(bool),
    Bytes(Bytes),
    Address(Address),
    Array(Vec<Value>),
    Null,
}

impl Value {
    /// Try to get this value as a string.
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get this value as a BigInt.
    pub fn as_big_int(&self) -> Option<&BigInt> {
        match self {
            Value::BigInt(n) => Some(n),
            _ => None,
        }
    }

    /// Try to get this value as an Address.
    pub fn as_address(&self) -> Option<&Address> {
        match self {
            Value::Address(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get this value as bytes.
    pub fn as_bytes(&self) -> Option<&Bytes> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// Try to get this value as a bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

// Convenient From implementations

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.into())
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Int(n)
    }
}

impl From<BigInt> for Value {
    fn from(n: BigInt) -> Self {
        Value::BigInt(n)
    }
}

impl From<BigDecimal> for Value {
    fn from(n: BigDecimal) -> Self {
        Value::BigDecimal(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<Bytes> for Value {
    fn from(b: Bytes) -> Self {
        Value::Bytes(b)
    }
}

impl From<Address> for Value {
    fn from(a: Address) -> Self {
        Value::Address(a)
    }
}

impl From<B256> for Value {
    fn from(h: B256) -> Self {
        Value::Bytes(Bytes::from_slice(h.as_slice()))
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::Array(v.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

/// Trait for types that can be stored as entities.
///
/// Typically implemented via `#[derive(Entity)]`.
pub trait Store: Sized {
    /// The entity type name (used as the table name in the store).
    const ENTITY_TYPE: &'static str;

    /// Get the entity's unique identifier.
    fn id(&self) -> &str;

    /// Convert this instance to a store Entity.
    fn to_entity(&self) -> Entity;

    /// Create an instance from a store Entity.
    fn from_entity(entity: Entity) -> Result<Self, EntityError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EntityError {
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("field type mismatch: expected {expected} for field {field}")]
    TypeMismatch { field: String, expected: String },
}
