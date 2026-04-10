# Graphite Roadmap — April 2026

Goal: full feature parity with AssemblyScript `graph-ts` so any production subgraph can be written in Rust.

Current state: store operations, event handlers, scalar ABI types, array/tuple decoding, ethereum.decode, contract calls, data sources, schema derivedFrom, call handlers, and native testing all work. ERC20 and ERC721 subgraphs are live on The Graph Studio (Arbitrum One).

---

## Week 1 — Quick Wins (Apr 9–13)

Low-effort, high-value. None of these require architectural changes.

### 1.1 `BigDecimal` arithmetic
**Status:** type exists, zero operators.
Add `Add`, `Sub`, `Mul`, `Div` impls in `graphite/src/primitives.rs`. Back them with `num-bigint` (already a dependency). Every DeFi subgraph needs this for price and TVL calculations.

### 1.2 `BigInt` modulo
**Status:** `%` operator missing.
Trivial `Rem` impl. Financial calculations and tick/slot math need it.

### 1.3 `entity.remove()` codegen
**Status:** `store.remove` FFI works, codegen never emits a `.remove()` method.
Add `pub fn remove(id: &str)` to each generated entity in `schema.rs`. One-line call to `ffi::store_remove`.

### 1.4 `@entity(immutable: true)` support
**Status:** schema codegen ignores all directives beyond `@entity`.
Parse the `immutable` argument in `schema.rs`. Immutable entities don't emit `save()` — only an initial write. Affects generated struct and serialisation.

### 1.5 `Address` type utilities
**Status:** `Address` re-exported from `alloy_primitives` but no `from_string()` convenience.
Add `Address::from_hex_str(s: &str)` in `graphite/src/primitives.rs`. Subgraphs compare addresses as strings all the time.

### 1.6 Field nullability in codegen
**Status:** all generated entity fields are `Option<T>` regardless of `!` in the schema.
Respect the non-null marker — non-nullable fields should be `T`, not `Option<T>`, to give users a compiler error rather than a silent `None`.

---

## Week 2 — Tuple and Array Event Decoding (Apr 14–20)

The single biggest gap. Any contract using complex event params breaks today.

### 2.1 Runtime decoder for array types
**Status:** `KIND_ARRAY` and `KIND_FIXED_ARRAY` discriminants recognised but return stub `EthereumValue::Array`.
Implement full decoding in `graph-as-runtime/src/ethereum.rs`:
- Fixed arrays: `T[N]` → `Vec<EthereumValue>`
- Dynamic arrays: `T[]` → `Vec<EthereumValue>`
- Nested arrays: `T[][]`

### 2.2 Runtime decoder for tuple types
**Status:** `KIND_TUPLE` returns stub.
Decode `(T1, T2, ...)` → `Vec<EthereumValue>` recursively.
Covers: Uniswap V3 `Swap`, Aave `Borrow`, most modern DeFi events.

### 2.3 Codegen for complex event param types
**Status:** codegen maps everything it doesn't recognise to `find_bytes()`.
Update `abi.rs` to emit correct field types for arrays and tuples:
- `uint256[]` → `Vec<Vec<u8>>`
- `address[]` → `Vec<[u8; 20]>`
- `(uint256, address)` → a named tuple struct or `(Vec<u8>, [u8; 20])`

### 2.4 `ethereum.decode` host function
**Status:** not present anywhere.
Add `ffi::ethereum_decode(types: AscPtr, data: AscPtr) -> AscPtr` and a higher-level wrapper `ethereum::decode(types: &[AbiType], data: &[u8]) -> Vec<EthereumValue>`. Required for subgraphs that decode calldata directly.

---

## Week 3 — Contract Calls and Dynamic Data Sources (Apr 21–25)

The two features that unlock DeFi factory patterns and on-chain state reads.

### 3.1 `ethereum.call` SDK wrapper
**Status:** FFI imported, no SDK or mock.
Add a `ContractCall` builder to `graphite`:
```rust
let result = ContractCall::new(address, "latestAnswer()(int256)")
    .call(host)?;
```
Must also add a `MockContractCall` to `graphite/src/mock.rs` so calls can be stubbed in `cargo test`.

### 3.2 `ethereum.call` native mock
Test support: `mock::set_call_result(address, signature, result)` and `mock::assert_called(address, signature)`.

### 3.3 Dynamic data sources
**Status:** `dataSource.create` FFI imported, no codegen, no manifest template support.
Three parts:
1. Parse `templates:` section in `subgraph.yaml` in the deploy tool.
2. Generate a `DataSourceTemplate::create(name, address)` call in codegen.
3. Expose `dataSource::address()` and `dataSource::network()` in the SDK.

### 3.4 `crypto.keccak256` native mock / implementation
**Status:** FFI imported, returns 0 locally.
On native builds, use `tiny-keccak` (no_std compatible) instead of calling the host. This means keccak works in `cargo test` without any mock setup.

---

## Week 4 — Schema Completeness and Handler Coverage (Apr 26–30)

Round out the schema layer and handler types.

### 4.1 `@derivedFrom` in schema codegen
**Status:** ✅ done. Fields with `@derivedFrom` are detected and excluded from the generated struct, setters, save(), and load(). Graph-node handles the reverse lookup at query time.
`@derivedFrom(field: "token")` should generate a read-only derived accessor that queries `store.get` with the reverse-lookup field. Required for entity relationships in every non-trivial subgraph.

### 4.2 Call handlers
**Status:** ✅ done (SDK + codegen). `#[handler(call)]` generates WASM entry points that call `read_ethereum_call` and decode via `FromRawCall`. `{Contract}{Fn}Call` structs are generated by `graphite codegen` for every ABI function. Manifest `callHandlers` section parsing is still manual (not auto-generated by the deploy tool).

### 4.3 Multiple data sources — validated example
**Status:** deploy tool code should handle it, untested.
Write a two-source subgraph example (e.g. ERC20 + ERC721 in one manifest) and add a CI test that deploys it to a local graph-node. Confirm the deploy tool handles multiple WASM files correctly.

### 4.4 Receipt handlers
**Status:** ethereum.rs allocates space for receipt (nullable), never decoded.
Expose `EthereumTransactionReceipt` with status and logs array. Add `#[handler(receipt)]` macro variant. Lower priority — uncommon in production subgraphs.

### 4.5 Non-fatal errors
**Status:** ✅ done. `nonfatal_error!(host, "msg")` macro logs at CRITICAL level. With `features: [nonFatalErrors]` in the manifest, graph-node records the error and continues indexing.

### 4.6 `ipfs.cat` native mock
**Status:** ✅ done. `mock::set_ipfs_result(cid, bytes)` registers content in a thread-local; `MockHost::ipfs_cat` checks both instance content and the thread-local.

---

## Out of Scope for April

These are real features but belong in a later release:

| Feature | Reason deferred |
|---------|----------------|
| **ENS** (`ens.nameByAddress`) | Rare in production subgraphs, complex to mock |
| **JSON module** | Needed for NFT metadata; sizeable parser work |
| **File data sources** (IPFS indexing) | New graph-node feature, separate indexing model |
| **Subgraph grafting** | Deployment-time feature, not a handler concern |
| **Timeseries / aggregations** | Very new graph-node feature, schema model differs |
| **Subgraph composition** | Separate protocol concern |
| **Fulltext search** (`@fulltext`) | PostgreSQL-specific, no WASM relevance |

---

## Success Criteria

By end of April, a developer should be able to:

1. Index any ERC20/ERC721/ERC1155 event with no manual workarounds
2. Index a factory-pattern contract (Uniswap V2 pairs, etc.) via dynamic data sources
3. Call contract view functions from inside a handler
4. Write entities with relationships using `@derivedFrom`
5. Run `cargo test` and cover all handler logic — including contract calls and keccak — without Docker

That covers ~95% of production subgraph patterns on The Graph today.
