# Primitives

Graphite provides Rust types for all The Graph's primitive scalars.

## BigInt

Arbitrary-precision integer, stored as little-endian bytes (`Vec<u8>`). Matches The Graph's `BigInt` scalar.

```rust
use graphite::BigInt;

let a = BigInt::from(1000u64);
let b = BigInt::from_signed_bytes_le(&[100, 0, 0, 0]);

// Arithmetic
let sum   = a.plus(&b);
let diff  = a.minus(&b);
let prod  = a.times(&b);
let quot  = a.divided_by(&b);
let rem   = a.mod_(&b);
let pow   = a.pow(10);

// Comparison
let is_gt = a.gt(&b);
let is_lt = a.lt(&b);
let is_eq = a.equals(&b);

// Bitwise
let shifted = a.left_shift(4);
let anded   = a.bit_and(&b);
let ored    = a.bit_or(&b);

// Conversion
let as_i32: i32 = a.to_i32();
let as_str: String = a.to_string();
let as_hex: String = a.to_hex();
let as_f64: f64 = a.to_f64();

// From various sources
let from_str = BigInt::from_string("12345678901234567890");
let from_hex = BigInt::from_hex("0xdeadbeef");
```

In handlers, `BigInt` values are typically passed as `event.value.clone()` — the generated event structs already carry the right type.

## BigDecimal

Arbitrary-precision decimal. Matches The Graph's `BigDecimal` scalar.

```rust
use graphite::BigDecimal;

let a = BigDecimal::from_string("1234.5678");
let b = BigDecimal::from_f64(3.14);

let sum   = a.plus(&b);
let diff  = a.minus(&b);
let prod  = a.times(&b);
let quot  = a.divided_by(&b);
let neg   = a.neg();
let abs   = a.truncated();

let as_str = a.to_string();
```

## Bytes

A raw byte array. Matches The Graph's `Bytes` scalar.

```rust
use graphite::Bytes;

let b = Bytes::from_hex("0xdeadbeef");
let s = b.to_hex();     // "0xdeadbeef"
let v: Vec<u8> = b.to_vec();
```

In practice, `Bytes` values from events are usually passed directly as `Vec<u8>`:

```rust
.set_transaction_hash(ctx.tx_hash.to_vec())
.set_from(event.from.to_vec())
```

## Address

A 20-byte Ethereum address. Matches The Graph's `Address` scalar.

```rust
use graphite::Address;

let addr = Address::from_bytes(&[0xaa; 20]);
let s = addr.to_hex();  // "0xaaaa...aaaa" (checksummed)
let b: [u8; 20] = addr.to_bytes();
```

## Working with Raw Event Values

Generated event structs expose typed fields that correspond directly to ABI types:

| Solidity ABI type | Rust field type |
|------------------|----------------|
| `address` | `[u8; 20]` |
| `uint256` / `uint128` / etc. | `Vec<u8>` (LE BigInt bytes) |
| `bytes32` | `[u8; 32]` |
| `bytes` | `Vec<u8>` |
| `string` | `String` |
| `bool` | `bool` |
| `int256` | `Vec<u8>` (LE signed bytes) |

Pass them to entity setters without conversion:

```rust
.set_from(event.from.to_vec())  // [u8; 20] → Vec<u8>
.set_value(event.value.clone()) // Vec<u8> BigInt
.set_token_id(event.token_id.clone())
```
