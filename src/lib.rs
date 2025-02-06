#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

use core::{num::NonZeroU64, ops::RangeInclusive};

pub use duration::*;

mod duration;

macro_rules! impl_varint {
  ($($ty:literal), +$(,)?) => {
    $(
      paste::paste! {
        impl Varint for [< u $ty >] {
          const MIN_ENCODED_LEN: usize = [< encoded_ u $ty _varint_len >](0);
          const MAX_ENCODED_LEN: usize = [< encoded_ u $ty _varint_len >](<[< u $ty >]>::MAX);

          #[inline]
          fn encoded_len(&self) -> usize {
            [< encoded_ u $ty _varint_len >](*self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            [< encode_ u $ty _varint_to >](*self, buf)
          }

          #[inline]
          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError> {
            [< decode_ u $ty _varint >](buf)
          }
        }

        impl Varint for [< i $ty >] {
          const MIN_ENCODED_LEN: usize = [< encoded_ i $ty _varint_len >](0);
          const MAX_ENCODED_LEN: usize = [< encoded_ i $ty _varint_len >](<[< i $ty >]>::MAX);

          #[inline]
          fn encoded_len(&self) -> usize {
            [< encoded_ i $ty _varint_len >](*self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            [< encode_ i $ty _varint_to >](*self, buf)
          }

          #[inline]
          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError> {
            [< decode_ i $ty _varint >](buf)
          }
        }
      }
    )*
  };
}

macro_rules! decode_varint {
  (|$buf:ident| $ty:ident) => {{
    let mut result = 0;
    let mut shift = 0;
    let mut index = 0;

    loop {
      if index == $ty::MAX_ENCODED_LEN {
        return Err(DecodeError::Overflow);
      }

      if index >= $buf.len() {
        return Err(DecodeError::Underflow);
      }

      let next = $buf[index] as $ty;

      let v = $ty::BITS as usize / 7 * 7;
      let has_overflow = if shift < v {
        false
      } else if shift == v {
        next & ((u8::MAX << (::core::mem::size_of::<$ty>() % 7)) as $ty) != 0
      } else {
        true
      };

      if has_overflow {
        return Err(DecodeError::Overflow);
      }

      result += (next & 0x7F) << shift;
      if next & 0x80 == 0 {
        break;
      }
      shift += 7;
      index += 1;
    }
    Ok((index + 1, result))
  }};
}

macro_rules! encode_varint {
  ($buf:ident[$x:ident]) => {{
    let mut i = 0;

    while $x >= 0x80 {
      if i >= $buf.len() {
        panic!("insufficient buffer capacity");
      }

      $buf[i] = ($x as u8) | 0x80;
      $x >>= 7;
      i += 1;
    }

    // Check buffer capacity before writing final byte
    if i >= $buf.len() {
      panic!("insufficient buffer capacity");
    }

    $buf[i] = $x as u8;
    i + 1
  }};
  (@to_buf $ty:ident::$buf:ident[$x:ident]) => {{
    paste::paste! {
      let mut i = 0;
      let orig = $x;

      while $x >= 0x80 {
        if i >= $buf.len() {
          return Err(EncodeError::underflow([< encoded_ $ty _varint_len >](orig), $buf.len()));
        }

        $buf[i] = ($x as u8) | 0x80;
        $x >>= 7;
        i += 1;
      }

      // Check buffer capacity before writing final byte
      if i >= $buf.len() {
        return Err(EncodeError::underflow(i + 1, $buf.len()));
      }

      $buf[i] = $x as u8;
      Ok(i + 1)
    }
  }};
}

macro_rules! varint_len {
  ($($ty:ident),+$(,)?) => {
    $(
      paste::paste! {
        /// Returns the encoded length of the value in LEB128 variable length format.
        #[doc = "The returned value will be in range of [`" $ty "::ENCODED_LEN_RANGE`]."]
        #[inline]
        pub const fn [< encoded_ $ty _varint_len >](value: $ty) -> usize {
          encoded_u64_varint_len(value as u64)
        }
      }
    )*
  };
  (@zigzag $($ty:ident),+$(,)?) => {
    $(
      paste::paste! {
        /// Returns the encoded length of the value in LEB128 variable length format.
        #[doc = "The returned value will be in range of [`" $ty "::ENCODED_LEN_RANGE`]."]
        #[inline]
        pub const fn [< encoded_ $ty _varint_len >](value: $ty) -> usize {
          encoded_i64_varint_len(value as i64)
        }
      }
    )*
  };
}

macro_rules! buffer {
  ($($ty:ident), +$(,)?) => {
    $(
      paste::paste! {
        #[doc = "A buffer for storing LEB128 encoded " $ty " values."]
        #[derive(Copy, Clone, PartialEq, Eq)]
        pub struct [< $ty:camel VarintBuffer >]([u8; $ty::MAX_ENCODED_LEN + 1]);

        impl core::fmt::Debug for [< $ty:camel VarintBuffer >] {
          fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.0[..self.len()].fmt(f)
          }
        }

        impl [< $ty:camel VarintBuffer >] {
          const LAST_INDEX: usize = $ty::MAX_ENCODED_LEN;

          #[allow(dead_code)]
          #[inline]
          const fn new(mut val: $ty) -> Self {
            let mut buf = [0; $ty::MAX_ENCODED_LEN + 1];
            let mut_buf = &mut buf;
            let len = encode_varint!(mut_buf[val]);
            buf[Self::LAST_INDEX] = len as u8;
            Self(buf)
          }

          /// Returns the number of bytes in the buffer.
          #[inline]
          #[allow(clippy::len_without_is_empty)]
          pub const fn len(&self) -> usize {
            self.0[Self::LAST_INDEX] as usize
          }

          /// Extracts a slice from the buffer.
          #[inline]
          pub const fn as_bytes(&self) -> &[u8] {
            self.0.split_at(self.len()).0
          }
        }

        impl core::ops::Deref for [< $ty:camel VarintBuffer >] {
          type Target = [u8];

          fn deref(&self) -> &Self::Target {
            &self.0[..self.len()]
          }
        }

        impl core::borrow::Borrow<[u8]> for [< $ty:camel VarintBuffer >] {
          fn borrow(&self) -> &[u8] {
            self
          }
        }

        impl AsRef<[u8]> for [< $ty:camel VarintBuffer >] {
          fn as_ref(&self) -> &[u8] {
            self
          }
        }
      }
    )*
  };
}

macro_rules! encode {
  ($($ty:literal), +$(,)?) => {
    $(
      paste::paste! {
        #[doc = "Encodes an `u" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ u $ty _varint >](x: [< u $ty >]) -> [< U $ty:camel VarintBuffer >] {
          [< U $ty:camel VarintBuffer >]::new(x)
        }

        #[doc = "Encodes an `i" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ i $ty _varint >](x: [< i $ty >]) -> [< I $ty:camel VarintBuffer >] {
          let x = (x << 1) ^ (x >> ($ty - 1)); // Zig-zag encoding;
          [< I $ty:camel VarintBuffer >]([< U $ty:camel VarintBuffer >]::new(x as [< u $ty >]).0)
        }

        #[doc = "Encodes an `u" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ u $ty _varint_to >](mut x: [< u $ty >], buf: &mut [u8]) -> Result<usize, EncodeError> {
          encode_varint!(@to_buf [< u $ty >]::buf[x])
        }

        #[doc = "Encodes an `i" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ i $ty _varint_to >](x: [< i $ty >], buf: &mut [u8]) -> Result<usize, EncodeError> {
          let mut x = {
            // Zig-zag encoding
            ((x << 1) ^ (x >> ($ty - 1))) as [< u $ty >]
          };
          encode_varint!(@to_buf [<u $ty>]::buf[x])
        }
      }
    )*
  };
}

macro_rules! decode {
  ($($ty:literal), + $(,)?) => {
    $(
      paste::paste! {
        #[doc = "Decodes a `i" $ty "` in LEB128 encoded format from the buffer."]
        ///
        /// Returns the bytes readed and the decoded value if successful.
        pub const fn [< decode_ u $ty _varint >](buf: &[u8]) -> Result<(usize, [< u $ty >]), DecodeError> {
          decode_varint!(|buf| [< u $ty >])
        }

        #[doc = "Decodes a `u" $ty "` in LEB128 encoded format from the buffer."]
        ///
        /// Returns the bytes readed and the decoded value if successful.
        pub const fn [< decode_ i $ty _varint >](buf: &[u8]) -> Result<(usize, [< i $ty >]), DecodeError> {
          match [< decode_ u $ty _varint >](buf) {
            Ok((bytes_read, value)) => {
              let value = ((value >> 1) as [< i $ty >]) ^ { -((value & 1) as [< i $ty >]) }; // Zig-zag decoding
              Ok((bytes_read, value))
            },
            Err(e) => Err(e),
          }
        }
      }
    )*
  };
}

impl_varint!(16, 32, 64, 128,);
varint_len!(u16, u32,);
varint_len!(@zigzag i16, i32,);
buffer!(u16, u32, u64, u128, i16, i32, i64, i128);
encode!(128, 64, 32, 16,);
decode!(128, 64, 32, 16,);

/// A trait for types that can be encoded as variable-length integers (varints).
///
/// Varints are a method of serializing integers using one or more bytes that allows small
/// numbers to be stored in fewer bytes. The encoding scheme is compatible with Protocol Buffers'
/// base-128 varint format.
pub trait Varint {
  /// The minimum number of bytes needed to encode any value of this type.
  ///
  /// - For `u16` and `i16`, this is `1`.
  /// - For `u32` and `i32`, this is `1`.
  /// - For `u64` and `i64`, this is `1`.
  /// - For `u128` and `i128`, this is `1`.
  const MIN_ENCODED_LEN: usize;

  /// The maximum number of bytes that might be needed to encode any value of this type.
  ///
  /// - For `u16` and `i16`, this is `3`.
  /// - For `u32` and `i32`, this is `5`.
  /// - For `u64` and `i64`, this is `10`.
  /// - For `u128` and `i128`, this is `19`.
  const MAX_ENCODED_LEN: usize;

  /// The range of possible encoded lengths for this type, from `MIN_ENCODED_LEN` to `MAX_ENCODED_LEN` inclusive.
  ///
  /// This range can be used to pre-allocate buffers or validate encoded data lengths.
  ///
  /// - For `u16` and `i16`, this range is `1..=3`, representing possible encoded lengths of 1, 2, or 3 bytes.
  /// - For `u32` and `i32`, this range is `1..=5`, representing possible encoded lengths of 1, 2, 3, 4, or 5 bytes.
  /// - For `u64` and `u64`, this range is `1..=10`, representing possible encoded lengths of 1 to 10 bytes.
  /// - For `u128` and `i128`, this range is `1..=19`, representing possible encoded lengths of 1 to 19 bytes.
  const ENCODED_LEN_RANGE: RangeInclusive<usize> = Self::MIN_ENCODED_LEN..=Self::MAX_ENCODED_LEN;

  /// Returns the encoded length of the value in LEB128 variable length format.
  /// The returned value will be in range [`Self::ENCODED_LEN_RANGE`](Varint::ENCODED_LEN_RANGE).
  fn encoded_len(&self) -> usize;

  /// Encodes the value as a varint and writes it to the buffer.
  ///
  /// Returns the number of bytes written to the buffer.
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError>;

  /// Decodes the value from the buffer.
  ///
  /// Returns the number of bytes read from the buffer and the decoded value if successful.
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized;
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`u128::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_u128_varint_len(value: u128) -> usize {
  // Each byte in LEB128 encoding can hold 7 bits of data
  // We want to find how many groups of 7 bits are needed
  // Special case for 0 and small numbers
  if value < 128 {
    return 1;
  }

  // Calculate position of highest set bit
  let highest_bit = 128 - value.leading_zeros();
  // Convert to number of LEB128 bytes needed
  // Each byte holds 7 bits, but we need to round up
  ((highest_bit + 6) / 7) as usize
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`i128::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_i128_varint_len(x: i128) -> usize {
  let x = (x << 1) ^ (x >> 127); // Zig-zag encoding; // Zig-zag decoding;
  encoded_u128_varint_len(x as u128)
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`i64::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_i64_varint_len(x: i64) -> usize {
  let x = (x << 1) ^ (x >> 63); // Zig-zag encoding
  encoded_u64_varint_len(x as u64)
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`u64::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_u64_varint_len(value: u64) -> usize {
  // Based on [VarintSize64][1].
  // [1]: https://github.com/protocolbuffers/protobuf/blob/v28.3/src/google/protobuf/io/coded_stream.h#L1744-L1756
  // Safety: (value | 1) is never zero
  let log2value = unsafe { NonZeroU64::new_unchecked(value | 1) }.ilog2();
  ((log2value * 9 + (64 + 9)) / 64) as usize
}

/// Encode varint error
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, thiserror::Error)]
pub enum EncodeError {
  /// The buffer does not have enough capacity to encode the value.
  #[error("buffer does not have enough capacity to encode the value")]
  Underflow {
    /// The number of bytes needed to encode the value.
    required: usize,
    /// The number of bytes remaining in the buffer.
    remaining: usize,
  },
}

impl EncodeError {
  /// Creates a new `EncodeError::Underflow` with the required and remaining bytes.
  #[inline]
  pub const fn underflow(required: usize, remaining: usize) -> Self {
    Self::Underflow {
      required,
      remaining,
    }
  }
}

/// Decoding varint error.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, thiserror::Error)]
pub enum DecodeError {
  /// The buffer does not contain a valid LEB128 encoding.
  #[error("value would overflow the target type")]
  Overflow,
  /// The buffer does not contain enough data to decode.
  #[error("buffer does not contain enough data to decode a value")]
  Underflow,
}

#[cfg(feature = "ruint_1")]
mod ruint_impl;

#[cfg(test)]
mod tests {
  extern crate std;

  use super::*;

  fn check(value: u64, encoded: &[u8]) {
    let a = encode_u64_varint(value);
    assert_eq!(a.as_ref(), encoded);
    assert_eq!(a.len(), encoded.len());
    assert_eq!(a.len(), encoded_u64_varint_len(value));

    let (read, decoded) = decode_u64_varint(&a).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(read, encoded.len());
    assert_eq!(a.len(), encoded_u64_varint_len(value));
  }

  #[test]
  fn roundtrip_u64() {
    check(2u64.pow(0) - 1, &[0x00]);
    check(2u64.pow(0), &[0x01]);

    check(2u64.pow(7) - 1, &[0x7F]);
    check(2u64.pow(7), &[0x80, 0x01]);
    check(300u64, &[0xAC, 0x02]);

    check(2u64.pow(14) - 1, &[0xFF, 0x7F]);
    check(2u64.pow(14), &[0x80, 0x80, 0x01]);

    check(2u64.pow(21) - 1, &[0xFF, 0xFF, 0x7F]);
    check(2u64.pow(21), &[0x80, 0x80, 0x80, 0x01]);

    check(2u64.pow(28) - 1, &[0xFF, 0xFF, 0xFF, 0x7F]);
    check(2u64.pow(28), &[0x80, 0x80, 0x80, 0x80, 0x01]);

    check(2u64.pow(35) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
    check(2u64.pow(35), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

    check(2u64.pow(42) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
    check(2u64.pow(42), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

    check(
      2u64.pow(49) - 1,
      &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
    );
    check(
      2u64.pow(49),
      &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
    );

    check(
      2u64.pow(56) - 1,
      &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
    );
    check(
      2u64.pow(56),
      &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
    );

    check(
      2u64.pow(63) - 1,
      &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
    );
    check(
      2u64.pow(63),
      &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
    );

    check(
      u64::MAX,
      &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
    );
  }

  #[test]
  fn test_large_number_encode_decode() {
    let original = 30000u64;
    let encoded = encode_u64_varint(original);
    let (bytes_read, decoded) = decode_u64_varint(&encoded).unwrap();
    assert_eq!(original, decoded);
    assert_eq!(bytes_read, encoded.len());
  }

  #[test]
  fn test_decode_overflow_error() {
    let buffer = [0x80u8; 11]; // More than 10 bytes
    match decode_u64_varint(&buffer) {
      Err(DecodeError::Overflow) => (),
      _ => panic!("Expected Overflow error"),
    }

    let buffer = [0x80u8; 6]; // More than 5 bytes
    match decode_u32_varint(&buffer) {
      Err(DecodeError::Overflow) => (),
      _ => panic!("Expected Overflow error"),
    }

    let buffer = [0x80u8; 4]; // More than 3 bytes
    match decode_u16_varint(&buffer) {
      Err(DecodeError::Overflow) => (),
      _ => panic!("Expected Overflow error"),
    }
  }

  // Helper function for zig-zag encoding and decoding
  fn test_zigzag_encode_decode<T>(value: T)
  where
    T: Copy
      + PartialEq
      + core::fmt::Debug
      + core::ops::Shl<Output = T>
      + core::ops::Shr<Output = T>
      + Into<i64>
      + core::convert::TryInto<usize>
      + core::convert::TryFrom<usize>,
  {
    let encoded = encode_i64_varint(value.into());
    let bytes_written = encoded.len();

    // Decode
    let decode_result = decode_i64_varint(&encoded);
    assert!(decode_result.is_ok(), "Decoding failed");
    let (decoded_bytes, decoded_value) = decode_result.unwrap();

    assert_eq!(
      decoded_bytes, bytes_written,
      "Incorrect number of bytes decoded"
    );
    assert_eq!(
      decoded_value,
      value.into(),
      "Decoded value does not match original"
    );
  }

  #[test]
  fn test_zigzag_encode_decode_i16() {
    let values = [-1, 0, 1, -100, 100, i16::MIN, i16::MAX];
    for &value in &values {
      test_zigzag_encode_decode(value);
    }
  }

  #[test]
  fn test_zigzag_encode_decode_i32() {
    let values = [-1, 0, 1, -10000, 10000, i32::MIN, i32::MAX];
    for &value in &values {
      test_zigzag_encode_decode(value);
    }
  }

  #[test]
  fn test_zigzag_encode_decode_i64() {
    let values = [-1, 0, 1, -1000000000, 1000000000, i64::MIN, i64::MAX];
    for &value in &values {
      test_zigzag_encode_decode(value);
    }
  }
}

#[cfg(test)]
mod fuzzy {
  use super::*;

  use quickcheck_macros::quickcheck;

  macro_rules! fuzzy {
    ($($ty:ty), +$(,)?) => {
      $(
        paste::paste! {
          #[quickcheck]
          fn [< fuzzy_ $ty >](value: $ty) -> bool {
            let encoded = [< encode_ $ty _varint >](value);
            if encoded.len() != [< encoded_ $ty _varint_len >] (value) || !(encoded.len() <= <$ty>::MAX_ENCODED_LEN) {
              return false;
            }

            if let Ok((bytes_read, decoded)) = [< decode_ $ty _varint >](&encoded) {
              value == decoded && encoded.len() == bytes_read
            } else {
              false
            }
          }

          #[quickcheck]
          fn [< fuzzy_ $ty _varint>](value: $ty) -> bool {
            let mut buf = [0; <$ty>::MAX_ENCODED_LEN];
            let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
            if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN) {
              return false;
            }

            if let Ok((bytes_read, decoded)) = <$ty>::decode(&buf) {
              value == decoded && encoded_len == bytes_read
            } else {
              false
            }
          }
        }
      )*
    };
  }

  fuzzy!(u16, u32, u64, u128, i16, i32, i64, i128);

  #[cfg(feature = "std")]
  mod with_std {
    use super::*;

    extern crate std;

    use std::{vec, vec::Vec};

    #[quickcheck]
    fn fuzzy_buffer_underflow(value: u64, short_len: usize) -> bool {
      let short_len = short_len % 9; // Keep length under max varint size
      if short_len >= value.encoded_len() {
        return true; // Skip test if buffer is actually large enough
      }
      let mut short_buffer = vec![0u8; short_len];
      matches!(
        value.encode(&mut short_buffer),
        Err(EncodeError::Underflow { .. })
      )
    }

    #[quickcheck]
    fn fuzzy_invalid_sequences(bytes: Vec<u8>) -> bool {
      if bytes.is_empty() {
        return matches!(decode_u64_varint(&bytes), Err(DecodeError::Underflow));
      }

      // Only test sequences up to max varint length
      if bytes.len() > 10 {
        return true;
      }

      // If all bytes have continuation bit set, should get Underflow
      if bytes.iter().all(|b| b & 0x80 != 0) {
        return matches!(decode_u64_varint(&bytes), Err(DecodeError::Underflow));
      }

      // For other cases, we should get either a valid decode or an error
      match decode_u64_varint(&bytes) {
        Ok(_) => true,
        Err(_) => true,
      }
    }
  }
}
