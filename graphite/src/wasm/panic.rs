//! Panic hook for WASM subgraph builds.
//!
//! Replaces the default std panic hook with one that routes panic messages
//! through graph-node's `abort` host function, so the error message, file,
//! and line number surface in graph-node logs instead of a generic WASM trap.

use crate::wasm::ffi;
use alloc::format;

/// Install a panic hook that forwards panic info to graph-node via `abort`.
///
/// Called automatically by the `#[handler]` macro wrapper before invoking
/// the user's handler function.
pub fn install() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        std::panic::set_hook(std::boxed::Box::new(|info| {
            let message = format!("panic: {}", info);
            let (file, line) = match info.location() {
                Some(loc) => (loc.file(), loc.line()),
                None => ("<unknown>", 0),
            };
            unsafe {
                ffi::abort(
                    message.as_ptr() as u32,
                    message.len() as u32,
                    file.as_ptr() as u32,
                    file.len() as u32,
                    line,
                );
            }
        }));
    });
}
