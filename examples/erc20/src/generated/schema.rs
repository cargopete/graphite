//! Generated entity types from schema.graphql.
//!
//! DO NOT EDIT — regenerate with `graphite codegen`

#![allow(dead_code)]

use graphite::prelude::*;

/// Entity: `Token`
///
/// Fields:
/// - `id`: `ID!`
/// - `symbol`: `String!`
/// - `name`: `String!`
/// - `decimals`: `Int!`
/// - `totalSupply`: `BigInt!`
#[derive(Entity, Debug, Clone, PartialEq)]
pub struct Token {
    #[id]
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub total_supply: BigInt,
}

/// Entity: `Account`
///
/// Fields:
/// - `id`: `ID!`
/// - `balances`: `[TokenBalance!]!`
#[derive(Entity, Debug, Clone, PartialEq)]
pub struct Account {
    #[id]
    pub id: String,
    pub balances: Vec<String /* TokenBalance */>,
}

/// Entity: `TokenBalance`
///
/// Fields:
/// - `id`: `ID!`
/// - `account`: `Account!`
/// - `token`: `Token!`
/// - `balance`: `BigInt!`
#[derive(Entity, Debug, Clone, PartialEq)]
pub struct TokenBalance {
    #[id]
    pub id: String,
    pub account: String /* Account */,
    pub token: String /* Token */,
    pub balance: BigInt,
}

/// Entity: `Transfer`
///
/// Fields:
/// - `id`: `ID!`
/// - `token`: `Token!`
/// - `from`: `Bytes!`
/// - `to`: `Bytes!`
/// - `value`: `BigInt!`
/// - `blockNumber`: `BigInt!`
/// - `timestamp`: `BigInt!`
/// - `transactionHash`: `Bytes!`
#[derive(Entity, Debug, Clone, PartialEq)]
pub struct Transfer {
    #[id]
    pub id: String,
    pub token: String /* Token */,
    pub from: Bytes,
    pub to: Bytes,
    pub value: BigInt,
    pub block_number: BigInt,
    pub timestamp: BigInt,
    pub transaction_hash: Bytes,
}

