# Multi-Source Subgraphs

A single WASM binary can index multiple contracts. This is useful when you want to combine related contracts — for example, an ERC20 token and a liquidity pool — in one deployment.

## Setup

Add multiple `[[contracts]]` sections to `graphite.toml`:

```toml
output_dir = "src/generated"
schema     = "schema.graphql"
network    = "mainnet"

[[contracts]]
name        = "ERC20"
abi         = "abis/ERC20.json"
address     = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
start_block = 6082465

[[contracts.event_handlers]]
event   = "Transfer(address,address,uint256)"
handler = "handle_transfer"

[[contracts]]
name        = "ERC721"
abi         = "abis/ERC721.json"
address     = "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D"
start_block = 12287507

[[contracts.event_handlers]]
event   = "Transfer(address,address,uint256)"
handler = "handle_nft_transfer"
```

## Generated Types

`graphite codegen` generates a separate module for each contract:

```
src/generated/
├── mod.rs
├── erc20.rs     # ERC20TransferEvent
├── erc721.rs    # ERC721TransferEvent
└── schema.rs    # all entity builders
```

Note that two contracts can have events with the same name (like `Transfer`). The generated types are prefixed with the contract name: `ERC20TransferEvent`, `ERC721TransferEvent`. They are fully distinct types.

## Handler Implementation

```rust
mod generated;
use generated::{
    ERC20TransferEvent, ERC721TransferEvent,
    TokenTransfer, NFTTransfer,
};

#[handler]
pub fn handle_transfer(event: &ERC20TransferEvent, ctx: &graphite::EventContext) {
    let id = format!("{}-{}", hex(&ctx.tx_hash), hex(&ctx.log_index));
    TokenTransfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_value(event.value.clone())
        .save();
}

#[handler]
pub fn handle_nft_transfer(event: &ERC721TransferEvent, ctx: &graphite::EventContext) {
    let id = format!("{}-{}", hex(&ctx.tx_hash), hex(&ctx.log_index));
    NFTTransfer::new(&id)
        .set_from(event.from.to_vec())
        .set_to(event.to.to_vec())
        .set_token_id(event.token_id.clone())
        .save();
}
```

## Entity Namespacing

Entity types are global across all contracts in a subgraph — they are defined in `schema.graphql`, not per contract. Make sure entity type names are unique.

## Testing

Tests work identically — `mock::entity_count` and `mock::has_entity` count across the whole store:

```rust
#[test]
fn both_handlers_independent() {
    mock::reset();

    handle_transfer_impl(&token_event, &graphite::EventContext::default());
    handle_nft_transfer_impl(&nft_event, &graphite::EventContext::default());

    assert_eq!(mock::entity_count("TokenTransfer"), 1);
    assert_eq!(mock::entity_count("NFTTransfer"), 1);
}
```

See the [multi-source example](../examples/erc20.md) in the repository for a complete working subgraph.
