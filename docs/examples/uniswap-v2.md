# Uniswap V2

**Source:** [`examples/uniswap-v2/`](https://github.com/your-org/graphite/tree/main/examples/uniswap-v2)

The definitive factory + template example. Demonstrates dynamic data sources, counter updates, and the `data_source::address_current()` API.

## What It Does

1. A `Factory` contract emits `PairCreated` whenever a new liquidity pool is deployed.
2. The factory handler creates a `Pool` entity and calls `data_source::create_contract("Pair", pair_address)` to start indexing the new pair.
3. Each pair emits `Swap` events. The `Pair` template handler records each swap and increments `Pool.swapCount`.

## Schema

```graphql
type Pool @entity {
  id: ID!       # pair address (0x-prefixed hex)
  token0: Bytes!
  token1: Bytes!
  swapCount: BigInt!
}

type Swap @entity {
  id: ID!
  pool: Pool!
  amount0In: BigInt!
  amount1In: BigInt!
  amount0Out: BigInt!
  amount1Out: BigInt!
  blockNumber: BigInt!
  timestamp: BigInt!
}
```

## Factory Handler

```rust
#[handler]
pub fn handle_pair_created(event: &FactoryPairCreatedEvent, _ctx: &graphite::EventContext) {
    let pool_id = addr_hex(&event.pair);

    Pool::new(&pool_id)
        .set_token0(event.token0.to_vec())
        .set_token1(event.token1.to_vec())
        .save();

    data_source::create_contract("Pair", event.pair);
}
```

## Template Handler

```rust
#[handler]
pub fn handle_swap(event: &PairSwapEvent, ctx: &graphite::EventContext) {
    let pair_addr = data_source::address_current();
    let pool_id = addr_hex(&pair_addr);

    let swap_id = format!("{}-{}", hex_bytes(&event.tx_hash), hex_bytes(&event.log_index));
    Swap::new(&swap_id)
        .set_pool(pool_id.clone())
        .set_amount0_in(event.amount0_in.clone())
        .set_amount1_in(event.amount1_in.clone())
        .set_amount0_out(event.amount0_out.clone())
        .set_amount1_out(event.amount1_out.clone())
        .set_block_number(ctx.block_number.clone())
        .set_timestamp(ctx.block_timestamp.clone())
        .save();

    // Increment pool.swapCount
    if let Some(pool) = Pool::load(&pool_id) {
        let new_count = le_add_one(pool.swap_count());
        pool.set_swap_count(new_count).save();
    }
}
```

## Counter Arithmetic

BigInt values are stored as little-endian byte vectors. The `le_add_one` helper increments them:

```rust
fn le_add_one(bytes: &[u8]) -> Vec<u8> {
    let mut result = bytes.to_vec();
    let mut carry = 1u16;
    for byte in result.iter_mut() {
        let sum = *byte as u16 + carry;
        *byte = sum as u8;
        carry = sum >> 8;
    }
    if carry > 0 {
        result.push(carry as u8);
    }
    result
}
```

## Tests

The example has four tests:

- **`pair_created_makes_pool_and_data_source`** — factory handler creates `Pool` and triggers data source creation.
- **`swap_creates_entity_and_increments_pool_count`** — swap handler creates `Swap` and updates `Pool.swapCount`.
- **`swap_count_increments_per_swap`** — two swaps produce `swapCount = [2]`.
- **`multiple_pairs_independent`** — two factory events produce two independent pools.

```bash
cd examples/uniswap-v2
cargo test
```

## Key Points

- `data_source::address_current()` returns the address of the contract instance that emitted the event — this is how the template handler knows which pool it belongs to.
- `mock::set_current_address(addr)` sets this value in tests.
- `mock::assert_contract_data_source_created("Pair", addr)` verifies the factory called `create_contract`.
