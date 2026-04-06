//! Integration test: load the ERC20 WASM module, invoke handle_transfer
//! with a serialized Transfer event, and verify store_set receives
//! the correct entity data.
//!
//! This replicates the exact flow graph-node takes:
//!   1. Serialize trigger as RustLogTrigger (TLV binary)
//!   2. Call allocate() in WASM to get memory
//!   3. Write trigger bytes to WASM memory
//!   4. Call handle_transfer(ptr, len)
//!   5. Capture store_set(entity_type, id, entity_data)
//!   6. Deserialize and verify entity fields

use std::sync::{Arc, Mutex};
use wasmtime::*;

// ============================================================================
// Test data: a Transfer(address,address,uint256) event
// ============================================================================

/// keccak256("Transfer(address,address,uint256)")
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

/// Build trigger bytes in the exact format graph-node's RustLogTrigger produces.
fn build_transfer_trigger(value_be: &[u8; 32]) -> Vec<u8> {
    let mut buf = Vec::new();

    // Fixed-size fields (same order as RustLogTrigger::write_to)
    buf.extend_from_slice(&CONTRACT_ADDRESS); // 20 bytes: address
    buf.extend_from_slice(&TX_HASH); // 32 bytes: tx_hash
    buf.extend_from_slice(&LOG_INDEX.to_le_bytes()); // 8 bytes: log_index
    buf.extend_from_slice(&BLOCK_NUMBER.to_le_bytes()); // 8 bytes: block_number
    buf.extend_from_slice(&BLOCK_TIMESTAMP.to_le_bytes()); // 8 bytes: block_timestamp

    // Topics: count + selector + from + to
    buf.extend_from_slice(&3u32.to_le_bytes()); // 3 topics

    // topic[0]: event selector
    buf.extend_from_slice(&TRANSFER_SELECTOR);

    // topic[1]: from address (left-padded to 32 bytes)
    let mut from_topic = [0u8; 32];
    from_topic[12..].copy_from_slice(&FROM_ADDRESS);
    buf.extend_from_slice(&from_topic);

    // topic[2]: to address (left-padded to 32 bytes)
    let mut to_topic = [0u8; 32];
    to_topic[12..].copy_from_slice(&TO_ADDRESS);
    buf.extend_from_slice(&to_topic);

    // Data: ABI-encoded uint256 value
    buf.extend_from_slice(&32u32.to_le_bytes()); // data length
    buf.extend_from_slice(value_be); // 32 bytes: uint256

    buf
}

// ============================================================================
// Captured store_set call
// ============================================================================

#[derive(Debug, Clone)]
struct StoredEntity {
    entity_type: String,
    id: String,
    data: Vec<u8>,
}

// ============================================================================
// Entity TLV deserialization (mirrors graph-node's rust_abi/entity.rs)
// ============================================================================

#[derive(Debug, PartialEq)]
enum EntityValue {
    String(String),
    Int(i32),
    BigInt(Vec<u8>), // signed big-endian bytes
    Bytes(Vec<u8>),
    Address([u8; 20]),
    Null,
    Other(u8), // tag we don't care about for this test
}

fn deserialize_entity_fields(data: &[u8]) -> Vec<(String, EntityValue)> {
    let mut fields = Vec::new();
    let mut pos = 0;

    // field_count: u32
    let count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    for _ in 0..count {
        // key_len: u32 + key: bytes
        let key_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let key = std::str::from_utf8(&data[pos..pos + key_len])
            .unwrap()
            .to_string();
        pos += key_len;

        // value tag
        let tag = data[pos];
        pos += 1;

        let value = match tag {
            0x00 => EntityValue::Null,
            0x01 => {
                // String
                let len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                pos += 4;
                let s = std::str::from_utf8(&data[pos..pos + len])
                    .unwrap()
                    .to_string();
                pos += len;
                EntityValue::String(s)
            }
            0x02 => {
                // Int
                let n = i32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
                pos += 4;
                EntityValue::Int(n)
            }
            0x04 => {
                // BigInt
                let len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                pos += 4;
                let bytes = data[pos..pos + len].to_vec();
                pos += len;
                EntityValue::BigInt(bytes)
            }
            0x07 => {
                // Bytes
                let len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                pos += 4;
                let bytes = data[pos..pos + len].to_vec();
                pos += len;
                EntityValue::Bytes(bytes)
            }
            0x08 => {
                // Address
                let mut addr = [0u8; 20];
                addr.copy_from_slice(&data[pos..pos + 20]);
                pos += 20;
                EntityValue::Address(addr)
            }
            other => {
                // Skip unknown: best effort
                EntityValue::Other(other)
            }
        };

        fields.push((key, value));
    }

    fields
}

// ============================================================================
// Test
// ============================================================================

#[test]
fn handle_transfer_end_to_end() {
    // Transfer value: 1000 (0x3E8) as big-endian uint256
    let mut value_be = [0u8; 32];
    value_be[30] = 0x03;
    value_be[31] = 0xE8;

    let trigger_bytes = build_transfer_trigger(&value_be);

    // Load WASM module
    let wasm_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/wasm32-unknown-unknown/release/erc20_subgraph.wasm"
    );
    let wasm_bytes = std::fs::read(wasm_path).expect(
        "WASM binary not found — run: cargo build --target wasm32-unknown-unknown --release -p erc20-subgraph",
    );

    let engine = Engine::new(Config::new().async_support(false)).unwrap();
    let module = Module::new(&engine, &wasm_bytes).unwrap();

    // Shared state for capturing store_set calls
    let captured: Arc<Mutex<Vec<StoredEntity>>> = Arc::new(Mutex::new(Vec::new()));

    let mut linker = Linker::new(&engine);

    // Link log_log — print to stdout for test visibility
    linker
        .func_wrap(
            "graphite",
            "log_log",
            |mut caller: Caller<'_, ()>, level: u32, msg_ptr: u32, msg_len: u32| {
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let data = mem.data(&caller);
                let msg = std::str::from_utf8(
                    &data[msg_ptr as usize..(msg_ptr + msg_len) as usize],
                )
                .unwrap_or("<invalid utf8>")
                .to_string();
                let level_str = match level {
                    0 => "CRITICAL",
                    1 => "ERROR",
                    2 => "WARNING",
                    3 => "INFO",
                    4 => "DEBUG",
                    _ => "LOG",
                };
                println!("[{}] {}", level_str, msg);
            },
        )
        .unwrap();

    // Link abort — called by the panic hook; treat as a test failure hint
    linker
        .func_wrap(
            "graphite",
            "abort",
            |mut caller: Caller<'_, ()>,
             msg_ptr: u32,
             msg_len: u32,
             _file_ptr: u32,
             _file_len: u32,
             _line: u32| {
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let data = mem.data(&caller);
                let msg = std::str::from_utf8(
                    &data[msg_ptr as usize..(msg_ptr + msg_len) as usize],
                )
                .unwrap_or("<invalid utf8>")
                .to_string();
                eprintln!("[ABORT] {}", msg);
                // Returning from abort causes an unreachable trap — that is correct
                // behaviour; wasmtime will surface it as a Trap::UnreachableCodeReached.
            },
        )
        .unwrap();

    // Link store_set — capture all calls
    let captured_clone = captured.clone();
    linker
        .func_wrap(
            "graphite",
            "store_set",
            move |mut caller: Caller<'_, ()>,
                  entity_type_ptr: u32,
                  entity_type_len: u32,
                  id_ptr: u32,
                  id_len: u32,
                  data_ptr: u32,
                  data_len: u32| {
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let data = mem.data(&caller);

                let entity_type = std::str::from_utf8(
                    &data[entity_type_ptr as usize..(entity_type_ptr + entity_type_len) as usize],
                )
                .unwrap()
                .to_string();

                let id =
                    std::str::from_utf8(&data[id_ptr as usize..(id_ptr + id_len) as usize])
                        .unwrap()
                        .to_string();

                let entity_data =
                    data[data_ptr as usize..(data_ptr + data_len) as usize].to_vec();

                captured_clone.lock().unwrap().push(StoredEntity {
                    entity_type,
                    id,
                    data: entity_data,
                });
            },
        )
        .unwrap();

    // Instantiate
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).unwrap();

    // Get exports
    let memory = instance
        .get_memory(&mut store, "memory")
        .expect("missing memory export");
    let allocate = instance
        .get_typed_func::<u32, u32>(&mut store, "allocate")
        .expect("missing allocate export");
    let handle_transfer = instance
        .get_typed_func::<(u32, u32), u32>(&mut store, "handle_transfer")
        .expect("missing handle_transfer export");

    // Allocate memory and write trigger bytes
    let trigger_len = trigger_bytes.len() as u32;
    let ptr = allocate.call(&mut store, trigger_len).unwrap();
    memory.data_mut(&mut store)[ptr as usize..(ptr + trigger_len) as usize]
        .copy_from_slice(&trigger_bytes);

    // Call the handler!
    let result = handle_transfer.call(&mut store, (ptr, trigger_len)).unwrap();
    assert_eq!(result, 0, "handler returned non-zero error code");

    // Verify store_set was called
    let entities = captured.lock().unwrap();
    assert_eq!(entities.len(), 1, "expected exactly 1 store_set call");

    let stored = &entities[0];
    assert_eq!(stored.entity_type, "Transfer");

    // Deserialize and check entity fields
    let fields = deserialize_entity_fields(&stored.data);
    let field_map: std::collections::HashMap<&str, &EntityValue> =
        fields.iter().map(|(k, v)| (k.as_str(), v)).collect();

    // Check ID is present and non-empty
    assert!(stored.id.len() > 0, "entity ID should be non-empty");

    // Check 'from' field — should be the FROM_ADDRESS as Bytes
    match field_map.get("from") {
        Some(EntityValue::Bytes(b)) => {
            assert_eq!(b.as_slice(), &FROM_ADDRESS, "from address mismatch");
        }
        other => panic!("expected 'from' as Bytes, got: {:?}", other),
    }

    // Check 'to' field — should be the TO_ADDRESS as Bytes
    match field_map.get("to") {
        Some(EntityValue::Bytes(b)) => {
            assert_eq!(b.as_slice(), &TO_ADDRESS, "to address mismatch");
        }
        other => panic!("expected 'to' as Bytes, got: {:?}", other),
    }

    // Check 'value' field — should be BigInt(1000)
    match field_map.get("value") {
        Some(EntityValue::BigInt(bytes)) => {
            // 1000 in signed little-endian = [0xE8, 0x03]
            assert_eq!(bytes, &[0xE8, 0x03], "value should be 1000");
        }
        other => panic!("expected 'value' as BigInt, got: {:?}", other),
    }

    // Check 'blockNumber' field — should be BigInt(18_000_000)
    match field_map.get("blockNumber") {
        Some(EntityValue::BigInt(bytes)) => {
            let val = graphite::primitives::BigInt::from_signed_bytes_le(bytes);
            assert_eq!(val, graphite::primitives::BigInt::from(BLOCK_NUMBER));
        }
        other => panic!("expected 'blockNumber' as BigInt, got: {:?}", other),
    }

    // Check 'timestamp' field — should be BigInt(1_700_000_000)
    match field_map.get("timestamp") {
        Some(EntityValue::BigInt(bytes)) => {
            let val = graphite::primitives::BigInt::from_signed_bytes_le(bytes);
            assert_eq!(val, graphite::primitives::BigInt::from(BLOCK_TIMESTAMP));
        }
        other => panic!("expected 'timestamp' as BigInt, got: {:?}", other),
    }

    // Check 'transactionHash' field — should be the TX_HASH as Bytes
    match field_map.get("transactionHash") {
        Some(EntityValue::Bytes(b)) => {
            assert_eq!(b.as_slice(), &TX_HASH, "transactionHash mismatch");
        }
        other => panic!("expected 'transactionHash' as Bytes, got: {:?}", other),
    }

    println!("=== Integration test passed! ===");
    println!("Entity type: {}", stored.entity_type);
    println!("Entity ID: {}", stored.id);
    println!("Fields stored: {}", fields.len());
    for (key, value) in &fields {
        println!("  {} = {:?}", key, value);
    }
}
