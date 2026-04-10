//! Panic handler for no_std WASM builds.
//!
//! Routes Rust panics through graph-node's `log.log` host function (at ERROR
//! level) so the panic message appears in the indexer logs, then calls
//! `abort` to halt the handler.
//!
//! # Message encoding
//!
//! We write the panic message into a fixed 512-byte stack buffer (UTF-8),
//! then build an AS string in a fresh allocation and pass its pointer to
//! `log.log`. This avoids re-entrant heap operations as much as possible.

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Build an error message into a fixed stack buffer — no heap needed.
    let mut buf = [0u8; 512];
    let msg = fmt_panic(info, &mut buf);

    // Log via graph-node before aborting so it appears in indexer logs.
    log_error(msg);

    unsafe {
        crate::ffi::abort(0, 0, 0, 0);
    }

    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable();

    #[cfg(not(target_arch = "wasm32"))]
    loop {}
}

/// Write a compact panic description into `buf` and return the filled slice.
fn fmt_panic<'a>(info: &PanicInfo<'_>, buf: &'a mut [u8; 512]) -> &'a str {
    let mut pos = 0usize;
    let prefix = b"[graphite panic] ";
    let copy_len = prefix.len().min(buf.len());
    buf[..copy_len].copy_from_slice(&prefix[..copy_len]);
    pos += copy_len;

    // Append the payload message if available.
    if let Some(msg) = info.message().as_str() {
        let bytes = msg.as_bytes();
        let avail = buf.len().saturating_sub(pos);
        let copy_len = bytes.len().min(avail);
        buf[pos..pos + copy_len].copy_from_slice(&bytes[..copy_len]);
        pos += copy_len;
    }

    // location: " @ file:line"
    if let Some(loc) = info.location() {
        let at = b" @ ";
        if pos + at.len() < buf.len() {
            buf[pos..pos + at.len()].copy_from_slice(at);
            pos += at.len();
        }
        let file = loc.file().as_bytes();
        let avail = buf.len().saturating_sub(pos + 8); // leave room for ":NNN"
        let copy_len = file.len().min(avail);
        buf[pos..pos + copy_len].copy_from_slice(&file[..copy_len]);
        pos += copy_len;

        // append ":line"
        if pos + 1 < buf.len() {
            buf[pos] = b':';
            pos += 1;
            let line = loc.line();
            let mut tmp = [0u8; 10];
            let n = fmt_u32(line, &mut tmp);
            let avail = buf.len().saturating_sub(pos);
            let copy_len = n.min(avail);
            buf[pos..pos + copy_len].copy_from_slice(&tmp[..copy_len]);
            pos += copy_len;
        }
    }

    // Safety: we only wrote valid UTF-8 bytes above.
    core::str::from_utf8(&buf[..pos]).unwrap_or("[graphite panic]")
}

/// Format a u32 as decimal ASCII into `buf`, return byte count written.
fn fmt_u32(mut n: u32, buf: &mut [u8; 10]) -> usize {
    if n == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut i = buf.len();
    while n > 0 && i > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    let len = buf.len() - i;
    buf.copy_within(i.., 0);
    len
}

/// Log `msg` at ERROR level via graph-node's `log.log` host function.
///
/// Builds an AS string in a fresh allocation — safe to call from panic
/// context as long as the allocator hasn't itself panicked.
fn log_error(msg: &str) {
    // Build an AscString from the message bytes.
    let str_ptr = crate::as_types::new_asc_string(msg);
    unsafe {
        crate::ffi::log_log(crate::ffi::LOG_ERROR, str_ptr);
    }
}

/// The `abort` export that AS runtimes call when *they* encounter an error.
/// Graph-node imports this from the WASM module under the name "abort".
///
/// Reads the message AscString (if non-null), logs it at ERROR level,
/// then forwards to the host abort.
#[unsafe(no_mangle)]
pub extern "C" fn abort(msg: u32, file: u32, line: u32, col: u32) -> ! {
    // Log the AS abort message if a pointer was provided.
    if msg != 0 {
        let s = unsafe { crate::store_read::read_asc_string(msg) };
        if !s.is_empty() {
            let mut buf = [0u8; 512];
            let prefix = b"[abort] ";
            let plen = prefix.len().min(buf.len());
            buf[..plen].copy_from_slice(&prefix[..plen]);
            let sbytes = s.as_bytes();
            let avail = buf.len().saturating_sub(plen);
            let copy_len = sbytes.len().min(avail);
            buf[plen..plen + copy_len].copy_from_slice(&sbytes[..copy_len]);
            if let Ok(text) = core::str::from_utf8(&buf[..plen + copy_len]) {
                log_error(text);
            }
        }
    }

    unsafe {
        crate::ffi::abort(msg, file, line, col);
    }

    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable();

    #[cfg(not(target_arch = "wasm32"))]
    loop {}
}
