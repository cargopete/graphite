//! Memory allocation for graph-node WASM runtime.
//!
//! Simple bump allocator with arena reset after each handler invocation.
//! No AssemblyScript memory layout - just raw bytes.

use core::sync::atomic::{AtomicU32, Ordering};

/// Start of heap (after 64KB reserved for stack/static data).
const HEAP_START: u32 = 0x10000;

/// Current allocation pointer.
static HEAP_PTR: AtomicU32 = AtomicU32::new(HEAP_START);

/// Maximum heap size (4MB — leaves room for stack in a 16MB WASM memory).
const HEAP_MAX: u32 = 4 * 1024 * 1024;

/// Allocate memory from the bump allocator.
///
/// Exported for graph-node to call when it needs to write data into WASM memory.
/// Aborts if allocation would exceed the heap limit.
#[unsafe(no_mangle)]
pub extern "C" fn allocate(size: u32) -> u32 {
    // Align to 8 bytes
    let aligned_size = (size + 7) & !7;
    let ptr = HEAP_PTR.fetch_add(aligned_size, Ordering::SeqCst);
    if ptr + aligned_size > HEAP_START + HEAP_MAX {
        // Roll back and abort — we're out of memory
        HEAP_PTR.fetch_sub(aligned_size, Ordering::SeqCst);
        core::arch::wasm32::unreachable();
    }
    ptr
}

/// Reset the arena allocator.
///
/// Called by graph-node after each handler completes to reclaim memory.
#[unsafe(no_mangle)]
pub extern "C" fn reset_arena() {
    HEAP_PTR.store(HEAP_START, Ordering::SeqCst);
}

/// Get current heap usage (for debugging/metrics).
#[unsafe(no_mangle)]
pub extern "C" fn heap_usage() -> u32 {
    HEAP_PTR.load(Ordering::SeqCst) - HEAP_START
}

// ============================================================================
// Internal allocation helpers
// ============================================================================

/// Allocate and copy a byte slice, returning the pointer.
pub fn alloc_slice(data: &[u8]) -> u32 {
    let ptr = allocate(data.len() as u32);
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
    }
    ptr
}

/// Allocate and copy a string, returning the pointer.
pub fn alloc_str(s: &str) -> u32 {
    alloc_slice(s.as_bytes())
}
