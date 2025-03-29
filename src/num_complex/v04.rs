use num_complex_0_4::Complex;

use crate::{DecodeError, EncodeError, Varint};

macro_rules! impl_varint_for_complex {
  (@inner $($sign:ident::$bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        impl Varint for Complex<[< $sign $bits >]> {
          const MIN_ENCODED_LEN: usize = $merged_ty::MAX_ENCODED_LEN;

          const MAX_ENCODED_LEN: usize = $merged_ty::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> usize {
            $crate::utils::[< pack_ $sign $bits >](self.re, self.im).encoded_len()
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            $crate::utils::[< pack_ $sign $bits >](self.re, self.im).encode(buf)
          }

          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
          where
            Self: Sized,
          {
            let (bytes_read, merged) = $merged_ty::decode(buf)?;
            let (re, im) = $crate::utils::[< unpack_ $sign $bits >](merged);
            Ok((bytes_read, Complex { re, im }))
          }
        }
      )*
    }
  };
  ($($bits:literal($merged_ty:ident)),+$(,)?) => {
    impl_varint_for_complex!(@inner $(u::$bits($merged_ty)),+);
    impl_varint_for_complex!(@inner $(i::$bits($merged_ty)),+);
  };
  (@const_inner $($sign:ident::$bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Returns the encoded length of the `Complex<" $sign $bits ">` value."]
        #[inline]
        pub const fn [< encoded_complex_ $sign $bits _len>](val: Complex<[< $sign $bits >]>) -> usize {
          $crate::[< encoded_ $merged_ty _varint_len >] ($crate::utils::[< pack_ $sign $bits >](val.re, val.im))
        }

        #[doc = "Encodes the `Complex<" $sign $bits ">` value."]
        #[inline]
        pub const fn [< encode_complex_ $sign $bits >](val: Complex<[< $sign $bits >]>) -> $crate::[< $merged_ty:camel VarintBuffer>] {
          $crate::[< encode_ $merged_ty _varint>] ($crate::utils::[< pack_ $sign $bits >](val.re, val.im))
        }

        #[doc = "Encodes the `Complex<" $sign $bits ">` value into the provided buffer."]
        #[inline]
        pub const fn [< encode_complex_ $sign $bits _to >](val: Complex<[< $sign $bits >]>, buf: &mut [u8]) -> Result<usize, EncodeError> {
          $crate::[< encode_ $merged_ty _varint_to>] ($crate::utils::[< pack_ $sign $bits >](val.re, val.im), buf)
        }

        #[doc = "Decodes the `Complex<" $sign $bits ">` value from the provided buffer."]
        #[inline]
        pub const fn [< decode_complex_ $sign $bits >](buf: &[u8]) -> Result<(usize, Complex<[< $sign $bits >]>), DecodeError> {
          match $crate::[< decode_ $merged_ty _varint >](buf) {
            Ok((bytes_read, merged)) => {
              let (re, im) = $crate::utils::[< unpack_ $sign $bits >](merged);
              Ok((bytes_read, Complex { re, im }))
            }
            Err(e) => Err(e),
          }
        }
      )*
    }
  };
  (@const $($bits:literal($merged_ty:ident)),+$(,)?) => {
    impl_varint_for_complex!(@const_inner $(u::$bits($merged_ty)),+);
    impl_varint_for_complex!(@const_inner $(i::$bits($merged_ty)),+);
  }
}

impl_varint_for_complex!(8(u16), 16(u32), 32(u64), 64(u128));
impl_varint_for_complex!(@const 8(u16), 16(u32), 32(u64), 64(u128));

#[cfg(feature = "ruint_1")]
mod ruint_1;

#[cfg(test)]
mod tests {
  #![allow(warnings)]

  use super::*;

  use quickcheck::{Arbitrary, Gen};

  #[cfg(feature = "ruint_1")]
  use ::ruint_1::aliases::*;

  #[derive(Clone, Copy, Debug)]
  struct ArbitraryComplex<T> {
    re: T,
    im: T,
  }

  macro_rules! impl_arbitrary_complex {
    ($($ty:ty),+$(,)?) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryComplex<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let re = <$ty>::arbitrary(g);
              let im = <$ty>::arbitrary(g);
              Self { re, im }
            }
          }

          impl From<ArbitraryComplex<$ty>> for Complex<$ty> {
            fn from(arbitrary: ArbitraryComplex<$ty>) -> Self {
              Complex { re: arbitrary.re, im: arbitrary.im }
            }
          }

          type [< Complex $ty:camel >] = ArbitraryComplex<$ty>;
        )*
      }
    };
    (@ruint ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryComplex<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let re = <$ty>::arbitrary(g);
              let im = <$ty>::arbitrary(g);
              Self { re, im }
            }
          }

          impl From<ArbitraryComplex<$ty>> for Complex<$ty> {
            fn from(arbitrary: ArbitraryComplex<$ty>) -> Self {
              Complex { re: arbitrary.re, im: arbitrary.im }
            }
          }

          type [< RUintComplex $ty:camel >] = ArbitraryComplex<$ty>;
        )*
      }
    };
  }

  impl_arbitrary_complex!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);

  fuzzy!(@varing_into (ComplexU8(Complex<u8>), ComplexU16(Complex<u16>), ComplexU32(Complex<u32>), ComplexU64(Complex<u64>), ComplexI8(Complex<i8>), ComplexI16(Complex<i16>), ComplexI32(Complex<i32>), ComplexI64(Complex<i64>),));
  fuzzy!(@varint_into (
    ComplexU8(Complex<u8>),
    ComplexU16(Complex<u16>),
    ComplexU32(Complex<u32>),
    ComplexU64(Complex<u64>),
    ComplexI8(Complex<i8>),
    ComplexI16(Complex<i16>),
    ComplexI32(Complex<i32>),
    ComplexI64(Complex<i64>),
  ));

  #[cfg(feature = "ruint_1")]
  fuzzy!(@varint_into(
    ComplexI128(Complex<i128>),
    ComplexU128(Complex<u128>),
  ));

  #[cfg(feature = "ruint_1")]
  impl_arbitrary_complex!(@ruint (U64, U128, U192, U256, U384, U448, U512, U768, U1024, U2048, U4096,));

  #[cfg(feature = "ruint_1")]
  macro_rules! complex_ruint_fuzzy {
    (@varint_into ($($ty:ident($target:ty)), +$(,)?)) => {
      $(
        paste::paste! {
          #[quickcheck_macros::quickcheck]
          fn [< fuzzy_ $ty:snake _varint>](value: $ty) -> bool {
            let value: $target = ::core::convert::Into::into(value);
            let mut buf = [0; <$target>::MAX_ENCODED_LEN];
            let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
            if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$target>::MAX_ENCODED_LEN) {
              return false;
            }

            let Ok(consumed) = $crate::consume_varint(&buf) else {
              return false;
            };
            if consumed != encoded_len {
              return false;
            }

            if let Ok((bytes_read, decoded)) = <$target>::decode(&buf) {
              value.re == decoded.re && value.im == decoded.im && encoded_len == bytes_read
            } else {
              false
            }
          }
        }
      )*
    };
  }

  #[cfg(feature = "ruint_1")]
  complex_ruint_fuzzy!(@varint_into (
    RUintComplexU64(Complex<U64>),
    RUintComplexU128(Complex<U128>),
    RUintComplexU192(Complex<U192>),
    RUintComplexU256(Complex<U256>),
    RUintComplexU384(Complex<U384>),
    RUintComplexU448(Complex<U448>),
    RUintComplexU512(Complex<U512>),
    RUintComplexU768(Complex<U768>),
    RUintComplexU1024(Complex<U1024>),
    RUintComplexU2048(Complex<U2048>),
    RUintComplexU4096(Complex<U4096>),
  ));
}
