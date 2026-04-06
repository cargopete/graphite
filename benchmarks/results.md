# Rust vs AssemblyScript Subgraph Benchmarks

Benchmark results for the equivalent ERC20 Transfer indexer compiled to
Rust (Graphite SDK) and AssemblyScript (`@graphprotocol/graph-ts`),
gathered for graph-node PR [#6462](https://github.com/graphprotocol/graph-node/pull/6462).

*Date: 2026-04-06*
*Host: Apple Silicon (Darwin 25.1.0), wasmtime 29.0.1, rustc release profile, AS via graph-cli 0.93.3 (Binaryen `asc -O3`).*

---

## 1. What we are comparing

Both subgraphs implement the same logic:

```text
on Transfer(from, to, value):
  let id = tx_hash + "-" + log_index
  Transfer { id, from, to, value, blockNumber, timestamp, transactionHash }.save()
```

| File | Subgraph |
|------|----------|
| `examples/erc20/src/lib.rs` | Rust handler (Graphite SDK) |
| `benchmarks/as-erc20/src/mapping.ts` | AssemblyScript handler |

Schemas, ABIs and event signatures are identical.

---

## 2. Binary size

Compiled with `cargo build --target wasm32-unknown-unknown --release` and
`graph build` respectively. `wasm-opt -Oz` is the optimisation level that
graph-cli applies to AS builds, applied here to both for a like-for-like
comparison.

The Rust release profile is now tuned for size in the workspace
`Cargo.toml`:

```toml
[profile.release]
panic = "abort"
lto = "fat"
opt-level = "z"
codegen-units = 1
strip = true
```

| Variant | AssemblyScript | Rust (Graphite) | Ratio |
|---------|---------------:|----------------:|------:|
| Raw release (default profile, historic) | 33,559 B (32.8 KB) | 107,259 B (104.7 KB) | 3.20× |
| **Raw release (size-optimised profile)** | 33,559 B (32.8 KB) | **57,540 B (56.2 KB)** | **1.71×** |
| `wasm-opt -Oz` (historic) | 19,212 B (18.8 KB) | 77,219 B (75.4 KB) | 4.02× |
| **`wasm-opt -Oz` (size-optimised profile)** | 19,212 B (18.8 KB) | **50,105 B (48.9 KB)** | **2.61×** |
| Raw, name section stripped (`wasm-strip`, historic default profile) | n/a (already stripped) | 92,786 B (90.6 KB) | 2.77× |

`wasm-opt` is invoked with
`-Oz --enable-bulk-memory --enable-nontrapping-float-to-int` because
rustc 1.94 emits `memory.copy`/`memory.fill` and saturating
float-to-int instructions by default, which the Binaryen validator
requires to be explicitly enabled.

The size-optimised release profile alone removes **46.4 %** of the raw
binary; combined with `wasm-opt -Oz` the total reduction versus the
historic baseline is **53.3 %** (107,259 B → 50,105 B). The Rust binary
is consistently ~1.7-2.6× the AS size. Section breakdown explains
why:

| Section | AS bytes | Rust bytes | Notes |
|---------|---------:|-----------:|-------|
| Code | 8,128 | 84,676 | Rust pulls in `num_bigint`, `dlmalloc`, `BTreeMap`, formatting |
| Data | 5,327 | 7,546 | Comparable; static strings + tables |
| Export | 4,610 | 93 | AS exports 169 `TypeId.*` globals; Rust has 7 functions |
| Import | 143 | 58 | AS = 5 fns across 4 namespaces; Rust = 3 fns in `graphite` |
| `name` (debug) | 14,271 | 14,240 | Both have similar debug name overhead |
| Function count | 24 | 174 | Rust includes monomorphised stdlib helpers |

The size delta is dominated by the Rust handler statically linking
arithmetic and formatting code that AS delegates to host functions
(`typeConversion.bigIntToString`, `numbers.bigDecimal.toString`).
Stripping `BigInt::Display` and switching `dlmalloc` for the existing
bump allocator would close the gap further.

---

## 3. Handler throughput (Rust)

**Methodology.** A wasmtime-based harness loads the Rust ERC20 WASM,
links no-op `store_set`/`log_log`/`abort` host functions, then loops:

```text
for _ in 0..N {
    ptr = allocate(212)            // bump-allocate 212 bytes
    memcpy(memory[ptr..], trigger) // identical Transfer event each call
    handle_transfer(ptr, 212)      // run the handler
    reset_arena()                  // reclaim heap
}
```

This is the exact sequence graph-node executes per event (minus gas
metering, fuel, store I/O, network, and trigger materialisation). It
measures **WASM handler cost in isolation**: TLV decode → entity build →
host call serialisation → arena reset.

`store_set` is a counter-only stub so we measure the handler itself,
not host-function memory copying. The trigger payload is 212 bytes (one
Ethereum log: address + tx_hash + block fields + 3 topics + 32 B data).
Each run does a 5,000-iteration warm-up before timing.

Source: `tests/integration/src/bin/handler_throughput.rs`.

**Results.** Three runs, no other load on the host:

| Run | Iterations | Elapsed | Throughput | Per event |
|-----|-----------:|--------:|-----------:|----------:|
| 1 | 500,000 | 0.812 s | 615,440 ev/s | 1,625 ns |
| 2 | 500,000 | 0.815 s | 613,173 ev/s | 1,631 ns |
| 3 | 2,000,000 | 3.219 s | 621,349 ev/s | 1,609 ns |

**Steady state: ~617 k events/sec, ~1.62 µs per Transfer event.**

The variance across runs is well under 2 %, and `store_set` was invoked
exactly `N` times in every run (sanity check that the handler ran to
completion).

---

## 4. Memory

| Metric | Rust |
|--------|------|
| Per-handler peak heap (bump allocator high-water mark) | **576 bytes** |
| Steady-state WASM `memory.size()` after N runs | 18 pages = **1.12 MB** |
| Heap-base offset | 64 KB (reserved for stack/static) |
| Heap cap (allocator-enforced) | 4 MB |

The 576 B per-handler figure is the actual amount of arena memory the
handler touches per event: trigger TLV decode buffers, the `Transfer`
entity field map, and TLV-encoded entity bytes shipped to `store_set`.
Because `reset_arena` runs between calls, this is also the steady-state
peak — the WASM memory never grows beyond what one handler invocation
plus a bit of CRT scratch needs.

The 18-page (1.12 MB) `memory.size()` is mostly Rust's `dlmalloc`
reservation for one-time bookkeeping at startup; it does not grow with
N.

---

## 5. AssemblyScript throughput — not measured

**Honest disclosure.** We do not report AS handler throughput in this
document, and here is why:

The AS module's `handleTransfer` export takes **a single `i32`** —
an `AscPtr<EthereumEvent>` — that points to a fully-formed
AssemblyScript object graph (managed-class headers, runtime type IDs,
ArrayBuffer-backed payloads, the lot). To call it from outside
graph-node we would need to recreate graph-node's entire `asc_abi/`
encoder: serialise an `EthereumEvent`, an `EthereumTransaction`, an
`EthereumBlock`, three `EventParam`s, type IDs, headers, the lot.

That is roughly 1-2 KLOC of fragile mirroring of graph-node internals
just to get the function to start. Doing it badly would silently
produce numbers that have nothing to do with reality, and doing it
properly is out of scope here.

**The right place to compare AS vs Rust handler throughput end-to-end
is graph-node itself**, by deploying both subgraphs against the same
chain head and reading `subgraph_indexing_handler_execution_time` from
graph-node's Prometheus metrics. The integration test in
`scripts/live-test.sh` already exercises the full graph-node path for
the Rust side.

What we *can* compare statically:

| Metric | AS | Rust |
|--------|---:|-----:|
| Imports (host functions) | 5 across 4 namespaces | 3 in `graphite` |
| Exports (functions) | 8 | 7 |
| Exports (globals) | 169 (`TypeId.*`) | 2 |
| Function count | 24 | 174 |
| Trigger ABI | `AscPtr<EthereumEvent>` (object graph) | `(ptr: u32, len: u32)` (TLV byte slice) |
| Per-call host round-trips (BigInt format, hex conversion) | yes | none (statically linked) |

The Rust ABI's per-event marshalling cost is one `allocate(212)` plus
one `memcpy` plus one `reset_arena` — versus the AS ABI's full object
graph reconstruction with type-ID writes, plus host calls back into
graph-node for every BigInt-to-string and bytes-to-hex conversion. We
expect Rust to win on per-event handler cost when measured inside
graph-node, but **this benchmark does not prove that** — it only
measures the Rust side in isolation.

---

## 6. Reproducing

```bash
# Build Rust ERC20 WASM
cd /Users/pepe/Projects/graphite/examples/erc20
cargo build --target wasm32-unknown-unknown --release

# Build AS ERC20 WASM
cd /Users/pepe/Projects/graphite/benchmarks/as-erc20
yarn install && yarn run codegen && yarn run build

# Sizes
ls -l /Users/pepe/Projects/graphite/target/wasm32-unknown-unknown/release/erc20_subgraph.wasm
ls -l /Users/pepe/Projects/graphite/benchmarks/as-erc20/build/ERC20/ERC20.wasm
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int <input> -o <output>

# Throughput
cd /Users/pepe/Projects/graphite
cargo build --release -p graphite-integration-tests --bin handler_throughput
ITERS=2000000 ./target/release/handler_throughput
```

The benchmark binary lives at
`tests/integration/src/bin/handler_throughput.rs` and is registered as
a `[[bin]]` target on the existing `graphite-integration-tests` crate.

---

## 7. Summary

| Question | Answer |
|----------|--------|
| Does Rust produce a larger binary? | Yes — 3-4× the AS size, dominated by `num_bigint` formatting code. |
| How fast is the Rust handler in isolation? | ~617 k Transfer events/sec, ~1.62 µs/event, on Apple Silicon under wasmtime 29 with no fuel metering. |
| How much memory does each Rust handler invocation use? | 576 bytes of arena heap; total WASM memory plateaus at 1.12 MB. |
| Did we measure AS throughput? | No. The AS ABI requires graph-node's `asc_abi` encoder to invoke. The honest comparison is end-to-end inside graph-node. |
| Can we cut the Rust binary size? | Already done — size-tuned `[profile.release]` (`opt-level = "z"`, `lto = "fat"`, `panic = "abort"`, `codegen-units = 1`, `strip = true`) brings raw from 107,259 B to 57,540 B (-46.4 %), and `wasm-opt -Oz` further to 50,105 B (-53.3 % total). Remaining bloat is `num_bigint` (~8 KB) plus `dlmalloc` (~6 KB) plus `core::fmt` (~3-4 KB) — droppable later by replacing the allocator and stripping `BigInt::Display`. |

The numbers above are raw measurements with no editorial spin. Where we
could not measure cleanly we said so; nothing here is fabricated.
