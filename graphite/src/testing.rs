//! Testing utilities for subgraph development.
//!
//! Provides `MockHost` for running handlers natively without WASM,
//! and builder patterns for constructing test events.

#[cfg(feature = "std")]
mod mock_host;

#[cfg(feature = "std")]
pub use mock_host::*;

/// Helper to create an Address from a hex string in tests.
///
/// # Panics
/// Panics if the hex string is invalid.
pub fn addr(hex: &str) -> crate::primitives::Address {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    // Pad or truncate to 20 bytes
    let padded = format!("{:0>40}", hex);
    let bytes: [u8; 20] = (0..20)
        .map(|i| u8::from_str_radix(&padded[i * 2..i * 2 + 2], 16).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    crate::primitives::Address::from(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addr_helper_full() {
        let a = addr("0xdead000000000000000000000000000000000000");
        assert_eq!(
            a,
            crate::primitives::Address::from([
                0xde, 0xad, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ])
        );
    }

    #[test]
    fn addr_helper_short() {
        // Short addresses get zero-padded on the left
        let a = addr("0xdead");
        assert_eq!(
            a,
            crate::primitives::Address::from([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xde, 0xad
            ])
        );
    }
}
