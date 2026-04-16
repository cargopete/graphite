# Event Context Reference

Every handler receives a context parameter alongside the event. It carries the block and transaction data available at the time of indexing.

## EventContext

Passed to event handlers (`#[handler]`) and block handlers (`#[handler(block)]`).

```rust
pub struct EventContext {
    pub address:                [u8; 20],
    pub log_index:              Vec<u8>,         // LE BigInt bytes
    pub transaction_log_index:  Vec<u8>,
    pub log_type:               Option<String>,

    pub block_hash:             [u8; 32],
    pub block_number:           Vec<u8>,         // LE BigInt bytes
    pub block_timestamp:        Vec<u8>,
    pub block_gas_used:         Vec<u8>,
    pub block_gas_limit:        Vec<u8>,
    pub block_difficulty:       Vec<u8>,
    pub block_total_difficulty: Vec<u8>,
    pub block_base_fee_per_gas: Option<Vec<u8>>, // EIP-1559, None pre-London

    pub tx_hash:                [u8; 32],
    pub tx_index:               Vec<u8>,
    pub tx_from:                [u8; 20],
    pub tx_to:                  Option<[u8; 20]>, // None for contract creation
    pub tx_value:               Vec<u8>,
    pub tx_gas_limit:           Vec<u8>,
    pub tx_gas_price:           Vec<u8>,
    pub tx_nonce:               Vec<u8>,
    pub tx_input:               Vec<u8>,

    pub receipt:                Option<TransactionReceipt>,
}
```

`receipt` is populated only when `receipt = true` is set in `graphite.toml` for the contract.

### TransactionReceipt

```rust
pub struct TransactionReceipt {
    pub transaction_hash:   [u8; 32],
    pub transaction_index:  Vec<u8>,
    pub block_hash:         [u8; 32],
    pub block_number:       Vec<u8>,
    pub cumulative_gas_used: Vec<u8>,
    pub gas_used:           Vec<u8>,
    pub contract_address:   Option<[u8; 20]>,
    pub status:             Option<Vec<u8>>,    // 1 = success, 0 = reverted
    pub root:               Option<[u8; 32]>,
    pub logs_bloom:         Vec<u8>,
    pub logs:               Vec<EthereumLog>,
}
```

### Enabling Receipt Data

```toml
[[contracts]]
name    = "ERC20"
abi     = "abis/ERC20.json"
address = "0x..."
receipt = true   # populate ctx.receipt in handlers
```

## CallContext

Passed to call handlers (`#[handler(call)]`).

```rust
pub struct CallContext {
    pub from:         [u8; 20],
    pub to:           [u8; 20],
    pub block_hash:   [u8; 32],
    pub block_number: Vec<u8>,
    pub tx_hash:      [u8; 32],
    pub tx_index:     Vec<u8>,
    pub gas:          Vec<u8>,
    pub gas_used:     Vec<u8>,
    pub input:        Vec<u8>,
    pub output:       Vec<u8>,
    pub value:        Vec<u8>,
}
```

## FileContext

Passed to file data source handlers (`#[handler(file)]`).

```rust
pub struct FileContext {
    pub id:      String,    // data source ID
    pub address: [u8; 20],  // address that created this data source
}
```

## Using Context Fields in Tests

`EventContext::default()` constructs a zeroed context for use in tests:

```rust
handle_transfer_impl(&event, &graphite::EventContext::default());
```

To set specific fields:

```rust
let ctx = graphite::EventContext {
    tx_hash: [0xab; 32],
    block_number: alloc::vec![42, 0, 0, 0],
    ..Default::default()
};
handle_transfer_impl(&event, &ctx);
```
