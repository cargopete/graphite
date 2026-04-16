# Contract Calls

Contract calls let you invoke view (read-only) functions on any contract during indexing. They require a running Ethereum node — they are not available in native `cargo test` without mocking.

## Making a Call

```rust
use graphite::ethereum::{self, EthereumValue};
use graphite::call::ContractCall;

#[handler]
pub fn handle_transfer(event: &ERC20TransferEvent, ctx: &graphite::EventContext) {
    // Call balanceOf(address) on the token contract
    let result = ContractCall::new(ctx.address, "balanceOf(address)")
        .arg(EthereumValue::Address(event.to))
        .call();

    if let Some(values) = result {
        let balance = values[0].clone(); // EthereumValue::Uint(...)
        // use balance...
    }
}
```

## ContractCall API

```rust
use graphite::call::ContractCall;
use graphite::ethereum::EthereumValue;

let result: Option<Vec<EthereumValue>> = ContractCall::new(
    contract_address,   // [u8; 20]
    "functionName(type1,type2)"  // function signature
)
.arg(EthereumValue::Address(addr))
.arg(EthereumValue::Uint(amount_bytes))
.call();
```

`call()` returns `None` if the call reverts or the function is not found. It returns `Some(Vec<EthereumValue>)` with the decoded return values on success.

## ABI Encoding and Decoding

For manual encoding:

```rust
use graphite::ethereum::{self, EthereumValue};

// Encode a value to ABI bytes
let encoded: Vec<u8> = ethereum::encode(&EthereumValue::Uint(value_bytes))?;

// Decode ABI bytes to a value
let value: EthereumValue = ethereum::decode("uint256", &encoded)?;
```

## Mocking Contract Calls in Tests

In tests, use `mock::set_call_result` to provide a return value for a call:

```rust
use graphite::mock;
use graphite::ethereum::EthereumValue;

mock::set_call_result(
    contract_address,
    "balanceOf(address)",
    Some(alloc::vec![EthereumValue::Uint(alloc::vec![200, 0, 0, 0])]),
);

handle_transfer_impl(&event, &graphite::EventContext::default());
```

Pass `None` to simulate a reverted call.

## Notes

- Contract calls are only available when graph-node has an Ethereum node configured. Not all networks support them.
- Calls are expensive — they require an archive node for historical state. Use them sparingly.
- Each call is synchronous and blocks the handler until the result arrives.
