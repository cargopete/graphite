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
    symbol: Option<alloc::string::String>,
    name: Option<alloc::string::String>,
    decimals: Option<i32>,
    total_supply: Option<Vec<u8>>,
}

impl Token {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            symbol: None,
            name: None,
            decimals: None,
            total_supply: None,
        }
    }

    pub fn set_symbol(mut self, v: alloc::string::String) -> Self {
        self.symbol = Some(v);
        self
    }

    pub fn set_name(mut self, v: alloc::string::String) -> Self {
        self.name = Some(v);
        self
    }

    pub fn set_decimals(mut self, v: i32) -> Self {
        self.decimals = Some(v);
        self
    }

    pub fn set_total_supply(mut self, v: Vec<u8>) -> Self {
        self.total_supply = Some(v);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        if let Some(ref v) = self.symbol {
            b.set_string("symbol", v);
        }
        if let Some(ref v) = self.name {
            b.set_string("name", v);
        }
        if let Some(v) = self.decimals {
            b.set_i32("decimals", v);
        }
        if let Some(ref v) = self.total_supply {
            b.set_bigint("totalSupply", v);
        }
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Token");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);
        unsafe {
            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        use graph_as_runtime::native_store::{FieldValue, with_store_mut};
        let mut fields = std::collections::HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        if let Some(ref v) = self.symbol {
            fields.insert("symbol".to_string(), FieldValue::String(v.clone()));
        }
        if let Some(ref v) = self.name {
            fields.insert("name".to_string(), FieldValue::String(v.clone()));
        }
        if let Some(v) = self.decimals {
            fields.insert("decimals".to_string(), FieldValue::Int(v));
        }
        if let Some(ref v) = self.total_supply {
            fields.insert("totalSupply".to_string(), FieldValue::BigInt(v.clone()));
        }
        with_store_mut(|s| s.set_entity("Token", &self.id, fields));
    }
}

/// Generated from `Account` entity in schema.graphql.
pub struct Account {
    id: alloc::string::String,
    balances: Option<Vec<alloc::string::String>>,
}

impl Account {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            balances: None,
        }
    }

    pub fn set_balances(mut self, v: Vec<alloc::string::String>) -> Self {
        self.balances = Some(v);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        // skipped list field `balances` (not directly storable)
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Account");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);
        unsafe {
            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        use graph_as_runtime::native_store::{FieldValue, with_store_mut};
        let mut fields = std::collections::HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        with_store_mut(|s| s.set_entity("Account", &self.id, fields));
    }
}

/// Generated from `TokenBalance` entity in schema.graphql.
pub struct TokenBalance {
    id: alloc::string::String,
    account: Option<alloc::string::String>,
    token: Option<alloc::string::String>,
    balance: Option<Vec<u8>>,
}

impl TokenBalance {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            account: None,
            token: None,
            balance: None,
        }
    }

    pub fn set_account(mut self, v: alloc::string::String) -> Self {
        self.account = Some(v);
        self
    }

    pub fn set_token(mut self, v: alloc::string::String) -> Self {
        self.token = Some(v);
        self
    }

    pub fn set_balance(mut self, v: Vec<u8>) -> Self {
        self.balance = Some(v);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        if let Some(ref v) = self.account {
            b.set_string("account", v);
        }
        if let Some(ref v) = self.token {
            b.set_string("token", v);
        }
        if let Some(ref v) = self.balance {
            b.set_bigint("balance", v);
        }
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("TokenBalance");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);
        unsafe {
            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        use graph_as_runtime::native_store::{FieldValue, with_store_mut};
        let mut fields = std::collections::HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        if let Some(ref v) = self.account {
            fields.insert("account".to_string(), FieldValue::String(v.clone()));
        }
        if let Some(ref v) = self.token {
            fields.insert("token".to_string(), FieldValue::String(v.clone()));
        }
        if let Some(ref v) = self.balance {
            fields.insert("balance".to_string(), FieldValue::BigInt(v.clone()));
        }
        with_store_mut(|s| s.set_entity("TokenBalance", &self.id, fields));
    }
}

/// Generated from `Transfer` entity in schema.graphql.
pub struct Transfer {
    id: alloc::string::String,
    token: Option<alloc::string::String>,
    from: Option<Vec<u8>>,
    to: Option<Vec<u8>>,
    value: Option<Vec<u8>>,
    block_number: Option<Vec<u8>>,
    timestamp: Option<Vec<u8>>,
    transaction_hash: Option<Vec<u8>>,
}

impl Transfer {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            token: None,
            from: None,
            to: None,
            value: None,
            block_number: None,
            timestamp: None,
            transaction_hash: None,
        }
    }

    pub fn set_token(mut self, v: alloc::string::String) -> Self {
        self.token = Some(v);
        self
    }

    pub fn set_from(mut self, v: Vec<u8>) -> Self {
        self.from = Some(v);
        self
    }

    pub fn set_to(mut self, v: Vec<u8>) -> Self {
        self.to = Some(v);
        self
    }

    pub fn set_value(mut self, v: Vec<u8>) -> Self {
        self.value = Some(v);
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
        if let Some(ref v) = self.token {
            b.set_string("token", v);
        }
        if let Some(ref v) = self.from {
            b.set_bytes("from", v);
        }
        if let Some(ref v) = self.to {
            b.set_bytes("to", v);
        }
        if let Some(ref v) = self.value {
            b.set_bigint("value", v);
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
        use graph_as_runtime::native_store::{FieldValue, with_store_mut};
        let mut fields = std::collections::HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        if let Some(ref v) = self.token {
            fields.insert("token".to_string(), FieldValue::String(v.clone()));
        }
        if let Some(ref v) = self.from {
            fields.insert("from".to_string(), FieldValue::Bytes(v.clone()));
        }
        if let Some(ref v) = self.to {
            fields.insert("to".to_string(), FieldValue::Bytes(v.clone()));
        }
        if let Some(ref v) = self.value {
            fields.insert("value".to_string(), FieldValue::BigInt(v.clone()));
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
        with_store_mut(|s| s.set_entity("Transfer", &self.id, fields));
    }
}
