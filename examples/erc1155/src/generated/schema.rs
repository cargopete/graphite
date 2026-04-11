//! Generated entity types from schema.graphql.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Generated from `Token` entity in schema.graphql.
pub struct Token {
    id: alloc::string::String,
    uri: Option<alloc::string::String>,
    total_supply: Vec<u8>,
}

impl Token {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            uri: None,
            total_supply: Default::default(),
        }
    }

    pub fn uri(&self) -> Option<&alloc::string::String> { self.uri.as_ref() }

    pub fn total_supply(&self) -> &Vec<u8> { &self.total_supply }

    pub fn set_uri(mut self, v: alloc::string::String) -> Self {
        self.uri = Some(v);
        self
    }

    pub fn set_total_supply(mut self, v: Vec<u8>) -> Self {
        self.total_supply = v;
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        if let Some(ref v) = self.uri { b.set_string("uri", v); }
        b.set_bigint("totalSupply", &self.total_supply);
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Token");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);
        unsafe {
            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        use std::collections::HashMap;
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        if let Some(ref v) = self.uri { fields.insert("uri".to_string(), FieldValue::String(v.clone())); }
        fields.insert("totalSupply".to_string(), FieldValue::BigInt(self.total_supply.clone()));
        STORE.with(|s| s.borrow_mut().set_entity("Token", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Token");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        let map_ptr = unsafe { graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) };
        if map_ptr == 0 {
            return None;
        }
        let fields = unsafe { graph_as_runtime::store_read::read_typed_map(map_ptr) };
        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        Some(Self {
            id: id.into(),
            uri: get("uri").and_then(|v| v.as_string().map(|s| s.to_string())),
            total_supply: get("totalSupply").and_then(|v| v.as_bytes()).unwrap_or_default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Token", id).cloned())?;
        Some(Self {
            id: id.into(),
            uri: fields.get("uri").and_then(|v| if let FieldValue::String(s) = v { Some(s.clone()) } else { None }),
            total_supply: fields.get("totalSupply").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove(id: &str) {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Token");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        unsafe {
            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(id: &str) {
        use graph_as_runtime::native_store::STORE;
        STORE.with(|s| s.borrow_mut().remove_entity("Token", id));
    }
}

/// Generated from `Transfer` entity in schema.graphql.
pub struct Transfer {
    id: alloc::string::String,
    operator: Vec<u8>,
    from: Vec<u8>,
    to: Vec<u8>,
    token: alloc::string::String,
    amount: Vec<u8>,
    block_number: Vec<u8>,
    timestamp: Vec<u8>,
}

impl Transfer {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            operator: Default::default(),
            from: Default::default(),
            to: Default::default(),
            token: Default::default(),
            amount: Default::default(),
            block_number: Default::default(),
            timestamp: Default::default(),
        }
    }

    pub fn operator(&self) -> &Vec<u8> { &self.operator }

    pub fn from(&self) -> &Vec<u8> { &self.from }

    pub fn to(&self) -> &Vec<u8> { &self.to }

    pub fn token(&self) -> &alloc::string::String { &self.token }

    pub fn amount(&self) -> &Vec<u8> { &self.amount }

    pub fn block_number(&self) -> &Vec<u8> { &self.block_number }

    pub fn timestamp(&self) -> &Vec<u8> { &self.timestamp }

    pub fn set_operator(mut self, v: Vec<u8>) -> Self {
        self.operator = v;
        self
    }

    pub fn set_from(mut self, v: Vec<u8>) -> Self {
        self.from = v;
        self
    }

    pub fn set_to(mut self, v: Vec<u8>) -> Self {
        self.to = v;
        self
    }

    pub fn set_token(mut self, v: alloc::string::String) -> Self {
        self.token = v;
        self
    }

    pub fn set_amount(mut self, v: Vec<u8>) -> Self {
        self.amount = v;
        self
    }

    pub fn set_block_number(mut self, v: Vec<u8>) -> Self {
        self.block_number = v;
        self
    }

    pub fn set_timestamp(mut self, v: Vec<u8>) -> Self {
        self.timestamp = v;
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        b.set_bytes("operator", &self.operator);
        b.set_bytes("from", &self.from);
        b.set_bytes("to", &self.to);
        b.set_string("token", &self.token);
        b.set_bigint("amount", &self.amount);
        b.set_bigint("blockNumber", &self.block_number);
        b.set_bigint("timestamp", &self.timestamp);
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Transfer");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);
        unsafe {
            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        use std::collections::HashMap;
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        fields.insert("operator".to_string(), FieldValue::Bytes(self.operator.clone()));
        fields.insert("from".to_string(), FieldValue::Bytes(self.from.clone()));
        fields.insert("to".to_string(), FieldValue::Bytes(self.to.clone()));
        fields.insert("token".to_string(), FieldValue::String(self.token.clone()));
        fields.insert("amount".to_string(), FieldValue::BigInt(self.amount.clone()));
        fields.insert("blockNumber".to_string(), FieldValue::BigInt(self.block_number.clone()));
        fields.insert("timestamp".to_string(), FieldValue::BigInt(self.timestamp.clone()));
        STORE.with(|s| s.borrow_mut().set_entity("Transfer", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Transfer");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        let map_ptr = unsafe { graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) };
        if map_ptr == 0 {
            return None;
        }
        let fields = unsafe { graph_as_runtime::store_read::read_typed_map(map_ptr) };
        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        Some(Self {
            id: id.into(),
            operator: get("operator").and_then(|v| v.as_bytes()).unwrap_or_default(),
            from: get("from").and_then(|v| v.as_bytes()).unwrap_or_default(),
            to: get("to").and_then(|v| v.as_bytes()).unwrap_or_default(),
            token: get("token").and_then(|v| v.as_string().map(|s| s.to_string())).unwrap_or_default(),
            amount: get("amount").and_then(|v| v.as_bytes()).unwrap_or_default(),
            block_number: get("blockNumber").and_then(|v| v.as_bytes()).unwrap_or_default(),
            timestamp: get("timestamp").and_then(|v| v.as_bytes()).unwrap_or_default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Transfer", id).cloned())?;
        Some(Self {
            id: id.into(),
            operator: fields.get("operator").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            from: fields.get("from").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            to: fields.get("to").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            token: fields.get("token").and_then(|v| if let FieldValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(),
            amount: fields.get("amount").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            block_number: fields.get("blockNumber").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            timestamp: fields.get("timestamp").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove(id: &str) {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Transfer");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        unsafe {
            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(id: &str) {
        use graph_as_runtime::native_store::STORE;
        STORE.with(|s| s.borrow_mut().remove_entity("Transfer", id));
    }
}

