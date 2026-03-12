//! Integration tests for the Entity derive macro.

use graphite::prelude::*;

#[derive(Entity, Debug, PartialEq)]
pub struct Transfer {
    #[id]
    id: String,
    from: Address,
    to: Address,
    value: BigInt,
    memo: Option<String>,
}

#[test]
fn entity_new_has_defaults() {
    let transfer = Transfer::new("tx-123");
    assert_eq!(transfer.id, "tx-123");
    assert_eq!(transfer.from, Address::ZERO);
    assert_eq!(transfer.to, Address::ZERO);
    assert!(transfer.value.is_zero());
    assert_eq!(transfer.memo, None);
}

#[test]
fn entity_to_and_from_roundtrip() {
    let mut transfer = Transfer::new("tx-456");
    transfer.from = Address::repeat_byte(0xAA);
    transfer.to = Address::repeat_byte(0xBB);
    transfer.value = BigInt::from(1000);
    transfer.memo = Some("test memo".into());

    // Convert to Entity
    let entity = transfer.to_entity();

    // Check fields are present
    assert!(entity.get("id").is_some());
    assert!(entity.get("from").is_some());
    assert!(entity.get("to").is_some());
    assert!(entity.get("value").is_some());
    assert!(entity.get("memo").is_some());

    // Convert back
    let restored = Transfer::from_entity(entity).expect("should deserialize");
    assert_eq!(restored, transfer);
}

#[test]
fn entity_save_and_load() {
    let mut host = MockHost::new();

    let mut transfer = Transfer::new("tx-789");
    transfer.from = Address::repeat_byte(0x11);
    transfer.to = Address::repeat_byte(0x22);
    transfer.value = BigInt::from(5000);

    // Save it
    transfer.save(&mut host);

    // Load it back
    let loaded = Transfer::load(&host, "tx-789").expect("should load");
    assert_eq!(loaded.id, "tx-789");
    assert_eq!(loaded.from, Address::repeat_byte(0x11));
    assert_eq!(loaded.to, Address::repeat_byte(0x22));
    assert_eq!(loaded.value, BigInt::from(5000));
}

#[test]
fn entity_remove() {
    let mut host = MockHost::new();

    let transfer = Transfer::new("tx-to-remove");
    transfer.save(&mut host);
    assert!(Transfer::load(&host, "tx-to-remove").is_some());

    Transfer::remove(&mut host, "tx-to-remove");
    assert!(Transfer::load(&host, "tx-to-remove").is_none());
}

#[test]
fn store_trait_entity_type() {
    assert_eq!(Transfer::ENTITY_TYPE, "Transfer");
}

// Test with different ID types and field combinations
#[derive(Entity, Debug, PartialEq)]
pub struct Token {
    #[id]
    id: String,
    symbol: String,
    decimals: i32,
    total_supply: BigInt,
}

#[test]
fn token_entity_works() {
    let mut token = Token::new("0xdead");
    token.symbol = "TEST".into();
    token.decimals = 18;
    token.total_supply = BigInt::from(1_000_000);

    let entity = token.to_entity();
    let restored = Token::from_entity(entity).unwrap();

    assert_eq!(restored.id, "0xdead");
    assert_eq!(restored.symbol, "TEST");
    assert_eq!(restored.decimals, 18);
}
