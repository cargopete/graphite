//! Panic handler for no_std WASM builds.
//!
//! Routes Rust panics through the AS `abort` host function so graph-node sees
//! a structured error rather than an opaque WASM trap.
//!
//! The AS abort signature is: `abort(msg: u32, file: u32, line: u32, col: u32)`
//! where each u32 is an AscPtr<AscString> (or 0).
//!
//! We pass 0 for everything — keeping the panic handler minimal avoids any
//! risk of re-entrant allocation failures at panic time. If you need richer
//! panic messages, build the AscString before calling abort.

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Forward to the AS abort export with zero pointers.
    // This produces a graph-node log entry indicating the handler aborted.
    unsafe {
        crate::ffi::abort(0, 0, 0, 0);
    }

    // abort() is ! in AS semantics; if graph-node somehow returns, trap.
    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable();

    #[cfg(not(target_arch = "wasm32"))]
    loop {}
}

/// The `abort` export that AS runtimes call when *they* encounter an error.
/// Graph-node imports this from the WASM module under the name "abort".
///
/// We just forward to the host's abort.
#[unsafe(no_mangle)]
pub extern "C" fn abort(msg: u32, file: u32, line: u32, col: u32) -> ! {
    unsafe {
        crate::ffi::abort(msg, file, line, col);
    }

    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable();

    #[cfg(not(target_arch = "wasm32"))]
    loop {}
}
