//! Generated entity types from schema.graphql.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Generated from `Pool` entity in schema.graphql.
pub struct Pool {
    id: alloc::string::String,
    token0: Vec<u8>,
    token1: Vec<u8>,
    swap_count: Vec<u8>,
}

impl Pool {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            token0: Default::default(),
            token1: Default::default(),
            swap_count: Default::default(),
        }
    }

    pub fn token0(&self) -> &Vec<u8> { &self.token0 }

    pub fn token1(&self) -> &Vec<u8> { &self.token1 }

    pub fn swap_count(&self) -> &Vec<u8> { &self.swap_count }

    pub fn set_token0(mut self, v: Vec<u8>) -> Self {
        self.token0 = v;
        self
    }

    pub fn set_token1(mut self, v: Vec<u8>) -> Self {
        self.token1 = v;
        self
    }

    pub fn set_swap_count(mut self, v: Vec<u8>) -> Self {
        self.swap_count = v;
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        b.set_bytes("token0", &self.token0);
        b.set_bytes("token1", &self.token1);
        b.set_bigint("swapCount", &self.swap_count);
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Pool");
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
        fields.insert("token0".to_string(), FieldValue::Bytes(self.token0.clone()));
        fields.insert("token1".to_string(), FieldValue::Bytes(self.token1.clone()));
        fields.insert("swapCount".to_string(), FieldValue::BigInt(self.swap_count.clone()));
        STORE.with(|s| s.borrow_mut().set_entity("Pool", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Pool");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        let map_ptr = unsafe { graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) };
        if map_ptr == 0 {
            return None;
        }
        let fields = unsafe { graph_as_runtime::store_read::read_typed_map(map_ptr) };
        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        Some(Self {
            id: id.into(),
            token0: get("token0").and_then(|v| v.as_bytes()).unwrap_or_default(),
            token1: get("token1").and_then(|v| v.as_bytes()).unwrap_or_default(),
            swap_count: get("swapCount").and_then(|v| v.as_bytes()).unwrap_or_default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Pool", id).cloned())?;
        Some(Self {
            id: id.into(),
            token0: fields.get("token0").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            token1: fields.get("token1").and_then(|v| if let FieldValue::Bytes(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            swap_count: fields.get("swapCount").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove(id: &str) {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Pool");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        unsafe {
            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(id: &str) {
        use graph_as_runtime::native_store::STORE;
        STORE.with(|s| s.borrow_mut().remove_entity("Pool", id));
    }
}

/// Generated from `Swap` entity in schema.graphql.
pub struct Swap {
    id: alloc::string::String,
    pool: alloc::string::String,
    amount0_in: Vec<u8>,
    amount1_in: Vec<u8>,
    amount0_out: Vec<u8>,
    amount1_out: Vec<u8>,
    block_number: Vec<u8>,
    timestamp: Vec<u8>,
}

impl Swap {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            pool: Default::default(),
            amount0_in: Default::default(),
            amount1_in: Default::default(),
            amount0_out: Default::default(),
            amount1_out: Default::default(),
            block_number: Default::default(),
            timestamp: Default::default(),
        }
    }

    pub fn pool(&self) -> &alloc::string::String { &self.pool }

    pub fn amount0_in(&self) -> &Vec<u8> { &self.amount0_in }

    pub fn amount1_in(&self) -> &Vec<u8> { &self.amount1_in }

    pub fn amount0_out(&self) -> &Vec<u8> { &self.amount0_out }

    pub fn amount1_out(&self) -> &Vec<u8> { &self.amount1_out }

    pub fn block_number(&self) -> &Vec<u8> { &self.block_number }

    pub fn timestamp(&self) -> &Vec<u8> { &self.timestamp }

    pub fn set_pool(mut self, v: alloc::string::String) -> Self {
        self.pool = v;
        self
    }

    pub fn set_amount0_in(mut self, v: Vec<u8>) -> Self {
        self.amount0_in = v;
        self
    }

    pub fn set_amount1_in(mut self, v: Vec<u8>) -> Self {
        self.amount1_in = v;
        self
    }

    pub fn set_amount0_out(mut self, v: Vec<u8>) -> Self {
        self.amount0_out = v;
        self
    }

    pub fn set_amount1_out(mut self, v: Vec<u8>) -> Self {
        self.amount1_out = v;
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
        b.set_string("pool", &self.pool);
        b.set_bigint("amount0In", &self.amount0_in);
        b.set_bigint("amount1In", &self.amount1_in);
        b.set_bigint("amount0Out", &self.amount0_out);
        b.set_bigint("amount1Out", &self.amount1_out);
        b.set_bigint("blockNumber", &self.block_number);
        b.set_bigint("timestamp", &self.timestamp);
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Swap");
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
        fields.insert("pool".to_string(), FieldValue::String(self.pool.clone()));
        fields.insert("amount0In".to_string(), FieldValue::BigInt(self.amount0_in.clone()));
        fields.insert("amount1In".to_string(), FieldValue::BigInt(self.amount1_in.clone()));
        fields.insert("amount0Out".to_string(), FieldValue::BigInt(self.amount0_out.clone()));
        fields.insert("amount1Out".to_string(), FieldValue::BigInt(self.amount1_out.clone()));
        fields.insert("blockNumber".to_string(), FieldValue::BigInt(self.block_number.clone()));
        fields.insert("timestamp".to_string(), FieldValue::BigInt(self.timestamp.clone()));
        STORE.with(|s| s.borrow_mut().set_entity("Swap", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Swap");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        let map_ptr = unsafe { graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) };
        if map_ptr == 0 {
            return None;
        }
        let fields = unsafe { graph_as_runtime::store_read::read_typed_map(map_ptr) };
        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        Some(Self {
            id: id.into(),
            pool: get("pool").and_then(|v| v.as_string().map(|s| s.to_string())).unwrap_or_default(),
            amount0_in: get("amount0In").and_then(|v| v.as_bytes()).unwrap_or_default(),
            amount1_in: get("amount1In").and_then(|v| v.as_bytes()).unwrap_or_default(),
            amount0_out: get("amount0Out").and_then(|v| v.as_bytes()).unwrap_or_default(),
            amount1_out: get("amount1Out").and_then(|v| v.as_bytes()).unwrap_or_default(),
            block_number: get("blockNumber").and_then(|v| v.as_bytes()).unwrap_or_default(),
            timestamp: get("timestamp").and_then(|v| v.as_bytes()).unwrap_or_default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Swap", id).cloned())?;
        Some(Self {
            id: id.into(),
            pool: fields.get("pool").and_then(|v| if let FieldValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(),
            amount0_in: fields.get("amount0In").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            amount1_in: fields.get("amount1In").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            amount0_out: fields.get("amount0Out").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            amount1_out: fields.get("amount1Out").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            block_number: fields.get("blockNumber").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
            timestamp: fields.get("timestamp").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove(id: &str) {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Swap");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        unsafe {
            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(id: &str) {
        use graph_as_runtime::native_store::STORE;
        STORE.with(|s| s.borrow_mut().remove_entity("Swap", id));
    }
}

