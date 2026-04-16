# Entities

Entities are the data you store in The Graph's indexed store. Each `@entity` type in your `schema.graphql` becomes a generated Rust struct with a builder.

## Schema Definition

```graphql
type Token @entity {
  id: ID!
  owner: Bytes!
  tokenId: BigInt!
  mintedAt: BigInt!
  uri: String
}
```

## Generated API

`graphite codegen` produces a `Token` struct with:

```rust
// Create a new entity (not yet saved)
let token = Token::new("0x1234");

// Builder pattern — chain setters
token
    .set_owner(owner_bytes)
    .set_token_id(token_id_bigint)
    .set_minted_at(block_number)
    .set_uri("ipfs://...".into())
    .save();                        // writes to the store

// Load an existing entity
if let Some(token) = Token::load("0x1234") {
    let owner = token.owner();      // returns &[u8]
    token.set_owner(new_owner).save();
}

// Remove an entity
Token::remove("0x1234");
```

### Methods

| Method | Description |
|--------|-------------|
| `Token::new(id)` | Constructs a new entity with the given ID. Not saved until `.save()` is called. |
| `Token::load(id)` | Loads from the store. Returns `Option<Token>`. |
| `Token::remove(id)` | Deletes the entity from the store. |
| `.set_field(value)` | Sets a field. Returns `&mut Self` for chaining. |
| `.field()` | Gets a field value. Returns a reference to the stored value. |
| `.save()` | Writes all set fields to the store. Upserts — creates if not present, updates if it exists. |

## Field Types

Setter types depend on the GraphQL schema type:

| GraphQL Type | Rust setter type |
|-------------|-----------------|
| `ID!` / `String` | `&str` or `String` |
| `Bytes` / `Address` | `Vec<u8>` |
| `BigInt` | `Vec<u8>` (little-endian bytes) |
| `BigDecimal` | `Vec<u8>` (serialised) |
| `Boolean` | `bool` |
| `Int` | `i32` |
| `Int8` | `i64` |
| `Float` | `f64` |

## Nullable Fields

Fields without `!` are optional. The setter accepts `Option<T>` or you can pass a value directly — the generated code wraps it.

## Immutable Entities

Fields marked `@entity(immutable: true)` in the schema can only be set once. Attempting to update them at a later block will cause graph-node to error.

## @derivedFrom

Fields with `@derivedFrom` are not stored in the entity — graph-node computes them as reverse lookups. They have no generated setter and do not appear in `save()` calls.

```graphql
type Token @entity {
  id: ID!
  transfers: [Transfer!]! @derivedFrom(field: "token")
}

type Transfer @entity {
  id: ID!
  token: Token!
}
```

## Entity IDs

IDs must be unique within an entity type. Common patterns:

```rust
// Event-scoped: tx hash + log index
let id = format!("{}-{}", hex(&ctx.tx_hash), hex(&ctx.log_index));

// Address-scoped
let id = format!("0x{}", hex(&address));

// Compound
let id = format!("{}-{}", token_id, owner);
```
