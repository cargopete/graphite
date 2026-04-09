//! Generated entity types from schema.graphql.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// Generated from `Token` entity in schema.graphql.
pub struct Token {
    id: alloc::string::String,
    owner: Option<Vec<u8>>,
    approved: Option<Vec<u8>>,
}

impl Token {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            owner: None,
            approved: None,
        }
    }

    pub fn set_owner(mut self, v: Vec<u8>) -> Self {
        self.owner = Some(v);
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
        if let Some(ref v) = self.owner {
            b.set_bytes("owner", v);
        }
        if let Some(ref v) = self.approved {
            b.set_bytes("approved", v);
        }
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Token");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);
        unsafe {
            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        use std::collections::HashMap;
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        if let Some(ref v) = self.owner {
            fields.insert("owner".to_string(), FieldValue::Bytes(v.clone()));
        }
        if let Some(ref v) = self.approved {
            fields.insert("approved".to_string(), FieldValue::Bytes(v.clone()));
        }
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
        let get = |k: &str| {
            fields
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.clone())
        };
        Some(Self {
            id: id.into(),
            owner: get("owner").and_then(|v| v.as_bytes()),
            approved: get("approved").and_then(|v| v.as_bytes()),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Token", id).cloned())?;
        Some(Self {
            id: id.into(),
            owner: fields.get("owner").and_then(|v| {
                if let FieldValue::Bytes(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            approved: fields.get("approved").and_then(|v| {
                if let FieldValue::Bytes(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
        })
    }
}

/// Generated from `Transfer` entity in schema.graphql.
pub struct Transfer {
    id: alloc::string::String,
    from: Option<Vec<u8>>,
    to: Option<Vec<u8>>,
    token_id: Option<Vec<u8>>,
    block_number: Option<Vec<u8>>,
    timestamp: Option<Vec<u8>>,
    transaction_hash: Option<Vec<u8>>,
}

impl Transfer {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            from: None,
            to: None,
            token_id: None,
            block_number: None,
            timestamp: None,
            transaction_hash: None,
        }
    }

    pub fn set_from(mut self, v: Vec<u8>) -> Self {
        self.from = Some(v);
        self
    }

    pub fn set_to(mut self, v: Vec<u8>) -> Self {
        self.to = Some(v);
        self
    }

    pub fn set_token_id(mut self, v: Vec<u8>) -> Self {
        self.token_id = Some(v);
        self
    }

    pub fn set_block_number(mut self, v: Vec<u8>) -> Self {
        self.block_number = Some(v);
        self
    }

    pub fn set_timestamp(mut self, v: Vec<u8>) -> Self {
        self.timestamp = Some(v);
        self
    }

    pub fn set_transaction_hash(mut self, v: Vec<u8>) -> Self {
        self.transaction_hash = Some(v);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        if let Some(ref v) = self.from {
            b.set_bytes("from", v);
        }
        if let Some(ref v) = self.to {
            b.set_bytes("to", v);
        }
        if let Some(ref v) = self.token_id {
            b.set_bigint("tokenId", v);
        }
        if let Some(ref v) = self.block_number {
            b.set_bigint("blockNumber", v);
        }
        if let Some(ref v) = self.timestamp {
            b.set_bigint("timestamp", v);
        }
        if let Some(ref v) = self.transaction_hash {
            b.set_bytes("transactionHash", v);
        }
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Transfer");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);
        unsafe {
            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        use std::collections::HashMap;
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        if let Some(ref v) = self.from {
            fields.insert("from".to_string(), FieldValue::Bytes(v.clone()));
        }
        if let Some(ref v) = self.to {
            fields.insert("to".to_string(), FieldValue::Bytes(v.clone()));
        }
        if let Some(ref v) = self.token_id {
            fields.insert("tokenId".to_string(), FieldValue::BigInt(v.clone()));
        }
        if let Some(ref v) = self.block_number {
            fields.insert("blockNumber".to_string(), FieldValue::BigInt(v.clone()));
        }
        if let Some(ref v) = self.timestamp {
            fields.insert("timestamp".to_string(), FieldValue::BigInt(v.clone()));
        }
        if let Some(ref v) = self.transaction_hash {
            fields.insert("transactionHash".to_string(), FieldValue::Bytes(v.clone()));
        }
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
        let get = |k: &str| {
            fields
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.clone())
        };
        Some(Self {
            id: id.into(),
            from: get("from").and_then(|v| v.as_bytes()),
            to: get("to").and_then(|v| v.as_bytes()),
            token_id: get("tokenId").and_then(|v| v.as_bytes()),
            block_number: get("blockNumber").and_then(|v| v.as_bytes()),
            timestamp: get("timestamp").and_then(|v| v.as_bytes()),
            transaction_hash: get("transactionHash").and_then(|v| v.as_bytes()),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Transfer", id).cloned())?;
        Some(Self {
            id: id.into(),
            from: fields.get("from").and_then(|v| {
                if let FieldValue::Bytes(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            to: fields.get("to").and_then(|v| {
                if let FieldValue::Bytes(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            token_id: fields.get("tokenId").and_then(|v| {
                if let FieldValue::BigInt(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            block_number: fields.get("blockNumber").and_then(|v| {
                if let FieldValue::BigInt(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            timestamp: fields.get("timestamp").and_then(|v| {
                if let FieldValue::BigInt(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            transaction_hash: fields.get("transactionHash").and_then(|v| {
                if let FieldValue::Bytes(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
        })
    }
}

/// Generated from `Approval` entity in schema.graphql.
pub struct Approval {
    id: alloc::string::String,
    owner: Option<Vec<u8>>,
    approved: Option<Vec<u8>>,
    token_id: Option<Vec<u8>>,
    block_number: Option<Vec<u8>>,
    transaction_hash: Option<Vec<u8>>,
}

impl Approval {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            owner: None,
            approved: None,
            token_id: None,
            block_number: None,
            transaction_hash: None,
        }
    }

    pub fn set_owner(mut self, v: Vec<u8>) -> Self {
        self.owner = Some(v);
        self
    }

    pub fn set_approved(mut self, v: Vec<u8>) -> Self {
        self.approved = Some(v);
        self
    }

    pub fn set_token_id(mut self, v: Vec<u8>) -> Self {
        self.token_id = Some(v);
        self
    }

    pub fn set_block_number(mut self, v: Vec<u8>) -> Self {
        self.block_number = Some(v);
        self
    }

    pub fn set_transaction_hash(mut self, v: Vec<u8>) -> Self {
        self.transaction_hash = Some(v);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        if let Some(ref v) = self.owner {
            b.set_bytes("owner", v);
        }
        if let Some(ref v) = self.approved {
            b.set_bytes("approved", v);
        }
        if let Some(ref v) = self.token_id {
            b.set_bigint("tokenId", v);
        }
        if let Some(ref v) = self.block_number {
            b.set_bigint("blockNumber", v);
        }
        if let Some(ref v) = self.transaction_hash {
            b.set_bytes("transactionHash", v);
        }
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Approval");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);
        unsafe {
            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        use std::collections::HashMap;
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        if let Some(ref v) = self.owner {
            fields.insert("owner".to_string(), FieldValue::Bytes(v.clone()));
        }
        if let Some(ref v) = self.approved {
            fields.insert("approved".to_string(), FieldValue::Bytes(v.clone()));
        }
        if let Some(ref v) = self.token_id {
            fields.insert("tokenId".to_string(), FieldValue::BigInt(v.clone()));
        }
        if let Some(ref v) = self.block_number {
            fields.insert("blockNumber".to_string(), FieldValue::BigInt(v.clone()));
        }
        if let Some(ref v) = self.transaction_hash {
            fields.insert("transactionHash".to_string(), FieldValue::Bytes(v.clone()));
        }
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
        let get = |k: &str| {
            fields
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.clone())
        };
        Some(Self {
            id: id.into(),
            owner: get("owner").and_then(|v| v.as_bytes()),
            approved: get("approved").and_then(|v| v.as_bytes()),
            token_id: get("tokenId").and_then(|v| v.as_bytes()),
            block_number: get("blockNumber").and_then(|v| v.as_bytes()),
            transaction_hash: get("transactionHash").and_then(|v| v.as_bytes()),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Approval", id).cloned())?;
        Some(Self {
            id: id.into(),
            owner: fields.get("owner").and_then(|v| {
                if let FieldValue::Bytes(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            approved: fields.get("approved").and_then(|v| {
                if let FieldValue::Bytes(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            token_id: fields.get("tokenId").and_then(|v| {
                if let FieldValue::BigInt(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            block_number: fields.get("blockNumber").and_then(|v| {
                if let FieldValue::BigInt(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
            transaction_hash: fields.get("transactionHash").and_then(|v| {
                if let FieldValue::Bytes(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            }),
        })
    }
}
