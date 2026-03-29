# Performance Comparison: Graphite (Rust) vs AssemblyScript

Comparison of equivalent ERC20 Transfer event indexing subgraphs.

*Date: 2026-03-29*

## Binary Size

| Metric | AssemblyScript | Rust (Graphite) | Ratio |
|--------|---------------|-----------------|-------|
| Raw WASM | 32.8 KB | 103.1 KB | 3.1× larger |
| wasm-opt -Oz | 18.8 KB | 75.3 KB | 4.0× larger |
| Code section | 8.1 KB | 83.9 KB | 10.4× |
| Data section | 5.3 KB | 8.2 KB | 1.5× |
| Functions | 24 | 153 | 6.4× |

Rust binaries are larger primarily due to `num_bigint` pulling in multiplication/division/formatting code (48% of the binary is `BigInt::Display::fmt` and its transitive dependencies). This is a known trade-off — the code is statically linked rather than delegated to host functions.

### Top size contributors (Rust)

| % of binary | Item |
|-------------|------|
| 12.4% | `num_bigint` multiplication (`mac3`) |
| 12.1% | Function name debug section |
| 7.7% | Read-only data (`.rodata`) |
| 5.8% | `num_bigint` radix conversion |
| 5.1% | `handle_transfer` (actual handler) |
| 4.7% | `dlmalloc` allocator |
| 3.9% | BTreeMap insert (entity storage) |

### Size reduction opportunities

- Strip `BigInt::Display` (saves ~48% — use a simpler formatting path)
- Strip debug function names (`--strip` flag, saves ~12%)
- Replace `dlmalloc` with the existing bump allocator for all allocations
- Use `wee_alloc` or remove std entirely

## Build Time

| Metric | AssemblyScript | Rust (Graphite) |
|--------|---------------|-----------------|
| Clean build | 1.8s | 11.4s |
| Incremental build | 1.8s | 0.4s |

Rust has a slower cold build (compiling all dependencies), but incremental builds are faster since only the changed crate is recompiled.

## Module Complexity

| Metric | AssemblyScript | Rust (Graphite) |
|--------|---------------|-----------------|
| Imports (host functions) | 5 | 3 |
| Exports (functions) | 8 | 5 |
| Exports (globals) | 169 | 2 |
| Import namespace | `env`, `conversion`, `numbers`, `index` | `graphite` (single namespace) |

The AS module exports 169 `TypeId.*` globals for runtime type identification — a consequence of AS's lack of generics. Rust needs none of this.

The Rust module imports only 3 host functions (the minimum needed for the handler: `store_set`, `log_log`, `abort`). AS imports 5, including `typeConversion` helpers that Rust handles natively.

## Runtime Performance

Graph-node measures handler execution time via `observe_handler_execution_time()` in `runtime/wasm/src/host.rs`. A proper runtime comparison requires deploying both subgraphs to the same graph-node instance indexing the same contract.

**Expected advantages for Rust:**
- No GC pauses (bump allocator with arena reset)
- No AscPtr indirection (direct ptr+len protocol)
- Native BigInt arithmetic (no host function round-trips for `bigIntToString`, etc.)
- Wasmtime optimises Rust WASM more effectively (standard WASM features, no AS-specific patterns)

**Expected disadvantages for Rust:**
- Larger WASM binary = slightly longer module compilation time on first load
- Fuel metering adds ~2-3× overhead vs no metering (but AS has equivalent parity_wasm gas injection)

*Runtime benchmarks pending — requires deploying both subgraphs to a running graph-node and comparing blocks/second indexing speed.*

## Developer Experience

| Aspect | AssemblyScript | Rust (Graphite) |
|--------|---------------|-----------------|
| Null safety | Runtime crashes | `Option<T>` at compile time |
| Testing | Requires graph-node or matchstick | `cargo test` with `MockHost` |
| Error messages | Opaque compiler crashes | Standard rustc errors |
| Closures | Not supported | Full support |
| Iterators | Limited | Full `Iterator` trait |
| Package ecosystem | npm (limited AS-compatible) | crates.io (full Rust ecosystem) |
| IDE support | VS Code only (limited) | rust-analyzer (excellent) |
| Debugging | Poor (WASM traps) | Panic messages with file + line |

## Summary

Rust subgraphs trade binary size for correctness, developer experience, and (expected) runtime performance. The 3× size increase is dominated by `num_bigint` formatting code that could be optimised away. For production subgraphs handling significant event volume, the runtime performance gains from zero-GC and native arithmetic are likely to outweigh the one-time module compilation cost.
