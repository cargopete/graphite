//! Generated entity types from schema.graphql.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// Generated from `Erc20Transfer` entity in schema.graphql.
pub struct Erc20Transfer {
    id: alloc::string::String,
    from: Vec<u8>,
    to: Vec<u8>,
    value: Vec<u8>,
    block_number: Vec<u8>,
    timestamp: Vec<u8>,
    transaction_hash: Vec<u8>,
}

impl Erc20Transfer {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            from: Default::default(),
            to: Default::default(),
            value: Default::default(),
            block_number: Default::default(),
            timestamp: Default::default(),
            transaction_hash: Default::default(),
        }
    }

    pub fn set_from(mut self, v: Vec<u8>) -> Self {
        self.from = v;
        self
    }

    pub fn set_to(mut self, v: Vec<u8>) -> Self {
        self.to = v;
        self
    }

    pub fn set_value(mut self, v: Vec<u8>) -> Self {
        self.value = v;
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

    pub fn set_transaction_hash(mut self, v: Vec<u8>) -> Self {
        self.transaction_hash = v;
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        b.set_bytes("from", &self.from);
        b.set_bytes("to", &self.to);
        b.set_bigint("value", &self.value);
        b.set_bigint("blockNumber", &self.block_number);
        b.set_bigint("timestamp", &self.timestamp);
        b.set_bytes("transactionHash", &self.transaction_hash);
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Erc20Transfer");
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
        fields.insert("from".to_string(), FieldValue::Bytes(self.from.clone()));
        fields.insert("to".to_string(), FieldValue::Bytes(self.to.clone()));
        fields.insert("value".to_string(), FieldValue::BigInt(self.value.clone()));
        fields.insert("blockNumber".to_string(), FieldValue::BigInt(self.block_number.clone()));
        fields.insert("timestamp".to_string(), FieldValue::BigInt(self.timestamp.clone()));
        fields.insert("transactionHash".to_string(), FieldValue::Bytes(self.transaction_hash.clone()));
        STORE.with(|s| s.borrow_mut().set_entity("Erc20Transfer", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Erc20Transfer");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        let map_ptr = unsafe { graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) };
        if map_ptr == 0 {
            return None;
        }
        let fields = unsafe { graph_as_runtime::store_read::read_typed_map(map_ptr) };
        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        Some(Self {
            id: id.into(),
            from: get("from").and_then(|v| v.as_bytes()).unwrap_or_default(),
            to: get("to").and_then(|v| v.as_bytes()).unwrap_or_default(),
            value: get("value").and_then(|v| v.as_bytes()).unwrap_or_default(),
            block_number: get("blockNumber").and_then(|v| v.as_bytes()).unwrap_or_default(),
            timestamp: get("timestamp").and_then(|v| v.as_bytes()).unwrap_or_default(),
            transaction_hash: get("transactionHash").and_then(|v| v.as_bytes()).unwrap_or_default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Erc20Transfer", id).cloned())?;
        Some(Self {
            id: id.into(),
            from: fields.get("from").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            to: fields.get("to").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            value: fields.get("value").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            block_number: fields.get("blockNumber").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            timestamp: fields.get("timestamp").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            transaction_hash: fields.get("transactionHash").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove(id: &str) {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Erc20Transfer");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        unsafe {
            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(id: &str) {
        use graph_as_runtime::native_store::STORE;
        STORE.with(|s| s.borrow_mut().remove_entity("Erc20Transfer", id));
    }
}

/// Generated from `Erc721Transfer` entity in schema.graphql.
pub struct Erc721Transfer {
    id: alloc::string::String,
    from: Vec<u8>,
    to: Vec<u8>,
    token_id: Vec<u8>,
    block_number: Vec<u8>,
    timestamp: Vec<u8>,
    transaction_hash: Vec<u8>,
}

impl Erc721Transfer {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            from: Default::default(),
            to: Default::default(),
            token_id: Default::default(),
            block_number: Default::default(),
            timestamp: Default::default(),
            transaction_hash: Default::default(),
        }
    }

    pub fn set_from(mut self, v: Vec<u8>) -> Self {
        self.from = v;
        self
    }

    pub fn set_to(mut self, v: Vec<u8>) -> Self {
        self.to = v;
        self
    }

    pub fn set_token_id(mut self, v: Vec<u8>) -> Self {
        self.token_id = v;
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

    pub fn set_transaction_hash(mut self, v: Vec<u8>) -> Self {
        self.transaction_hash = v;
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        b.set_bytes("from", &self.from);
        b.set_bytes("to", &self.to);
        b.set_bigint("tokenId", &self.token_id);
        b.set_bigint("blockNumber", &self.block_number);
        b.set_bigint("timestamp", &self.timestamp);
        b.set_bytes("transactionHash", &self.transaction_hash);
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Erc721Transfer");
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
        fields.insert("from".to_string(), FieldValue::Bytes(self.from.clone()));
        fields.insert("to".to_string(), FieldValue::Bytes(self.to.clone()));
        fields.insert("tokenId".to_string(), FieldValue::BigInt(self.token_id.clone()));
        fields.insert("blockNumber".to_string(), FieldValue::BigInt(self.block_number.clone()));
        fields.insert("timestamp".to_string(), FieldValue::BigInt(self.timestamp.clone()));
        fields.insert("transactionHash".to_string(), FieldValue::Bytes(self.transaction_hash.clone()));
        STORE.with(|s| s.borrow_mut().set_entity("Erc721Transfer", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Erc721Transfer");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        let map_ptr = unsafe { graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) };
        if map_ptr == 0 {
            return None;
        }
        let fields = unsafe { graph_as_runtime::store_read::read_typed_map(map_ptr) };
        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        Some(Self {
            id: id.into(),
            from: get("from").and_then(|v| v.as_bytes()).unwrap_or_default(),
            to: get("to").and_then(|v| v.as_bytes()).unwrap_or_default(),
            token_id: get("tokenId").and_then(|v| v.as_bytes()).unwrap_or_default(),
            block_number: get("blockNumber").and_then(|v| v.as_bytes()).unwrap_or_default(),
            timestamp: get("timestamp").and_then(|v| v.as_bytes()).unwrap_or_default(),
            transaction_hash: get("transactionHash").and_then(|v| v.as_bytes()).unwrap_or_default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Erc721Transfer", id).cloned())?;
        Some(Self {
            id: id.into(),
            from: fields.get("from").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            to: fields.get("to").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            token_id: fields.get("tokenId").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            block_number: fields.get("blockNumber").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            timestamp: fields.get("timestamp").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            transaction_hash: fields.get("transactionHash").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove(id: &str) {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Erc721Transfer");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        unsafe {
            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(id: &str) {
        use graph_as_runtime::native_store::STORE;
        STORE.with(|s| s.borrow_mut().remove_entity("Erc721Transfer", id));
    }
}

