//! Generated entity types from schema.graphql.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use graphite::prelude::*;

/// Entity: `Token`
///
/// Fields:
/// - `id`: `ID!`
/// - `owner`: `Bytes!`
/// - `approved`: `Bytes` (optional)
#[derive(Entity, Debug, Clone, PartialEq)]
pub struct Token {
    #[id]
    pub id: String,
    pub owner: Bytes,
    pub approved: Bytes,
}

/// Entity: `Transfer`
///
/// Fields:
/// - `id`: `ID!`
/// - `from`: `Bytes!`
/// - `to`: `Bytes!`
/// - `tokenId`: `BigInt!`
/// - `blockNumber`: `BigInt!`
/// - `timestamp`: `BigInt!`
/// - `transactionHash`: `Bytes!`
#[derive(Entity, Debug, Clone, PartialEq)]
pub struct Transfer {
    #[id]
    pub id: String,
    pub from: Bytes,
    pub to: Bytes,
    pub token_id: BigInt,
    pub block_number: BigInt,
    pub timestamp: BigInt,
    pub transaction_hash: Bytes,
}

/// Entity: `Approval`
///
/// Fields:
/// - `id`: `ID!`
/// - `owner`: `Bytes!`
/// - `approved`: `Bytes!`
/// - `tokenId`: `BigInt!`
/// - `blockNumber`: `BigInt!`
/// - `transactionHash`: `Bytes!`
#[derive(Entity, Debug, Clone, PartialEq)]
pub struct Approval {
    #[id]
    pub id: String,
    pub owner: Bytes,
    pub approved: Bytes,
    pub token_id: BigInt,
    pub block_number: BigInt,
    pub transaction_hash: Bytes,
}

