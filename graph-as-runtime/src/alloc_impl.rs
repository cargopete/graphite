//! Bump allocator with AssemblyScript object header layout.
//!
//! Every heap-allocated AS object has a 16-byte header immediately before the
//! data pointer that graph-node reads through AscPtr:
//!
//! ```text
//! offset  size  field
//! -4       4    mm_info    (allocator bookkeeping — we write 0)
//! -8       4    gc_info    (GC flags — we write 0; we don't have a GC)
//! -12      4    rt_id      (class ID from class_ids module)
//! -16      4    rt_size    (payload byte count — NOT including the header)
//! ```
//!
//! The pointer returned to callers (and stored in AscPtr fields) points at
//! byte 0 of the payload, i.e. 16 bytes after the allocation start.
//!
//! # Global allocator
//!
//! We provide `#[global_allocator]` so that any `Box`/`Vec`/`String` in this
//! crate (or downstream) goes through our bump allocator rather than dlmalloc.
//! LLD provides the `__heap_base` symbol on wasm32-unknown-unknown, pointing
//! just past the data segment.
//!
//! # Exports
//!
//! graph-node calls `__new(size, id)` to allocate AS objects. We export that
//! function using the exact signature AS emits.

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicU32, Ordering};

// ============================================================================
// Heap base from LLD
// ============================================================================

unsafe extern "C" {
    /// Provided by LLD — first byte past the data segment.
    static __heap_base: u32;
}

// ============================================================================
// Bump allocator state
// ============================================================================

/// Current bump pointer. Zero means "not yet initialised".
static BUMP_PTR: AtomicU32 = AtomicU32::new(0);

/// Lazy-initialise and return the current bump pointer.
fn bump_ptr() -> u32 {
    let p = BUMP_PTR.load(Ordering::Relaxed);
    if p != 0 {
        return p;
    }
    // First call — seed from __heap_base.
    let base = unsafe { __heap_base };
    // Align to 16.
    let aligned = (base + 15) & !15;
    BUMP_PTR.store(aligned, Ordering::Relaxed);
    aligned
}

/// Bump-allocate `bytes` bytes, aligned to `align` (must be power-of-two).
/// Returns the pointer or 0 on overflow.
fn raw_alloc(bytes: u32, align: u32) -> u32 {
    // Use a CAS loop so this is safe if somehow called from multiple contexts
    // (wasm32 is single-threaded, but let's be tidy).
    loop {
        let current = BUMP_PTR.load(Ordering::Relaxed);
        let ptr = if current == 0 { bump_ptr() } else { current };

        let aligned = (ptr + align - 1) & !(align - 1);
        let next = aligned.checked_add(bytes).unwrap_or(0);
        if next == 0 {
            // Overflow — unreachable trap.
            #[cfg(target_arch = "wasm32")]
            core::arch::wasm32::unreachable();
            #[cfg(not(target_arch = "wasm32"))]
            panic!("bump allocator overflow");
        }
        if BUMP_PTR
            .compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            return aligned;
        }
    }
}

// ============================================================================
// GlobalAlloc impl
// ============================================================================

pub struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align() as u32;
        let size = layout.size() as u32;
        let ptr = raw_alloc(size, align);
        ptr as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator — no individual dealloc. Memory is reclaimed
        // by graph-node calling reset_arena() after each handler.
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

// ============================================================================
// AS object allocation
// ============================================================================

/// Header size: 16 bytes (4 fields × 4 bytes each).
const HEADER_SIZE: u32 = 16;

/// Allocate an AS object with the given class ID and payload size.
///
/// Writes the 16-byte AS header and returns a pointer to the payload.
/// The returned pointer is what gets stored in AscPtr<T> fields.
pub fn alloc_as_obj(rt_id: u32, payload_bytes: u32) -> u32 {
    // Total allocation: header + payload, aligned to 16.
    let total = HEADER_SIZE + payload_bytes;
    let base = raw_alloc(total, 16);

    // Write header at base (little-endian, wasm32 is always LE).
    let hdr = base as *mut u32;
    unsafe {
        // offset 0: mm_info (allocator bookkeeping)
        hdr.write(0);
        // offset 4: gc_info (GC flags)
        hdr.add(1).write(0);
        // offset 8: rt_id (class ID)
        hdr.add(2).write(rt_id);
        // offset 12: rt_size (payload byte count)
        hdr.add(3).write(payload_bytes);
    }

    // Return pointer to payload (past the header).
    base + HEADER_SIZE
}

// ============================================================================
// graph-node / AS runtime exports
// ============================================================================

/// `__new(size: usize, id: u32) -> usize`
///
/// Exported with the exact name and signature that AssemblyScript emits.
/// graph-node calls this to allocate strings and other AS objects.
#[unsafe(no_mangle)]
pub extern "C" fn __new(size: u32, id: u32) -> u32 {
    alloc_as_obj(id, size)
}

/// `__pin(ptr: usize) -> usize` — no-op; we have no GC.
#[unsafe(no_mangle)]
pub extern "C" fn __pin(ptr: u32) -> u32 {
    ptr
}

/// `__unpin(ptr: usize)` — no-op.
#[unsafe(no_mangle)]
pub extern "C" fn __unpin(_ptr: u32) {}

/// `__collect()` — no-op; no GC cycle collection.
#[unsafe(no_mangle)]
pub extern "C" fn __collect() {}

/// Reset the bump allocator back to `__heap_base`.
///
/// graph-node (or the test harness) calls this after each handler completes
/// to reclaim all per-handler memory in O(1).
#[unsafe(no_mangle)]
pub extern "C" fn reset_arena() {
    let base = unsafe { __heap_base };
    let aligned = (base + 15) & !15;
    BUMP_PTR.store(aligned, Ordering::Relaxed);
}

/// Return current heap usage in bytes (for debugging).
#[unsafe(no_mangle)]
pub extern "C" fn heap_usage() -> u32 {
    let base = unsafe { __heap_base };
    let ptr = BUMP_PTR.load(Ordering::Relaxed);
    if ptr == 0 { 0 } else { ptr.saturating_sub(base) }
}

// ============================================================================
// Raw byte allocation helpers (used by as_types module)
// ============================================================================

/// Allocate `n` bytes with 8-byte alignment. Returns the pointer.
pub fn alloc_bytes(n: u32) -> u32 {
    raw_alloc(n, 8)
}

/// Write a byte slice to freshly allocated memory. Returns the pointer.
pub fn alloc_copy(data: &[u8]) -> u32 {
    let ptr = alloc_bytes(data.len() as u32);
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
    }
    ptr
}
