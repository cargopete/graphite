//! Memory allocation for graph-node WASM runtime.
//!
//! Graph-node uses AssemblyScript-style memory layout. We need to provide
//! allocation functions that match this layout.
//!
//! AS memory layout for objects:
//! - Bytes 0-3: Runtime type ID
//! - Bytes 4-7: Runtime size
//! - Bytes 8+: Data
//!
//! For now, we use a simple bump allocator. A more sophisticated
//! implementation might use a proper arena or integrate with graph-node's
//! memory management.

use core::sync::atomic::{AtomicU32, Ordering};

/// Simple bump allocator state.
/// We start allocating after a reserved area for static data.
static HEAP_OFFSET: AtomicU32 = AtomicU32::new(0x10000); // Start at 64KB

/// Allocate memory for graph-node.
/// Returns a pointer to the allocated memory (after the AS header).
#[unsafe(no_mangle)]
pub extern "C" fn allocate(size: u32) -> u32 {
    // AS header is 8 bytes
    const HEADER_SIZE: u32 = 8;
    let total_size = HEADER_SIZE + size;

    // Align to 8 bytes
    let aligned_size = (total_size + 7) & !7;

    // Bump allocate
    let ptr = HEAP_OFFSET.fetch_add(aligned_size, Ordering::SeqCst);

    // Return pointer to data (after header)
    ptr + HEADER_SIZE
}

/// Deallocate memory (no-op for bump allocator).
#[unsafe(no_mangle)]
pub extern "C" fn deallocate(_ptr: u32, _size: u32) {
    // Bump allocator doesn't actually free
    // In a real implementation, we might track allocations
}

/// Write AS string header and return data pointer.
/// type_id 1 = String in AS runtime
pub fn alloc_string(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let ptr = allocate(bytes.len() as u32);

    // Write header (ptr - 8 is the header location)
    let header_ptr = ptr - 8;
    unsafe {
        // Type ID (String = 1)
        core::ptr::write((header_ptr) as *mut u32, 1);
        // Size in bytes
        core::ptr::write((header_ptr + 4) as *mut u32, bytes.len() as u32);
        // Data
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
    }

    ptr
}

/// Read a string from AS memory layout.
pub unsafe fn read_string(ptr: u32) -> &'static str {
    if ptr == 0 {
        return "";
    }

    let header_ptr = ptr - 8;
    unsafe {
        let size = core::ptr::read((header_ptr + 4) as *const u32) as usize;
        let data = core::slice::from_raw_parts(ptr as *const u8, size);
        core::str::from_utf8_unchecked(data)
    }
}

/// Write AS bytes header and return data pointer.
/// type_id 2 = Bytes in AS runtime
pub fn alloc_bytes(bytes: &[u8]) -> u32 {
    let ptr = allocate(bytes.len() as u32);

    let header_ptr = ptr - 8;
    unsafe {
        // Type ID (Bytes = 2)
        core::ptr::write((header_ptr) as *mut u32, 2);
        // Size
        core::ptr::write((header_ptr + 4) as *mut u32, bytes.len() as u32);
        // Data
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
    }

    ptr
}

/// Read bytes from AS memory layout.
pub unsafe fn read_bytes(ptr: u32) -> &'static [u8] {
    if ptr == 0 {
        return &[];
    }

    unsafe {
        let header_ptr = ptr - 8;
        let size = core::ptr::read((header_ptr + 4) as *const u32) as usize;
        core::slice::from_raw_parts(ptr as *const u8, size)
    }
}
