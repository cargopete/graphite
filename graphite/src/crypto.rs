//! Cryptographic utilities.
//!
//! `keccak256` works natively (using `alloy_primitives`) without calling the
//! graph-node host, so it is usable inside any context including WASM builds.

/// Compute the Keccak-256 hash of `data`.
///
/// Uses `alloy_primitives::keccak256` — no host call required.
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    alloy_primitives::keccak256(data).0
}

/// Return the 4-byte ABI function selector for a canonical Solidity signature.
///
/// The selector is the first 4 bytes of `keccak256(signature)`.
///
/// # Example
///
/// ```rust
/// use graphite::crypto::selector;
/// let sel = selector("transfer(address,uint256)");
/// assert_eq!(sel, [0xa9, 0x05, 0x9c, 0xbb]);
/// ```
pub fn selector(signature: &str) -> [u8; 4] {
    // Strip return type if present: "fn()(ret)" → "fn()"
    let input_sig = strip_return_type(signature);
    let h = keccak256(input_sig.as_bytes());
    [h[0], h[1], h[2], h[3]]
}

/// Strip the return-type suffix from a Solidity signature.
///
/// `"latestAnswer()(int256)"` → `"latestAnswer()"`.
/// Signatures without a return type are returned unchanged.
fn strip_return_type(sig: &str) -> &str {
    // Find ")(" pattern that marks the boundary between input and return types.
    // We need the *last* closing paren of the input argument list, so we track
    // paren depth.
    let bytes = sig.as_bytes();
    let mut depth: i32 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    // This is the closing paren of the input list.
                    return &sig[..i + 1];
                }
            }
            _ => {}
        }
    }
    sig
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keccak256_hello() {
        let h = keccak256(b"hello");
        assert_eq!(
            hex::encode(h),
            "1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
    }

    #[test]
    fn selector_transfer() {
        // Known selector for transfer(address,uint256)
        assert_eq!(selector("transfer(address,uint256)"), [0xa9, 0x05, 0x9c, 0xbb]);
    }

    #[test]
    fn selector_strips_return_type() {
        // "latestAnswer()(int256)" and "latestAnswer()" should give same selector
        assert_eq!(selector("latestAnswer()(int256)"), selector("latestAnswer()"));
    }

    #[test]
    fn strip_return_type_basic() {
        assert_eq!(strip_return_type("latestAnswer()(int256)"), "latestAnswer()");
        assert_eq!(strip_return_type("balanceOf(address)"), "balanceOf(address)");
        assert_eq!(strip_return_type("swap((uint256,uint256),address)(uint256)"), "swap((uint256,uint256),address)");
    }
}
