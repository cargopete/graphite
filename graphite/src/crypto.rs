//! Cryptographic utilities.
//!
//! All functions in this module run natively — no graph-node host call required.
//! They work in `cargo test` without Docker and in WASM builds alike.

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

/// Compute the SHA-256 hash of `data`.
///
/// Native implementation — no graph-node host call.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute the SHA3-256 (NIST Keccak variant) hash of `data`.
///
/// Note: this is SHA3-256, not Keccak-256. For Ethereum's Keccak-256 use `keccak256`.
/// Native implementation — no graph-node host call.
pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    use sha3::Digest;
    let mut hasher = sha3::Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute the SHA3-512 hash of `data`.
///
/// Native implementation — no graph-node host call.
pub fn sha3_512(data: &[u8]) -> [u8; 64] {
    use sha3::Digest;
    let mut hasher = sha3::Sha3_512::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Recover the Ethereum address from a secp256k1 signature.
///
/// - `message_hash`: 32-byte Keccak-256 hash of the signed message.
/// - `r`: 32-byte signature component r.
/// - `s`: 32-byte signature component s.
/// - `v`: recovery id — 0 or 1 (or 27/28 for legacy Ethereum signatures).
///
/// Returns `Some([u8; 20])` with the signer's address, or `None` if recovery fails.
///
/// Native implementation — no graph-node host call.
pub fn secp256k1_recover(message_hash: &[u8; 32], r: &[u8; 32], s: &[u8; 32], v: u8) -> Option<[u8; 20]> {
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    // Build the signature from r and s bytes.
    let mut sig_bytes = [0u8; 64];
    sig_bytes[..32].copy_from_slice(r);
    sig_bytes[32..].copy_from_slice(s);
    let sig = Signature::from_bytes((&sig_bytes).into()).ok()?;

    // Normalise v to a recovery id (0 or 1).
    let recovery_id = match v {
        0 | 1 => RecoveryId::new(v != 0, false),
        27 => RecoveryId::new(false, false),
        28 => RecoveryId::new(true, false),
        _ => return None,
    };

    // Recover the verifying key.
    let vk = VerifyingKey::recover_from_prehash(message_hash, &sig, recovery_id).ok()?;

    // Derive the Ethereum address: keccak256(uncompressed_pubkey[1..])[12..]
    let uncompressed = vk.to_encoded_point(false);
    let pub_bytes = &uncompressed.as_bytes()[1..]; // strip 0x04 prefix
    let hash = keccak256(pub_bytes);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[12..]);
    Some(addr)
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

    #[test]
    fn sha256_hello() {
        let h = sha256(b"hello");
        assert_eq!(
            hex::encode(h),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn sha3_256_empty() {
        let h = sha3_256(b"");
        // Known SHA3-256 of empty string
        assert_eq!(
            hex::encode(h),
            "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
        );
    }

    #[test]
    fn sha3_512_empty() {
        let h = sha3_512(b"");
        // Known SHA3-512 of empty string
        assert_eq!(
            hex::encode(&h[..]),
            "a69f73cca23a9ac5c8b567dc185a756e97c982164fe25859e0d1dcc1475c80a615b2123af1f5f94c11e3e9402c3ac558f500199d95b6d3e301758586281dcd26"
        );
    }
}
