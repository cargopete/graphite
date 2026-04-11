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
    symbol: alloc::string::String,
    name: alloc::string::String,
    decimals: i32,
    total_supply: Vec<u8>,
}

impl Token {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            symbol: Default::default(),
            name: Default::default(),
            decimals: Default::default(),
            total_supply: Default::default(),
        }
    }

    pub fn symbol(&self) -> &alloc::string::String { &self.symbol }

    pub fn name(&self) -> &alloc::string::String { &self.name }

    pub fn decimals(&self) -> &i32 { &self.decimals }

    pub fn total_supply(&self) -> &Vec<u8> { &self.total_supply }

    pub fn set_symbol(mut self, v: alloc::string::String) -> Self {
        self.symbol = v;
        self
    }

    pub fn set_name(mut self, v: alloc::string::String) -> Self {
        self.name = v;
        self
    }

    pub fn set_decimals(mut self, v: i32) -> Self {
        self.decimals = v;
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
        b.set_string("symbol", &self.symbol);
        b.set_string("name", &self.name);
        b.set_i32("decimals", self.decimals);
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
        fields.insert("symbol".to_string(), FieldValue::String(self.symbol.clone()));
        fields.insert("name".to_string(), FieldValue::String(self.name.clone()));
        fields.insert("decimals".to_string(), FieldValue::Int(self.decimals));
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
            symbol: get("symbol").and_then(|v| v.as_string().map(|s| s.to_string())).unwrap_or_default(),
            name: get("name").and_then(|v| v.as_string().map(|s| s.to_string())).unwrap_or_default(),
            decimals: get("decimals").and_then(|v| v.as_i32()).unwrap_or_default(),
            total_supply: get("totalSupply").and_then(|v| v.as_bytes()).unwrap_or_default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Token", id).cloned())?;
        Some(Self {
            id: id.into(),
            symbol: fields.get("symbol").and_then(|v| if let FieldValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(),
            name: fields.get("name").and_then(|v| if let FieldValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(),
            decimals: fields.get("decimals").and_then(|v| if let FieldValue::Int(n) = v { Some(*n) } else { None }).unwrap_or_default(),
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

/// Generated from `Account` entity in schema.graphql.
pub struct Account {
    id: alloc::string::String,
    balances: Vec<alloc::string::String>,
}

impl Account {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            balances: Default::default(),
        }
    }

    pub fn balances(&self) -> &Vec<alloc::string::String> { &self.balances }

    pub fn set_balances(mut self, v: Vec<alloc::string::String>) -> Self {
        self.balances = v;
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
        use std::collections::HashMap;
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldValue::String(self.id.clone()));
        // skipped list field `balances` (not directly storable)
        STORE.with(|s| s.borrow_mut().set_entity("Account", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Account");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        let map_ptr = unsafe { graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) };
        if map_ptr == 0 {
            return None;
        }
        let fields = unsafe { graph_as_runtime::store_read::read_typed_map(map_ptr) };
        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        Some(Self {
            id: id.into(),
            balances: Default::default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("Account", id).cloned())?;
        Some(Self {
            id: id.into(),
            balances: Default::default(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove(id: &str) {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("Account");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        unsafe {
            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(id: &str) {
        use graph_as_runtime::native_store::STORE;
        STORE.with(|s| s.borrow_mut().remove_entity("Account", id));
    }
}

/// Generated from `TokenBalance` entity in schema.graphql.
pub struct TokenBalance {
    id: alloc::string::String,
    account: alloc::string::String,
    token: alloc::string::String,
    balance: Vec<u8>,
}

impl TokenBalance {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            account: Default::default(),
            token: Default::default(),
            balance: Default::default(),
        }
    }

    pub fn account(&self) -> &alloc::string::String { &self.account }

    pub fn token(&self) -> &alloc::string::String { &self.token }

    pub fn balance(&self) -> &Vec<u8> { &self.balance }

    pub fn set_account(mut self, v: alloc::string::String) -> Self {
        self.account = v;
        self
    }

    pub fn set_token(mut self, v: alloc::string::String) -> Self {
        self.token = v;
        self
    }

    pub fn set_balance(mut self, v: Vec<u8>) -> Self {
        self.balance = v;
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        b.set_string("account", &self.account);
        b.set_string("token", &self.token);
        b.set_bigint("balance", &self.balance);
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("TokenBalance");
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
        fields.insert("account".to_string(), FieldValue::String(self.account.clone()));
        fields.insert("token".to_string(), FieldValue::String(self.token.clone()));
        fields.insert("balance".to_string(), FieldValue::BigInt(self.balance.clone()));
        STORE.with(|s| s.borrow_mut().set_entity("TokenBalance", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("TokenBalance");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        let map_ptr = unsafe { graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) };
        if map_ptr == 0 {
            return None;
        }
        let fields = unsafe { graph_as_runtime::store_read::read_typed_map(map_ptr) };
        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
        Some(Self {
            id: id.into(),
            account: get("account").and_then(|v| v.as_string().map(|s| s.to_string())).unwrap_or_default(),
            token: get("token").and_then(|v| v.as_string().map(|s| s.to_string())).unwrap_or_default(),
            balance: get("balance").and_then(|v| v.as_bytes()).unwrap_or_default(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("TokenBalance", id).cloned())?;
        Some(Self {
            id: id.into(),
            account: fields.get("account").and_then(|v| if let FieldValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(),
            token: fields.get("token").and_then(|v| if let FieldValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(),
            balance: fields.get("balance").and_then(|v| if let FieldValue::BigInt(b) = v { Some(b.clone()) } else { None }).unwrap_or_default(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove(id: &str) {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("TokenBalance");
        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);
        unsafe {
            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(id: &str) {
        use graph_as_runtime::native_store::STORE;
        STORE.with(|s| s.borrow_mut().remove_entity("TokenBalance", id));
    }
}

/// Generated from `Transfer` entity in schema.graphql.
pub struct Transfer {
    id: alloc::string::String,
    token: alloc::string::String,
    from: Vec<u8>,
    to: Vec<u8>,
    value: Vec<u8>,
    block_number: Vec<u8>,
    timestamp: Vec<u8>,
    transaction_hash: Vec<u8>,
}

impl Transfer {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            token: Default::default(),
            from: Default::default(),
            to: Default::default(),
            value: Default::default(),
            block_number: Default::default(),
            timestamp: Default::default(),
            transaction_hash: Default::default(),
        }
    }

    pub fn token(&self) -> &alloc::string::String { &self.token }

    pub fn from(&self) -> &Vec<u8> { &self.from }

    pub fn to(&self) -> &Vec<u8> { &self.to }

    pub fn value(&self) -> &Vec<u8> { &self.value }

    pub fn block_number(&self) -> &Vec<u8> { &self.block_number }

    pub fn timestamp(&self) -> &Vec<u8> { &self.timestamp }

    pub fn transaction_hash(&self) -> &Vec<u8> { &self.transaction_hash }

    pub fn set_token(mut self, v: alloc::string::String) -> Self {
        self.token = v;
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
        b.set_string("token", &self.token);
        b.set_bytes("from", &self.from);
        b.set_bytes("to", &self.to);
        b.set_bigint("value", &self.value);
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
        fields.insert("token".to_string(), FieldValue::String(self.token.clone()));
        fields.insert("from".to_string(), FieldValue::Bytes(self.from.clone()));
        fields.insert("to".to_string(), FieldValue::Bytes(self.to.clone()));
        fields.insert("value".to_string(), FieldValue::BigInt(self.value.clone()));
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
            token: get("token").and_then(|v| v.as_string().map(|s| s.to_string())).unwrap_or_default(),
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
        let fields = STORE.with(|s| s.borrow().get_entity("Transfer", id).cloned())?;
        Some(Self {
            id: id.into(),
            token: fields.get("token").and_then(|v| if let FieldValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(),
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

