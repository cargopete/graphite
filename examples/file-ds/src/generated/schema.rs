//! Generated entity types from schema.graphql.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Generated from `NFT` entity in schema.graphql.
pub struct NFT {
    id: alloc::string::String,
    /// owner: Bytes! (non-nullable)
    owner: Vec<u8>,
    /// tokenURI: String! (non-nullable)
    token_uri: alloc::string::String,
    /// name: String (nullable)
    name: Option<alloc::string::String>,
    /// description: String (nullable)
    description: Option<alloc::string::String>,
    /// imageURI: String (nullable)
    image_uri: Option<alloc::string::String>,
}

impl NFT {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            owner: Default::default(),
            token_uri: Default::default(),
            name: None,
            description: None,
            image_uri: None,
        }
    }

    pub fn owner(&self) -> &Vec<u8> { &self.owner }
    pub fn token_uri(&self) -> &alloc::string::String { &self.token_uri }
    pub fn name(&self) -> Option<&alloc::string::String> { self.name.as_ref() }
    pub fn description(&self) -> Option<&alloc::string::String> { self.description.as_ref() }
    pub fn image_uri(&self) -> Option<&alloc::string::String> { self.image_uri.as_ref() }

    pub fn set_owner(mut self, v: Vec<u8>) -> Self {
        self.owner = v;
        self
    }

    pub fn set_token_uri(mut self, v: alloc::string::String) -> Self {
        self.token_uri = v;
        self
    }

    pub fn set_name(mut self, v: alloc::string::String) -> Self {
        self.name = Some(v);
        self
    }

    pub fn set_description(mut self, v: alloc::string::String) -> Self {
        self.description = Some(v);
        self
    }

    pub fn set_image_uri(mut self, v: alloc::string::String) -> Self {
        self.image_uri = Some(v);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let mut b = graph_as_runtime::entity::EntityBuilder::new();
        b.set_string("id", &self.id);
        b.set_bytes("owner", &self.owner);
        b.set_string("tokenURI", &self.token_uri);
        if let Some(ref v) = self.name { b.set_string("name", v); }
        if let Some(ref v) = self.description { b.set_string("description", v); }
        if let Some(ref v) = self.image_uri { b.set_string("imageURI", v); }
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("NFT");
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
        fields.insert("tokenURI".to_string(), FieldValue::String(self.token_uri.clone()));
        if let Some(ref v) = self.name {
            fields.insert("name".to_string(), FieldValue::String(v.clone()));
        }
        if let Some(ref v) = self.description {
            fields.insert("description".to_string(), FieldValue::String(v.clone()));
        }
        if let Some(ref v) = self.image_uri {
            fields.insert("imageURI".to_string(), FieldValue::String(v.clone()));
        }
        STORE.with(|s| s.borrow_mut().set_entity("NFT", &self.id, fields));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(id: &str) -> Option<Self> {
        let entity_ptr = graph_as_runtime::as_types::new_asc_string("NFT");
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
            token_uri: get("tokenURI").and_then(|v| v.as_string().map(|s| s.to_string())).unwrap_or_default(),
            name: get("name").and_then(|v| v.as_string().map(|s| s.to_string())),
            description: get("description").and_then(|v| v.as_string().map(|s| s.to_string())),
            image_uri: get("imageURI").and_then(|v| v.as_string().map(|s| s.to_string())),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(id: &str) -> Option<Self> {
        use graph_as_runtime::native_store::{FieldValue, STORE};
        let fields = STORE.with(|s| s.borrow().get_entity("NFT", id).cloned())?;
        Some(Self {
            id: id.into(),
            owner: match fields.get("owner") {
                Some(FieldValue::Bytes(b)) => b.clone(),
                _ => Default::default(),
            },
            token_uri: match fields.get("tokenURI") {
                Some(FieldValue::String(s)) => s.clone(),
                _ => Default::default(),
            },
            name: match fields.get("name") {
                Some(FieldValue::String(s)) => Some(s.clone()),
                _ => None,
            },
            description: match fields.get("description") {
                Some(FieldValue::String(s)) => Some(s.clone()),
                _ => None,
            },
            image_uri: match fields.get("imageURI") {
                Some(FieldValue::String(s)) => Some(s.clone()),
                _ => None,
            },
        })
    }
}
