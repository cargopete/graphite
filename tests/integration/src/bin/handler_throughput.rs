//! Handler throughput benchmark.
//!
//! Loads the Rust ERC20 WASM module, feeds it the same Transfer event N times,
//! and measures wall-clock throughput (events/sec) plus per-handler heap usage.
//!
//! This measures the WASM handler in isolation: deserialise event TLV →
//! build entity → call store_set → reset arena. It does NOT measure
//! graph-node overhead, gas metering, network I/O, or DB writes.
//!
//! Usage:
//!   cargo run --release --bin handler_throughput -p graphite-integration-tests
//!   ITERS=2000000 cargo run --release --bin handler_throughput -p graphite-integration-tests

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use wasmtime::*;

const TRANSFER_SELECTOR: [u8; 32] = [
    0xdd, 0xf2, 0x52, 0xad, 0x1b, 0xe2, 0xc8, 0x9b, 0x69, 0xc2, 0xb0, 0x68, 0xfc, 0x37, 0x8d,
    0xaa, 0x95, 0x2b, 0xa7, 0xf1, 0x63, 0xc4, 0xa1, 0x16, 0x28, 0xf5, 0x5a, 0x4d, 0xf5, 0x23,
    0xb3, 0xef,
];

const FROM_ADDRESS: [u8; 20] = [
    0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00,
    0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
];

const TO_ADDRESS: [u8; 20] = [
    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
    0x11, 0x22, 0x33, 0x44, 0x55,
];

const CONTRACT_ADDRESS: [u8; 20] = [
    0xA0, 0xb8, 0x69, 0x91, 0xc6, 0x21, 0x8b, 0x36, 0xc1, 0xd1, 0x9D, 0x4a, 0x2e, 0x9E, 0xb0,
    0xcE, 0x36, 0x06, 0xeB, 0x48,
];

const TX_HASH: [u8; 32] = [0xAB; 32];
const BLOCK_NUMBER: u64 = 18_000_000;
const BLOCK_TIMESTAMP: u64 = 1_700_000_000;
const LOG_INDEX: u64 = 42;

/// Build trigger bytes in graph-node's RustLogTrigger format.
fn build_transfer_trigger() -> Vec<u8> {
    let mut buf = Vec::new();

    buf.extend_from_slice(&CONTRACT_ADDRESS);
    buf.extend_from_slice(&TX_HASH);
    buf.extend_from_slice(&LOG_INDEX.to_le_bytes());
    buf.extend_from_slice(&BLOCK_NUMBER.to_le_bytes());
    buf.extend_from_slice(&BLOCK_TIMESTAMP.to_le_bytes());

    buf.extend_from_slice(&3u32.to_le_bytes()); // 3 topics
    buf.extend_from_slice(&TRANSFER_SELECTOR);

    let mut from_topic = [0u8; 32];
    from_topic[12..].copy_from_slice(&FROM_ADDRESS);
    buf.extend_from_slice(&from_topic);

    let mut to_topic = [0u8; 32];
    to_topic[12..].copy_from_slice(&TO_ADDRESS);
    buf.extend_from_slice(&to_topic);

    // value = 1000 as uint256 big-endian
    let mut value_be = [0u8; 32];
    value_be[30] = 0x03;
    value_be[31] = 0xE8;
    buf.extend_from_slice(&32u32.to_le_bytes());
    buf.extend_from_slice(&value_be);

    buf
}

fn main() {
    let iters: u64 = std::env::var("ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500_000);
    let warmup: u64 = (iters / 100).max(1_000);

    let wasm_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/wasm32-unknown-unknown/release/erc20_subgraph.wasm"
    );
    let wasm_bytes = std::fs::read(wasm_path).expect(
        "WASM binary not found — run: cargo build --target wasm32-unknown-unknown --release -p erc20-subgraph",
    );

    let trigger_bytes = build_transfer_trigger();
    let trigger_len = trigger_bytes.len() as u32;

    // Counter for store_set calls — sanity check that the handler is actually running.
    let store_set_count = Arc::new(AtomicU64::new(0));

    // wasmtime engine — no fuel metering (we want raw handler cost).
    let engine = Engine::new(Config::new().async_support(false)).unwrap();
    let module = Module::new(&engine, &wasm_bytes).unwrap();

    let mut linker = Linker::new(&engine);

    // store_set: just bump the counter, don't actually copy memory (we're measuring handler, not memcpy).
    let counter_clone = store_set_count.clone();
    linker
        .func_wrap(
            "graphite",
            "store_set",
            move |_caller: Caller<'_, ()>,
                  _entity_type_ptr: u32,
                  _entity_type_len: u32,
                  _id_ptr: u32,
                  _id_len: u32,
                  _data_ptr: u32,
                  _data_len: u32| {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            },
        )
        .unwrap();

    // log_log: no-op
    linker
        .func_wrap(
            "graphite",
            "log_log",
            |_caller: Caller<'_, ()>, _level: u32, _ptr: u32, _len: u32| {},
        )
        .unwrap();

    // abort: trap if the handler panics — that would invalidate the benchmark
    linker
        .func_wrap(
            "graphite",
            "abort",
            |_caller: Caller<'_, ()>,
             _msg_ptr: u32,
             _msg_len: u32,
             _file_ptr: u32,
             _file_len: u32,
             _line: u32|
             -> Result<()> { Err(Error::msg("WASM handler called abort")) },
        )
        .unwrap();

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).unwrap();

    let memory = instance
        .get_memory(&mut store, "memory")
        .expect("missing memory export");
    let allocate = instance
        .get_typed_func::<u32, u32>(&mut store, "allocate")
        .expect("missing allocate export");
    let handle_transfer = instance
        .get_typed_func::<(u32, u32), u32>(&mut store, "handle_transfer")
        .expect("missing handle_transfer export");
    let reset_arena = instance
        .get_typed_func::<(), ()>(&mut store, "reset_arena")
        .expect("missing reset_arena export");
    let heap_usage = instance
        .get_typed_func::<(), u32>(&mut store, "heap_usage")
        .expect("missing heap_usage export");

    // Tight loop helper — replicates exactly what graph-node does per event:
    //   allocate → memcpy → handle_transfer → reset_arena
    let run_one = |store: &mut Store<()>| -> Result<()> {
        let ptr = allocate.call(&mut *store, trigger_len)?;
        memory.data_mut(&mut *store)[ptr as usize..(ptr + trigger_len) as usize]
            .copy_from_slice(&trigger_bytes);
        let rc = handle_transfer.call(&mut *store, (ptr, trigger_len))?;
        if rc != 0 {
            return Err(Error::msg(format!("handler returned non-zero: {rc}")));
        }
        reset_arena.call(&mut *store, ())?;
        Ok(())
    };

    println!("=== Rust ERC20 handler throughput ===");
    println!("WASM size: {} bytes", wasm_bytes.len());
    println!("Trigger size: {} bytes", trigger_len);
    println!("Warmup iterations: {}", warmup);
    println!("Measured iterations: {}", iters);
    println!();

    // Warm-up to let wasmtime finalise tier-up and cache the function pointers
    for _ in 0..warmup {
        run_one(&mut store).unwrap();
    }

    // After warmup: capture per-handler peak heap usage (single shot)
    let ptr = allocate.call(&mut store, trigger_len).unwrap();
    memory.data_mut(&mut store)[ptr as usize..(ptr + trigger_len) as usize]
        .copy_from_slice(&trigger_bytes);
    handle_transfer.call(&mut store, (ptr, trigger_len)).unwrap();
    let peak_per_handler = heap_usage.call(&mut store, ()).unwrap();
    reset_arena.call(&mut store, ()).unwrap();

    // Reset the store_set counter before the timed loop
    store_set_count.store(0, Ordering::Relaxed);

    // Timed loop
    let t0 = Instant::now();
    for _ in 0..iters {
        run_one(&mut store).unwrap();
    }
    let elapsed = t0.elapsed();

    let total_set = store_set_count.load(Ordering::Relaxed);
    let mem_pages = memory.size(&store);
    let mem_bytes = mem_pages * 64 * 1024;

    let secs = elapsed.as_secs_f64();
    let events_per_sec = (iters as f64) / secs;
    let nanos_per_event = elapsed.as_nanos() as f64 / (iters as f64);

    println!("--- Results ---");
    println!("Elapsed:               {:.3} s", secs);
    println!("Throughput:            {:.0} events/sec", events_per_sec);
    println!("Per event:             {:.0} ns", nanos_per_event);
    println!("store_set invocations: {}", total_set);
    println!();
    println!("--- Memory ---");
    println!(
        "Per-handler peak heap: {} bytes ({:.2} KB)",
        peak_per_handler,
        peak_per_handler as f64 / 1024.0
    );
    println!(
        "WASM memory.size():    {} pages ({} bytes / {:.2} MB)",
        mem_pages,
        mem_bytes,
        mem_bytes as f64 / (1024.0 * 1024.0)
    );

    // Sanity: if store_set wasn't called every iteration, results are bogus
    assert_eq!(
        total_set, iters,
        "store_set count mismatch — handler logic broken"
    );
}
