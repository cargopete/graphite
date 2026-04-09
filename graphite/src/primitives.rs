//! Core primitive types for subgraph development.
//!
//! These types wrap and extend `alloy-primitives` with additional
//! functionality needed for subgraph mappings.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::ops::{Add, Div, Mul, Rem, Sub};
use num_traits::Signed;

// Re-export alloy primitives
pub use alloy_primitives::{Address, B256, U256};

/// Arbitrary-length byte array.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self(slice.to_vec())
    }

    pub fn from_hex(s: &str) -> Result<Self, BytesError> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let bytes = (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| BytesError::InvalidHex)?;
        Ok(Self(bytes))
    }

    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(2 + self.0.len() * 2);
        s.push_str("0x");
        for byte in &self.0 {
            use core::fmt::Write;
            write!(s, "{:02x}", byte).unwrap();
        }
        s
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert to owned Vec<u8>.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<&[u8]> for Bytes {
    fn from(slice: &[u8]) -> Self {
        Self::from_slice(slice)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BytesError {
    #[error("invalid hex string")]
    InvalidHex,
}

/// Arbitrary precision signed integer.
///
/// Wraps `num_bigint::BigInt` and provides operator overloading for ergonomic arithmetic.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BigInt(num_bigint::BigInt);

impl BigInt {
    pub fn zero() -> Self {
        Self(num_bigint::BigInt::ZERO)
    }

    pub fn one() -> Self {
        Self(num_bigint::BigInt::from(1))
    }

    pub fn from_signed_bytes_be(bytes: &[u8]) -> Self {
        Self(num_bigint::BigInt::from_signed_bytes_be(bytes))
    }

    pub fn to_signed_bytes_be(&self) -> Vec<u8> {
        self.0.to_signed_bytes_be()
    }

    pub fn to_signed_bytes_le(&self) -> Vec<u8> {
        self.0.to_signed_bytes_le()
    }

    pub fn from_signed_bytes_le(bytes: &[u8]) -> Self {
        Self(num_bigint::BigInt::from_signed_bytes_le(bytes))
    }

    pub fn pow(&self, exp: u32) -> Self {
        Self(self.0.pow(exp))
    }

    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    pub fn is_zero(&self) -> bool {
        self.0 == num_bigint::BigInt::ZERO
    }

    pub fn is_negative(&self) -> bool {
        self.0 < num_bigint::BigInt::ZERO
    }

    /// Create from unsigned big-endian bytes.
    pub fn from_unsigned_bytes_be(bytes: &[u8]) -> Self {
        Self(num_bigint::BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            bytes,
        ))
    }

    /// Try to convert to u64. Returns None if out of range.
    pub fn to_u64(&self) -> Option<u64> {
        use num_traits::ToPrimitive;
        self.0.to_u64()
    }

    /// Try to convert to i64. Returns None if out of range.
    pub fn to_i64(&self) -> Option<i64> {
        use num_traits::ToPrimitive;
        self.0.to_i64()
    }
}

impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for BigInt {
    fn from(n: i32) -> Self {
        Self(num_bigint::BigInt::from(n))
    }
}

impl From<i64> for BigInt {
    fn from(n: i64) -> Self {
        Self(num_bigint::BigInt::from(n))
    }
}

impl From<u64> for BigInt {
    fn from(n: u64) -> Self {
        Self(num_bigint::BigInt::from(n))
    }
}

impl From<U256> for BigInt {
    fn from(n: U256) -> Self {
        Self(num_bigint::BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            &n.to_be_bytes::<32>(),
        ))
    }
}

impl Add for BigInt {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for BigInt {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for BigInt {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for BigInt {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Add for &BigInt {
    type Output = BigInt;
    fn add(self, rhs: Self) -> Self::Output {
        BigInt(&self.0 + &rhs.0)
    }
}

impl Sub for &BigInt {
    type Output = BigInt;
    fn sub(self, rhs: Self) -> Self::Output {
        BigInt(&self.0 - &rhs.0)
    }
}

impl Mul for &BigInt {
    type Output = BigInt;
    fn mul(self, rhs: Self) -> Self::Output {
        BigInt(&self.0 * &rhs.0)
    }
}

impl Div for &BigInt {
    type Output = BigInt;
    fn div(self, rhs: Self) -> Self::Output {
        BigInt(&self.0 / &rhs.0)
    }
}

impl Rem for BigInt {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl Rem for &BigInt {
    type Output = BigInt;
    fn rem(self, rhs: Self) -> Self::Output {
        BigInt(&self.0 % &rhs.0)
    }
}

/// Arbitrary precision decimal number.
///
/// For financial calculations where precision matters.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BigDecimal {
    /// The unscaled integer value
    value: BigInt,
    /// Number of decimal places (negative exponent)
    scale: i64,
}

/// Precision used for BigDecimal division results (decimal places).
const BIGDEC_DIV_PRECISION: u32 = 18;

/// Align two BigDecimal values to the same scale, returning raw inner values.
fn align_bigdec(
    a: &BigDecimal,
    b: &BigDecimal,
) -> (num_bigint::BigInt, num_bigint::BigInt, i64) {
    let av = a.value.0.clone();
    let bv = b.value.0.clone();
    if a.scale == b.scale {
        return (av, bv, a.scale);
    }
    if a.scale > b.scale {
        let factor = num_bigint::BigInt::from(10i64).pow((a.scale - b.scale) as u32);
        (av, bv * factor, a.scale)
    } else {
        let factor = num_bigint::BigInt::from(10i64).pow((b.scale - a.scale) as u32);
        (av * factor, bv, b.scale)
    }
}

impl BigDecimal {
    pub fn zero() -> Self {
        Self {
            value: BigInt::zero(),
            scale: 0,
        }
    }

    /// Construct from a BigInt mantissa and an explicit decimal scale.
    /// E.g. `from_bigint(BigInt::from(12345), 3)` represents `12.345`.
    pub fn from_bigint(value: BigInt, scale: i64) -> Self {
        Self { value, scale }
    }

    pub fn from_str(s: &str) -> Result<Self, BigDecimalError> {
        let (int_part, dec_part) = if let Some(pos) = s.find('.') {
            (&s[..pos], &s[pos + 1..])
        } else {
            (s, "")
        };

        let combined = alloc::format!("{}{}", int_part, dec_part);
        let value = combined
            .parse::<num_bigint::BigInt>()
            .map_err(|_| BigDecimalError::InvalidFormat)?;

        Ok(Self {
            value: BigInt(value),
            scale: dec_part.len() as i64,
        })
    }
}

impl fmt::Display for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.scale == 0 {
            return write!(f, "{}", self.value);
        }
        let s = self.value.to_string();
        let (sign, digits) = if s.starts_with('-') {
            ("-", &s[1..])
        } else {
            ("", s.as_str())
        };
        let scale = self.scale as usize;
        if scale >= digits.len() {
            // Need leading zeros after decimal point
            let zeros = scale - digits.len();
            write!(f, "{}0.{:0>width$}{}", sign, "", digits, width = zeros)
        } else {
            let (int_part, dec_part) = digits.split_at(digits.len() - scale);
            write!(f, "{}{}.{}", sign, int_part, dec_part)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BigDecimalError {
    #[error("invalid decimal format")]
    InvalidFormat,
}

impl Add for BigDecimal {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let (av, bv, scale) = align_bigdec(&self, &rhs);
        Self { value: BigInt(av + bv), scale }
    }
}

impl Sub for BigDecimal {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let (av, bv, scale) = align_bigdec(&self, &rhs);
        Self { value: BigInt(av - bv), scale }
    }
}

impl Mul for BigDecimal {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            value: BigInt(self.value.0 * rhs.value.0),
            scale: self.scale + rhs.scale,
        }
    }
}

impl Div for BigDecimal {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        // Multiply numerator by 10^PRECISION to preserve decimal places.
        let precision_factor =
            num_bigint::BigInt::from(10i64).pow(BIGDEC_DIV_PRECISION);
        let scaled = self.value.0 * precision_factor;
        Self {
            value: BigInt(scaled / rhs.value.0),
            scale: self.scale - rhs.scale + BIGDEC_DIV_PRECISION as i64,
        }
    }
}

impl Add for &BigDecimal {
    type Output = BigDecimal;
    fn add(self, rhs: Self) -> Self::Output {
        let (av, bv, scale) = align_bigdec(self, rhs);
        BigDecimal { value: BigInt(av + bv), scale }
    }
}

impl Sub for &BigDecimal {
    type Output = BigDecimal;
    fn sub(self, rhs: Self) -> Self::Output {
        let (av, bv, scale) = align_bigdec(self, rhs);
        BigDecimal { value: BigInt(av - bv), scale }
    }
}

impl Mul for &BigDecimal {
    type Output = BigDecimal;
    fn mul(self, rhs: Self) -> Self::Output {
        BigDecimal {
            value: BigInt(self.value.0.clone() * rhs.value.0.clone()),
            scale: self.scale + rhs.scale,
        }
    }
}

impl Div for &BigDecimal {
    type Output = BigDecimal;
    fn div(self, rhs: Self) -> Self::Output {
        let precision_factor =
            num_bigint::BigInt::from(10i64).pow(BIGDEC_DIV_PRECISION);
        let scaled = self.value.0.clone() * precision_factor;
        BigDecimal {
            value: BigInt(scaled / rhs.value.0.clone()),
            scale: self.scale - rhs.scale + BIGDEC_DIV_PRECISION as i64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bigint_arithmetic() {
        let a = BigInt::from(100);
        let b = BigInt::from(42);

        assert_eq!((a.clone() + b.clone()).0, num_bigint::BigInt::from(142));
        assert_eq!((a.clone() - b.clone()).0, num_bigint::BigInt::from(58));
        assert_eq!((a.clone() * b.clone()).0, num_bigint::BigInt::from(4200));
        assert_eq!((a.clone() / b.clone()).0, num_bigint::BigInt::from(2));
        assert_eq!((a % b).0, num_bigint::BigInt::from(16));
    }

    #[test]
    fn bigdecimal_add_same_scale() {
        let a = BigDecimal::from_str("1.5").unwrap();
        let b = BigDecimal::from_str("2.5").unwrap();
        assert_eq!((a + b).to_string(), "4.0");
    }

    #[test]
    fn bigdecimal_add_different_scale() {
        let a = BigDecimal::from_str("1.5").unwrap();   // scale=1
        let b = BigDecimal::from_str("0.25").unwrap();  // scale=2
        assert_eq!((a + b).to_string(), "1.75");
    }

    #[test]
    fn bigdecimal_sub() {
        let a = BigDecimal::from_str("10.0").unwrap();
        let b = BigDecimal::from_str("3.5").unwrap();
        assert_eq!((a - b).to_string(), "6.5");
    }

    #[test]
    fn bigdecimal_mul() {
        let a = BigDecimal::from_str("2.5").unwrap();  // 25 * 10^-1
        let b = BigDecimal::from_str("4.0").unwrap();  // 40 * 10^-1
        // 25*40 = 1000, scale = 2 → 10.00
        assert_eq!((a * b).to_string(), "10.00");
    }

    #[test]
    fn bigdecimal_div_precision() {
        let a = BigDecimal::from_str("1.0").unwrap();
        let b = BigDecimal::from_str("3.0").unwrap();
        let result = a / b;
        // Should have 18 decimal places of precision
        assert!(result.to_string().starts_with("0.333333333"));
    }

    #[test]
    fn bytes_hex_roundtrip() {
        let original = Bytes::from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        let hex = original.to_hex();
        assert_eq!(hex, "0xdeadbeef");

        let parsed = Bytes::from_hex(&hex).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn bytes_from_hex_no_prefix() {
        let bytes = Bytes::from_hex("deadbeef").unwrap();
        assert_eq!(bytes.as_slice(), &[0xde, 0xad, 0xbe, 0xef]);
    }
}
