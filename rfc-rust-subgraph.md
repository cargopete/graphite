# Designing a native Rust subgraph experience for The Graph

**Rust subgraphs can eliminate AssemblyScript's deepest pain points — broken nullable handling, missing closures, opaque compiler crashes — while delivering ~2× performance and access to Rust's entire crate ecosystem.** The dual-ABI approach proposed in the RFC is architecturally sound: graph-node's `HostExports<C>` layer is already language-agnostic, and the AS-specific coupling exists solely in the serialization boundary between WASM memory and host functions. This report covers every dimension needed to ship a Rust subgraph SDK that developers will genuinely prefer.

---

## 1. Why developers are frustrated with AssemblyScript

The case for a Rust alternative rests on concrete, well-documented developer pain. Across GitHub issues, forums, and community cheatsheets, the same complaints surface repeatedly.

**Nullable handling is the single largest source of bugs.** AssemblyScript's type narrowing only works on local variables, not property accesses — a design flaw the AS compiler team never fixed. Developers must assign every nullable field to a local before checking it, turning three lines of TypeScript into eight lines of AS boilerplate. Worse, nullable arithmetic compiles without error but crashes at runtime: `wrapper.n = wrapper.n + x` where `n` is `BigInt | null` silently produces a WASM trap. Rust's `Option<T>` with `match`, `map`, `unwrap_or`, and `and_then` eliminates this entire class of bugs at compile time.

**No closures means no functional programming.** `array.map()`, `.filter()`, and `.forEach()` all crash the AS compiler. Developers resort to C-style `for` loops with manual index tracking. Rust's iterator chains (`.iter().filter().map().collect()`) would be transformational for subgraph code that frequently manipulates entity arrays and event parameter lists.

**The debugging experience is hostile.** When the AS compiler crashes — which happens on basic operations like null-checking `event.transaction.to` — the official guidance from maintainers is to "comment the whole file and uncomment it little by little." Rust's compiler, by contrast, is renowned for actionable error messages that point directly to the problem and suggest fixes.

Additional pain points compound the frustration: no `Date` support (developers write 50+ line timestamp parsers), no regex, no `console.log` (triggers a cryptic WASM import error), `===` performs identity comparison instead of value equality, array fields on entities can't be mutated in place, and the `BigInt.fromI32()` ceremony turns simple arithmetic into verbose method chains. The testing framework Matchstick requires PostgreSQL installed locally, produces silent type-mismatch failures, and offers no branch coverage. Perhaps most tellingly, **The Graph already validated the Rust path with Substreams** — Rust modules compiled to WASM, described by the team as enabling "extremely high-performance indexing."

---

## 2. graph-node internals and the dual-ABI insertion point

The WASM runtime in `runtime/wasm/src/` uses **Wasmtime** (Cranelift JIT) and follows a clear architecture that makes dual-ABI support tractable.

### How host functions flow today

When a blockchain trigger arrives, `RuntimeHostWasm` sends a `MappingRequest` over an mpsc channel to a mapping thread. That thread calls `WasmInstance::from_valid_module_with_ctx()`, which configures a `wasmtime::Linker` with every host function, then instantiates the module. The handler is invoked by serializing trigger data into AS memory via `ToAscPtr`, calling the exported handler function by name, and collecting the mutated `BlockState`.

Every host function closure follows the same pattern: read `AscPtr<T>` arguments from WASM memory via `asc_get()`, call the corresponding `HostExports` method with native Rust types, then serialize the return value back via `asc_new()`. The critical insight is that **`HostExports<C>` is already completely language-agnostic** — it operates on `String`, `HashMap<String, Value>`, `Vec<u8>`, and other native types. The AS coupling lives entirely in the serialization layer.

### Where to insert language detection

The cleanest insertion point is the manifest's `mapping.language` field, which currently accepts only `wasm/assemblyscript`. Adding `wasm/rust` requires changes to manifest parsing in `graph/src/data_source/`, propagating a `Language` enum through `MappingContext`, and branching in `WasmInstance::from_valid_module_with_ctx()` to register either AS-ABI or Rust-ABI host functions.

```
                    ┌─────────────────────────┐
                    │    HostExports<C>        │  ← Language-agnostic (UNCHANGED)
                    │  (store, crypto, ipfs)   │
                    └────────────┬────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              │                                      │
    ┌─────────┴─────────┐              ┌─────────────┴────────────┐
    │  AscAbiHost        │              │  RustAbiHost              │
    │  (current code)    │              │  (new)                    │
    │  asc_get/asc_new   │              │  ptr+len, bincode deser   │
    │  AscPtr<T>         │              │  repr(C) structs          │
    └────────────────────┘              └──────────────────────────┘
```

### Estimated scope of changes

New code totals roughly **2,000–3,000 lines** across 4–6 new files (`rust_abi.rs`, `rust_abi/types.rs`, `rust_instance.rs`), plus **600–1,200 lines modified** across 8–12 existing files. The highest-risk change involves the memory allocator assumption: graph-node currently calls `allocate()` exported from the AS module. Rust modules must export an equivalent `allocate(size: i32) -> i32` function, or graph-node must adopt a shared-buffer protocol for Rust modules. The bump allocator + `reset_arena()` approach from the RFC is the right design — it avoids the complexity of AS's managed heap while keeping allocation fast and deterministic.

---

## 3. The Rust SDK: graph-rs API design

The SDK (`graph-rs`) wraps all graph-node host functions behind ergonomic Rust types and proc macros. The design draws from Anchor (Solana), ink! (Substrate), and CosmWasm — all battle-tested Rust-to-WASM frameworks.

### Complete host function catalog

Graph-node exposes host functions across these namespaces, all of which need Rust bindings:

- **store**: `set`, `get`, `remove` (entity CRUD)
- **ethereum**: `call`, `encode`, `decode` (contract interaction)
- **log**: `log` (with levels: debug, info, warning, error, critical)
- **ipfs**: `cat`, `map` (content retrieval)
- **crypto**: `keccak256` (hashing)
- **typeConversion**: `bytesToString`, `bytesToHex`, `bigIntToString`, `stringToH160`, `bytesToBase58`, `i32ToBigInt`, `bigIntToI32`
- **bigInt**: `plus`, `minus`, `times`, `dividedBy`, `mod`, `pow`, `fromString`, `bitOr`, `bitAnd`, `leftShift`, `rightShift`
- **bigDecimal**: `plus`, `minus`, `times`, `dividedBy`, `equals`, `toString`, `fromString`
- **dataSource**: `create`, `createWithContext`, `address`, `network`, `context`
- **json**: `fromBytes`, `try_fromBytes`, `toI64`, `toU64`, `toF64`, `toBigInt`
- **ens**: `nameByHash`

### Entity derive macro

The `#[derive(Entity)]` macro generates store serialization, typed load/save, and field accessors:

```rust
use graph_rs::prelude::*;

#[derive(Entity)]
pub struct Transfer {
    #[id]
    id: Bytes,
    from: Address,
    to: Address,
    value: BigInt,
    timestamp: BigInt,
    block_number: BigInt,
}

// Macro expands to:
impl Transfer {
    pub fn new(id: impl Into<Bytes>) -> Self { /* fields default to None internally */ }

    pub fn load(id: &str) -> Option<Self> {
        store::get("Transfer", id).map(Self::from_entity)
    }

    pub fn save(&self) {
        let mut entity = Entity::new();
        entity.set("id", &self.id);
        entity.set("from", &self.from);
        entity.set("to", &self.to);
        entity.set("value", &self.value);
        entity.set("timestamp", &self.timestamp);
        entity.set("blockNumber", &self.block_number); // snake_case → camelCase
        store::set("Transfer", &self.id.to_hex(), entity);
    }

    pub fn remove(id: &str) { store::remove("Transfer", id); }

    fn from_entity(e: Entity) -> Self { /* field extraction with proper types */ }
}
```

### Handler proc macro

The `#[handler]` macro generates the `#[no_mangle] extern "C"` wrapper that graph-node calls:

```rust
// What the developer writes:
#[handler]
pub fn handle_transfer(event: TransferEvent) {
    let id = format!("{}-{}", event.transaction.hash, event.log_index);
    let mut transfer = Transfer::new(Bytes::from_hex(&id));
    transfer.from = event.params.from;
    transfer.to = event.params.to;
    transfer.value = event.params.value;
    transfer.timestamp = event.block.timestamp.into();
    transfer.block_number = event.block.number.into();
    transfer.save();
}

// Macro expands to:
#[no_mangle]
pub extern "C" fn handle_transfer(event_ptr: u32, event_len: u32) {
    let raw = unsafe { core::slice::from_raw_parts(event_ptr as *const u8, event_len as usize) };
    let event = TransferEvent::decode(raw).expect("Failed to decode event");
    // Calls the user's function body
    _handle_transfer_impl(event);
    reset_arena(); // Bump allocator reset after handler completes
}
```

### Strongly typed primitives with operator overloading

```rust
// BigInt wraps host-function arithmetic with native operators
let fee = gas_used * gas_price;          // BigInt * BigInt → BigInt
let balance = old_balance - amount;      // BigInt - BigInt → BigInt
let share = amount / total_supply;       // BigInt / BigInt → BigInt
let threshold = BigInt::from(1_000_000); // From<i64>

// Address and Bytes are zero-copy wrappers
let addr = Address::from_hex("0xdead...beef");
let is_zero = addr == Address::ZERO;

// U256 from alloy-primitives, with From<BigInt> conversion
let value: U256 = event.params.value;
let big: BigInt = value.into();
```

### Reusable crate evaluation

| Crate | Purpose | WASM compatible | Recommendation |
|-------|---------|:-:|---|
| **alloy-primitives** | Address, B256, U256, keccak256 | ✅ Full support | **Use as foundation** — modern, fast, maintained by Paradigm |
| **alloy-sol-types** | `sol!` macro, ABI encode/decode | ✅ | **Use for codegen engine** — replaces ethabi for new code |
| **num-bigint** | Arbitrary-precision integers | ✅ (pure Rust) | **Use for BigInt** — wraps host calls but provides native fallback for testing |
| **num-rational** | Rational numbers for BigDecimal | ✅ | Evaluate for BigDecimal; may use host functions instead |
| **primitive-types** | H160, H256, U256 (older) | ✅ | Skip — alloy-primitives is the successor |
| **ethabi** | ABI codec (older) | ✅ | Skip — alloy-sol-types is better |
| **serde** | Serialization framework | ✅ | **Use for entity serialization** in test mode |

The key constraint: all crates must compile to `wasm32-unknown-unknown` without `std` filesystem or networking. **Alloy explicitly provides "full support for all wasm targets"** per their documentation, making it the ideal foundation.

---

## 4. Build tooling: cargo-subgraph

### What graph-cli does today

Graph-cli handles the full lifecycle: `graph init` scaffolds from a contract address, `graph codegen` generates AS types from ABIs + GraphQL schema, `graph build` compiles AS → WASM, and `graph deploy` uploads to IPFS and registers with graph-node. Code generation produces event classes, function call classes, contract binding classes (from ABIs), and entity classes with `load()`/`save()` methods (from `schema.graphql`).

### The Rust equivalent: cargo-subgraph

```
cargo subgraph init --from-contract 0x... --network mainnet
cargo subgraph codegen     # ABI + schema → Rust types in generated/
cargo subgraph build       # cargo build --target wasm32-unknown-unknown + wasm-opt
cargo subgraph test        # cargo test (native, no WASM)
cargo subgraph deploy      # Upload to IPFS → register with graph-node
```

**Project structure:**

```
my-subgraph/
├── Cargo.toml                  # crate-type = ["cdylib"]
├── subgraph.yaml               # language: wasm/rust
├── schema.graphql
├── abis/
│   └── ERC20.json
├── src/
│   ├── lib.rs                  # Re-exports handlers
│   └── mappings/
│       └── erc20.rs            # Handler implementations
├── generated/                  # Auto-generated by codegen
│   ├── erc20.rs                # Event structs, contract bindings
│   └── schema.rs               # Entity structs
└── tests/
    └── transfer_test.rs
```

### ABI → Rust codegen

From an ERC20 Transfer event ABI, codegen produces:

```rust
// generated/erc20.rs — AUTO-GENERATED, DO NOT EDIT
use graph_rs::prelude::*;

#[derive(Debug, Clone)]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub value: U256,
}

impl FromEthereumEvent for TransferEvent {
    const SIGNATURE: &'static str = "Transfer(address,address,uint256)";

    fn decode(event: &RawEvent) -> Result<Self, DecodeError> {
        Ok(Self {
            from: Address::from_word(event.topics[1]),
            to: Address::from_word(event.topics[2]),
            value: U256::from_be_bytes(event.data[0..32]),
        })
    }
}

pub struct ERC20Contract { address: Address }

impl ERC20Contract {
    pub fn bind(address: Address) -> Self { Self { address } }

    pub fn balance_of(&self, owner: Address) -> Result<U256, CallError> {
        ethereum::call(self.address, "balanceOf(address)", &[owner.into()])
            .map(|r| U256::decode(&r[0]))
    }

    pub fn total_supply(&self) -> Result<U256, CallError> {
        ethereum::call(self.address, "totalSupply()", &[])
            .map(|r| U256::decode(&r[0]))
    }
}
```

### WASM build pipeline

The build command runs: `cargo build --target wasm32-unknown-unknown --release` → `wasm-opt -Oz` → validate exports match manifest → copy to `build/`. Critical Cargo.toml settings for minimal binary size:

```toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = true            # Dead code elimination
codegen-units = 1     # Single codegen unit for better optimization
panic = "abort"       # No unwind tables
strip = true          # Remove debug symbols
```

Target binary size: **under 500KB** after wasm-opt, enabling sub-20ms cold starts in graph-node. The codegen approach should be **build-time file generation** (like graph-cli today) rather than compile-time proc macros, because generated files provide better IDE support and debuggability. An optional `subgraph_abi!("./abis/ERC20.json")` proc macro can serve advanced users who prefer inline generation.

---

## 5. A testing story that makes developers smile

### What Matchstick gets wrong

Matchstick requires PostgreSQL, downloads platform-specific binaries, offers no branch coverage, produces silent type-mismatch failures, and demands verbose manual construction of `ethereum.EventParam` arrays for every test event. The ArtBlocks team documented: "Matchstick won't always flag a type mismatch. Instead, the test might pass, or break with no verbose error message."

### Native Rust testing: just `cargo test`

The Rust approach runs handlers natively — no WASM compilation, no external dependencies. Host functions are abstracted behind a `HostFunctions` trait with a `MockHost` implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use graph_rs::testing::*;

    #[test]
    fn transfer_creates_entity() {
        let mut host = MockHost::default();

        let event = TransferEvent::mock()
            .from(addr("0xaaaa..."))
            .to(addr("0xbbbb..."))
            .value(1_000u64)
            .block_number(12345)
            .build();

        handle_transfer(&mut host, &event);

        assert_eq!(host.store.entity_count("Transfer"), 1);
        host.store.assert_field_equals(
            "Transfer", &event.id(),
            "value", &Value::BigInt(1_000u64.into()),
        );
    }

    #[test]
    fn zero_value_transfer_ignored() {
        let mut host = MockHost::default();
        let event = TransferEvent::mock().value(0u64).build();
        handle_transfer(&mut host, &event);
        assert_eq!(host.store.entity_count("Transfer"), 0);
    }
}
```

The **builder pattern** for events replaces Matchstick's verbose `ethereum.EventParam` arrays. The `MockHost` provides an in-memory HashMap store, configurable ethereum call mocks, and IPFS mocks — all without PostgreSQL.

### Property-based testing with proptest

Rust's ecosystem enables testing strategies impossible in AS:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn handler_never_panics(
        from in arb_address(),
        to in arb_address(),
        value in 0u64..1_000_000_000u64,
    ) {
        let mut host = MockHost::default();
        let event = TransferEvent::mock().from(from).to(to).value(value).build();
        handle_transfer(&mut host, &event); // Must not panic
    }

    #[test]
    fn entity_ids_always_nonempty(event in arb_transfer_event()) {
        let mut host = MockHost::default();
        handle_transfer(&mut host, &event);
        for (_, entities) in host.store.all() {
            for (id, _) in entities {
                prop_assert!(!id.is_empty());
            }
        }
    }
}
```

**Snapshot testing** with `insta` captures full store state as JSON, making it trivial to detect regressions. **Full branch coverage** via `cargo-llvm-cov` replaces Matchstick's handler-level-only coverage. **WASM integration tests** using wasmtime verify that the compiled module exports the correct handler functions and interacts correctly with mock host functions.

| Capability | Matchstick (AS) | Rust native testing |
|---|---|---|
| Setup requirements | Node.js + PostgreSQL + binary download | `cargo add --dev` only |
| Test runner | Custom `graph test` binary | Standard `cargo test` |
| Event creation | Manual EventParam arrays (~15 lines) | Builder pattern (~1 line) |
| Type safety | Silent coercion bugs at runtime | Compile-time guarantees |
| Coverage | Handler-level only | Full branch coverage |
| Property testing | Not possible | `proptest` built-in |
| Snapshot testing | Not available | `insta` crate |
| Debugging | `logStore()` only | Full breakpoint debugging |
| Speed | WASM compilation each run | Native speed, instant |

---

## 6. Competitive landscape confirms the opportunity

### No indexer offers Rust as a mapping language

Across Ponder (TypeScript), Subsquid (TypeScript), Envio/HyperIndex (TypeScript/ReScript), Goldsky (AS subgraphs), and Sentio (TypeScript), **no blockchain indexing platform supports Rust for handler/mapping code**. The closest analog is **Reth ExEx** (Execution Extensions), which allows Rust indexing code compiled directly into the Reth node binary — delivering 10–100× performance but requiring node operation. A Rust subgraph SDK would be a genuine first-mover advantage.

### Prior art validates the approach

The `subgraph` crate on crates.io (~2,800 downloads, by Jose Alvarez and Nicholas Rodrigues Lordello) provides basic Rust bindings to graph-node host functions, proving community interest. Edge & Node's job listing for a "Rust Engineer (WASM)" explicitly mentions "building developer tooling for WASM modules written in Rust and JS/TS" and "definition of the interface between the host and WASM." Substreams already demonstrates Rust → WASM in The Graph's ecosystem, and the Uniswap V3 Substreams-powered subgraph "syncs much faster" than its AS equivalent.

### Macro DX lessons from Anchor, ink!, CosmWasm, and Stylus

Four Rust-to-WASM frameworks offer proven macro patterns:

- **Anchor** (Solana): `#[program]` marks the handler module, `#[derive(Accounts)]` generates validation, `#[account]` auto-serializes state. Lesson: declarative constraints as attributes work well, but manual `space` calculations are error-prone — auto-calculate entity sizes.
- **ink!** (Substrate): `#[ink::contract]` wraps an entire module, `#[ink(message)]` marks callable functions, `#[ink(storage)]` defines state. Lesson: "just standard Rust in a well-defined format" is the gold standard for DX — minimize magic, maximize familiarity.
- **CosmWasm**: `#[entry_point]` generates FFI stubs, `Deps`/`DepsMut` provide typed access to storage and API. Lesson: trait-based host function abstraction (`mock_dependencies()`) enables excellent native testing — adopt this pattern directly.
- **Arbitrum Stylus**: `sol_storage!{}` defines Solidity-compatible storage, `#[public]` marks external functions, `#[entrypoint]` marks the contract. Lesson: `TestVM::default()` mocks are clean; `cargo stylus check/deploy` is the right CLI pattern.

The recommended macro hierarchy for subgraphs synthesizes these lessons:

```rust
// Minimal, familiar, composable
#[derive(Entity)]     // Like #[account] in Anchor
struct Transfer { .. }

#[handler]            // Like #[ink(message)] or #[entry_point]
fn handle_transfer(event: TransferEvent) { .. }
```

---

## A complete end-to-end Rust subgraph

Putting it all together, here's what an ERC20 Transfer indexer looks like in Rust:

```rust
// src/lib.rs
mod mappings;

// src/mappings/erc20.rs
use crate::generated::{erc20::TransferEvent, schema::Transfer};
use graph_rs::prelude::*;

#[handler]
pub fn handle_transfer(event: TransferEvent) {
    let id = format!("{:#x}-{}", event.transaction.hash, event.log_index);

    let mut transfer = Transfer::new(id);
    transfer.from = event.params.from;
    transfer.to = event.params.to;
    transfer.value = event.params.value.into();
    transfer.timestamp = event.block.timestamp.into();
    transfer.block_number = event.block.number.into();
    transfer.save();

    log::info!("Transfer: {} → {} ({})",
        event.params.from, event.params.to, event.params.value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_rs::testing::*;

    #[test]
    fn creates_transfer_entity() {
        let mut ctx = MockHost::default();
        let event = TransferEvent::mock()
            .from(addr("0xaaaa...")).to(addr("0xbbbb..."))
            .value(1000u64).block_number(18_000_000).build();

        handle_transfer(&mut ctx, &event);

        assert_eq!(ctx.store.entity_count("Transfer"), 1);
    }
}
```

Compare this to the AS equivalent: no nullable ceremony, no `BigInt.fromI32()` boilerplate, no class constructors, closures and iterators available, standard `cargo test` with no PostgreSQL, and compile-time type safety that prevents the entire class of runtime null-pointer traps that plague AS subgraphs. The estimated **3–6 week implementation timeline** for graph-node changes makes this an achievable investment with outsized developer experience returns.

## Conclusion

The path to Rust subgraphs is clear and architecturally clean. Graph-node's `HostExports<C>` is already language-agnostic — only the WASM serialization layer needs a parallel Rust ABI implementation, estimated at ~3,000 lines of new code. The SDK design should follow ink!'s philosophy of "just standard Rust with attribute macros," using `alloy-primitives` as the type foundation and build-time codegen from ABIs and GraphQL schemas. The testing story alone — `cargo test` with mock stores, property-based testing, full branch coverage, zero external dependencies — would justify the effort. Combined with Rust's type safety eliminating AS's nullable-crash epidemic, ~2× WASM performance, and access to the entire crate ecosystem, this positions The Graph to capture the growing population of Rust-native blockchain developers that Reth, Alloy, and Foundry are cultivating.
