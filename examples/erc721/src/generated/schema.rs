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
    owner: Vec<u8>,
    approved: Option<Vec<u8>>,
}

impl Token {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            owner: Default::default(),
            approved: None,
        }
    }

    pub fn owner(&self) -> &Vec<u8> { &self.owner }

    pub fn approved(&self) -> Option<&Vec<u8>> { self.approved.as_ref() }

    pub fn set_owner(mut self, v: Vec<u8>) -> Self {
        self.owner = v;
        self
    }

    pub fn set_approved(mut self, v: Vec<u8>) -> Self {
        self.approved = Some(v);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        b.set_bytes("owner", &self.owner);
        if let Some(ref v) = self.approved { b.set_bytes("approved", v); }
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
        fields.insert("owner".to_string(), FieldValue::Bytes(self.owner.clone()));
        if let Some(ref v) = self.approved { fields.insert("approved".to_string(), FieldValue::Bytes(v.clone())); }
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
            owner: get("owner").and_then(|v| v.as_bytes()).unwrap_or_default(),
            approved: get("approved").and_then(|v| v.as_bytes()),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Token", id).cloned())?;
        Some(Self {
            id: id.into(),
            owner: fields.get("owner").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            approved: fields.get("approved").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }),
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
    from: Vec<u8>,
    to: Vec<u8>,
    token_id: Vec<u8>,
    block_number: Vec<u8>,
    timestamp: Vec<u8>,
    transaction_hash: Vec<u8>,
}

impl Transfer {
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

    pub fn from(&self) -> &Vec<u8> { &self.from }

    pub fn to(&self) -> &Vec<u8> { &self.to }

    pub fn token_id(&self) -> &Vec<u8> { &self.token_id }

    pub fn block_number(&self) -> &Vec<u8> { &self.block_number }

    pub fn timestamp(&self) -> &Vec<u8> { &self.timestamp }

    pub fn transaction_hash(&self) -> &Vec<u8> { &self.transaction_hash }

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
        fields.insert("from".to_string(), FieldValue::Bytes(self.from.clone()));
        fields.insert("to".to_string(), FieldValue::Bytes(self.to.clone()));
        fields.insert("tokenId".to_string(), FieldValue::BigInt(self.token_id.clone()));
        fields.insert("blockNumber".to_string(), FieldValue::BigInt(self.block_number.clone()));
        fields.insert("timestamp".to_string(), FieldValue::BigInt(self.timestamp.clone()));
        fields.insert("transactionHash".to_string(), FieldValue::Bytes(self.transaction_hash.clone()));
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
        let fields = STORE.with(|s| s.borrow().get_entity("Transfer", id).cloned())?;
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

/// Generated from `Approval` entity in schema.graphql.
pub struct Approval {
    id: alloc::string::String,
    owner: Vec<u8>,
    approved: Vec<u8>,
    token_id: Vec<u8>,
    block_number: Vec<u8>,
    transaction_hash: Vec<u8>,
}

impl Approval {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            owner: Default::default(),
            approved: Default::default(),
            token_id: Default::default(),
            block_number: Default::default(),
            transaction_hash: Default::default(),
        }
    }

    pub fn owner(&self) -> &Vec<u8> { &self.owner }

    pub fn approved(&self) -> &Vec<u8> { &self.approved }

    pub fn token_id(&self) -> &Vec<u8> { &self.token_id }

    pub fn block_number(&self) -> &Vec<u8> { &self.block_number }

    pub fn transaction_hash(&self) -> &Vec<u8> { &self.transaction_hash }

    pub fn set_owner(mut self, v: Vec<u8>) -> Self {
        self.owner = v;
        self
    }

    pub fn set_approved(mut self, v: Vec<u8>) -> Self {
        self.approved = v;
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

    pub fn set_transaction_hash(mut self, v: Vec<u8>) -> Self {
        self.transaction_hash = v;
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        b.set_bytes("owner", &self.owner);
        b.set_bytes("approved", &self.approved);
        b.set_bigint("tokenId", &self.token_id);
        b.set_bigint("blockNumber", &self.block_number);
        b.set_bytes("transactionHash", &self.transaction_hash);
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Approval");
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
        fields.insert("owner".to_string(), FieldValue::Bytes(self.owner.clone()));
        fields.insert("approved".to_string(), FieldValue::Bytes(self.approved.clone()));
        fields.insert("tokenId".to_string(), FieldValue::BigInt(self.token_id.clone()));
        fields.insert("blockNumber".to_string(), FieldValue::BigInt(self.block_number.clone()));
        fields.insert("transactionHash".to_string(), FieldValue::Bytes(self.transaction_hash.clone()));
        STORE.with(|s| s.borrow_mut().set_entity("Approval", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Approval");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        let map_ptr = unsafe { graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) };
        if map_ptr == 0 {
            return None;
        }
        let fields = unsafe { graph_as_runtime::store_read::read_typed_map(map_ptr) };
        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        Some(Self {
            id: id.into(),
            owner: get("owner").and_then(|v| v.as_bytes()).unwrap_or_default(),
            approved: get("approved").and_then(|v| v.as_bytes()).unwrap_or_default(),
            token_id: get("tokenId").and_then(|v| v.as_bytes()).unwrap_or_default(),
            block_number: get("blockNumber").and_then(|v| v.as_bytes()).unwrap_or_default(),
            transaction_hash: get("transactionHash").and_then(|v| v.as_bytes()).unwrap_or_default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Approval", id).cloned())?;
        Some(Self {
            id: id.into(),
            owner: fields.get("owner").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            approved: fields.get("approved").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            token_id: fields.get("tokenId").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            block_number: fields.get("blockNumber").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            transaction_hash: fields.get("transactionHash").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove(id: &str) {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Approval");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        unsafe {
            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(id: &str) {
        use graph_as_runtime::native_store::STORE;
        STORE.with(|s| s.borrow_mut().remove_entity("Approval", id));
    }
}

