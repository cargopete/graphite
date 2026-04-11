//! Generated entity types from schema.graphql.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

// ── Pool ────────────────────────────────────────────────────────────────────

/// Generated from `Pool` entity in schema.graphql.
pub struct Pool {
    id: alloc::string::String,
    /// token0: Bytes! (non-nullable)
    token0: Vec<u8>,
    /// token1: Bytes! (non-nullable)
    token1: Vec<u8>,
    /// swapCount: BigInt! (non-nullable)
    swap_count: Vec<u8>,
}

impl Pool {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            token0: Default::default(),
            token1: Default::default(),
            swap_count: alloc::vec![0],
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
        use graph_as_runtime::native_store::{FieldValue, STORE};
        use std::collections::HashMap;
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
            swap_count: get("swapCount").and_then(|v| v.as_bytes()).unwrap_or(alloc::vec![0]),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Pool", id).cloned())?;
        Some(Self {
            id: id.into(),
            token0: match fields.get("token0") {
                Some(FieldValue::Bytes(b)) => b.clone(),
                _ => Default::default(),
            },
            token1: match fields.get("token1") {
                Some(FieldValue::Bytes(b)) => b.clone(),
                _ => Default::default(),
            },
            swap_count: match fields.get("swapCount") {
                Some(FieldValue::BigInt(b)) => b.clone(),
                _ => alloc::vec![0],
            },
        })
    }
}

// ── Swap ────────────────────────────────────────────────────────────────────

/// Generated from `Swap` entity in schema.graphql.
pub struct Swap {
    id: alloc::string::String,
    /// pool: String! (non-nullable)
    pool: alloc::string::String,
    /// amount0In: BigInt! (non-nullable)
    amount0_in: Vec<u8>,
    /// amount1In: BigInt! (non-nullable)
    amount1_in: Vec<u8>,
    /// amount0Out: BigInt! (non-nullable)
    amount0_out: Vec<u8>,
    /// amount1Out: BigInt! (non-nullable)
    amount1_out: Vec<u8>,
    /// blockNumber: BigInt! (non-nullable)
    block_number: Vec<u8>,
    /// timestamp: BigInt! (non-nullable)
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
        use graph_as_runtime::native_store::{FieldValue, STORE};
        use std::collections::HashMap;
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
}
