# Dynamic Data Sources

Dynamic data sources (also called the factory pattern) let you start indexing a new contract address at runtime — typically when a factory contract deploys a new instance.

The classic example is Uniswap V2: when the factory emits `PairCreated`, you create a new data source for the pair contract so its `Swap` events get indexed.

## Setup

Declare a template in `graphite.toml`:

```toml
[[contracts]]
name        = "Factory"
abi         = "abis/Factory.json"
address     = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"
start_block = 10000835

[[contracts.event_handlers]]
event   = "PairCreated(address,address,address,uint256)"
handler = "handle_pair_created"

[[templates]]
name = "Pair"
abi  = "abis/Pair.json"

[[templates.event_handlers]]
event   = "Swap(address,uint256,uint256,uint256,uint256,address)"
handler = "handle_swap"
```

## Factory Handler

In the factory handler, call `data_source::create_contract` to start indexing the new address:

```rust
use graphite::data_source;

#[handler]
pub fn handle_pair_created(event: &FactoryPairCreatedEvent, _ctx: &graphite::EventContext) {
    let pool_id = addr_hex(&event.pair);

    Pool::new(&pool_id)
        .set_token0(event.token0.to_vec())
        .set_token1(event.token1.to_vec())
        .save();

    // Start indexing events from this pair address using the "Pair" template
    data_source::create_contract("Pair", event.pair);
}
```

## Template Handler

Template handlers use `data_source::address_current()` to find out which instance they're running for:

```rust
#[handler]
pub fn handle_swap(event: &PairSwapEvent, ctx: &graphite::EventContext) {
    let pair_addr = data_source::address_current();
    let pool_id = addr_hex(&pair_addr);

    let swap_id = format!("{}-{}", hex(&event.tx_hash), hex(&event.log_index));
    Swap::new(&swap_id)
        .set_pool(pool_id.clone())
        .set_amount0_in(event.amount0_in.clone())
        .set_amount1_out(event.amount1_out.clone())
        .set_block_number(ctx.block_number.clone())
        .save();
}
```

## Data Source API

```rust
use graphite::data_source;

// Create a new contract data source instance
data_source::create_contract("TemplateName", address_bytes);

// Create with context data attached
data_source::create_contract_with_context("TemplateName", address_bytes, context_map);

// Introspect the current data source (inside a template handler)
let addr: [u8; 20]  = data_source::address_current();
let net:  String    = data_source::network_current();
let id:   String    = data_source::id_current();
let ctx:  TypedMap  = data_source::context_current();
```

## Testing Dynamic Data Sources

```rust
#[test]
fn pair_created_makes_data_source() {
    mock::reset();

    handle_pair_created_impl(&event, &graphite::EventContext::default());

    mock::assert_contract_data_source_created("Pair", pair_address);
    assert!(mock::has_entity("Pool", &addr_hex(&pair_address)));
}

#[test]
fn swap_increments_count() {
    mock::reset();

    // Create the pool first
    handle_pair_created_impl(&factory_event, &graphite::EventContext::default());

    // Set the data source address for the template handler
    mock::set_current_address(pair_address);

    handle_swap_impl(&swap_event, &graphite::EventContext::default());
    assert_eq!(mock::entity_count("Swap"), 1);
}
```

See the [Uniswap V2 example](../examples/uniswap-v2.md) for a complete factory + template subgraph.
