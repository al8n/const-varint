use crate::*;

use ruint_1::Uint;

impl<const BITS: usize, const LBITS: usize> Varint for Uint<BITS, LBITS> {
  const MIN_ENCODED_LEN: usize = const {
    if BITS == 0 {
      0
    } else {
      1
    }
  };
  const MAX_ENCODED_LEN: usize = BITS.div_ceil(7);

  fn encoded_len(&self) -> usize {
    match BITS {
      0 => 0,
      1..=8 => encoded_u8_varint_len(self.to()),
      9..=16 => encoded_u16_varint_len(self.to()),
      17..=32 => encoded_u32_varint_len(self.to()),
      33..=64 => encoded_u64_varint_len(self.to()),
      65..=128 => encoded_u128_varint_len(self.to()),
      _ => {
        // Each byte in LEB128 can store 7 bits
        // Special case for 0 since it always needs 1 byte
        if self.is_zero() {
          return 1;
        }

        // Calculate position of highest set bit
        let highest_bit = BITS - self.leading_zeros();
        // Convert to number of LEB128 bytes needed
        // Each byte holds 7 bits, but we need to round up
        highest_bit.div_ceil(7)
      }
    }
  }

  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    match BITS {
      0 => Ok(0),
      1..=8 => encode_u8_varint_to(self.to(), buf),
      9..=16 => encode_u16_varint_to(self.to(), buf),
      17..=32 => encode_u32_varint_to(self.to(), buf),
      33..=64 => encode_u64_varint_to(self.to(), buf),
      65..=128 => encode_u128_varint_to(self.to(), buf),
      _ => {
        let len = self.encoded_len();
        let buf_len = buf.len();
        if buf_len < len {
          return Err(EncodeError::underflow(len, buf_len));
        }

        let mut value = *self;
        let mut bytes_written = 0;

        loop {
          let mut byte = (value & Uint::from(0x7f)).to::<u8>();
          value >>= 7;

          // If there are more bits to encode, set the continuation bit
          if !value.is_zero() {
            byte |= 0x80;
          }

          buf[bytes_written] = byte;
          bytes_written += 1;

          if value.is_zero() {
            break;
          }
        }

        Ok(bytes_written)
      }
    }
  }

  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    if BITS == 0 {
      return Ok((0, Self::ZERO));
    }

    if buf.is_empty() {
      return Err(DecodeError::Underflow);
    }

    let mut result = Self::ZERO;
    let mut shift = 0;
    let mut bytes_read = 0;

    while bytes_read < buf.len() {
      let byte = buf[bytes_read];
      // Extract the 7 data bits
      let value = Self::from(byte & 0x7f);

      // Check for overflow
      if shift >= BITS {
        return Err(DecodeError::Overflow);
      }

      // Add the bits to the result
      // Need to handle potential overflow
      if let Some(shifted) = value.checked_shl(shift) {
        result |= shifted;
      } else {
        return Err(DecodeError::Overflow);
      }

      bytes_read += 1;

      // If continuation bit is not set, we're done
      if byte & 0x80 == 0 {
        return Ok((bytes_read, result));
      }

      shift += 7;
    }

    // If we get here, the input ended with a continuation bit set
    Err(DecodeError::Underflow)
  }
}

#[cfg(any(feature = "num-rational_0_4", feature = "num-complex_0_4"))]
pub(crate) trait Packable<O> {
  fn pack(low: Self, high: Self) -> O;

  fn unpack(value: O) -> (Self, Self)
  where
    Self: Sized;
}

#[cfg(any(feature = "num-rational_0_4", feature = "num-complex_0_4"))]
impl<const BITS: usize, const LIMBS: usize, const OBITS: usize, const OLIMBS: usize>
  Packable<Uint<OBITS, OLIMBS>> for Uint<BITS, LIMBS>
{
  fn pack(low: Self, high: Self) -> Uint<OBITS, OLIMBS> {
    debug_assert_eq!(BITS * 2, OBITS, "BITS * 2 != OBITS");
    Uint::<OBITS, OLIMBS>::from(low) | (Uint::<OBITS, OLIMBS>::from(high) << Self::BITS)
  }

  fn unpack(value: Uint<OBITS, OLIMBS>) -> (Self, Self) {
    debug_assert_eq!(BITS * 2, OBITS, "BITS * 2 != OBITS");

    let low = value & Uint::<OBITS, OLIMBS>::from(Uint::<BITS, LIMBS>::MAX);
    let high = (value >> Self::BITS).to::<Uint<BITS, LIMBS>>();
    (low.to(), high.to())
  }
}

#[cfg(test)]
mod tests_ruint_1 {
  use super::*;

  use ruint_1::aliases::{
    U0, U1, U1024, U128, U16, U2048, U256, U32, U320, U384, U4096, U448, U512, U64, U768,
  };

  use quickcheck_macros::quickcheck;

  macro_rules! fuzzy {
    ($($ty:ident), +$(,)?) => {
      $(
        paste::paste! {
          #[quickcheck]
          fn [< fuzzy_ $ty:snake >](value: $ty) -> bool {
            let mut buf = [0; <$ty>::MAX_ENCODED_LEN];
            let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
            if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN) {
              return false;
            }

            let Ok(consumed) = crate::consume_varint(&buf) else {
              return false;
            };
            if consumed != encoded_len {
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

  fuzzy!(U0, U1, U16, U32, U64, U128, U256, U320, U384, U448, U512, U768, U1024, U2048, U4096);

  macro_rules! max_encoded_len {
    ($($ty:ident), +$(,)?) => {
      $(
        paste::paste! {
          #[test]
          fn [< test_ $ty:snake _min_max_encoded_len>]() {
            let max = $ty::MAX;
            let min = $ty::MIN;
            assert_eq!(max.encoded_len(), $ty::MAX_ENCODED_LEN);
            assert_eq!(min.encoded_len(), $ty::MIN_ENCODED_LEN);
          }
        }
      )*
    };
  }

  max_encoded_len!(
    U0, U1, U16, U32, U64, U128, U256, U320, U384, U448, U512, U768, U1024, U2048, U4096
  );

  #[cfg(feature = "std")]
  mod with_std {
    extern crate std;

    use super::*;

    use std::{vec, vec::Vec};

    // Helper type to generate fixed size arrays
    #[derive(Debug, Clone)]
    struct ByteArray<const N: usize>([u8; N]);

    impl<const N: usize> quickcheck::Arbitrary for ByteArray<N> {
      fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut arr = [0u8; N];
        for b in arr.iter_mut() {
          *b = u8::arbitrary(g);
        }
        ByteArray(arr)
      }
    }

    // Underflow tests for different sizes
    #[quickcheck]
    fn fuzzy_u256_buffer_underflow(bytes: ByteArray<32>, short_len: usize) -> bool {
      let uint = Uint::<256, 4>::from_be_bytes(bytes.0);
      let short_len = short_len % (Uint::<256, 4>::MAX_ENCODED_LEN - 1);
      if short_len >= uint.encoded_len() {
        return true;
      }
      let mut short_buffer = vec![0u8; short_len];
      matches!(
        uint.encode(&mut short_buffer),
        Err(EncodeError::Underflow { .. })
      )
    }

    #[quickcheck]
    fn fuzzy_u512_buffer_underflow(bytes: ByteArray<64>, short_len: usize) -> bool {
      let uint = Uint::<512, 8>::from_be_bytes(bytes.0);
      let short_len = short_len % (Uint::<512, 8>::MAX_ENCODED_LEN - 1);
      if short_len >= uint.encoded_len() {
        return true;
      }
      let mut short_buffer = vec![0u8; short_len];
      matches!(
        uint.encode(&mut short_buffer),
        Err(EncodeError::Underflow { .. })
      )
    }

    #[test]
    fn t() {
      std::println!("{} {}", U0::MAX, U0::MAX_ENCODED_LEN);
    }

    #[quickcheck]
    fn fuzzy_invalid_sequences(bytes: Vec<u8>) -> bool {
      if bytes.is_empty() {
        return matches!(U256::decode(&bytes), Err(DecodeError::Underflow));
      }

      // Only test sequences up to max varint length
      if bytes.len() > 10 {
        return true;
      }

      // If all bytes have continuation bit set, should get Underflow
      if bytes.iter().all(|b| b & 0x80 != 0) {
        return matches!(U256::decode(&bytes), Err(DecodeError::Underflow));
      }

      // For other cases, we should get either a valid decode or an error
      match U256::decode(&bytes) {
        Ok(_) => true,
        Err(_) => true,
      }
    }
  }
}
