use num_rational_0_4::Ratio;

use crate::{DecodeError, EncodeError, Varint};

macro_rules! impl_varint_for_ratio {
  (@inner $($sign:ident::$bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        impl Varint for Ratio<[< $sign $bits >]> {
          const MIN_ENCODED_LEN: usize = $merged_ty::MAX_ENCODED_LEN;

          const MAX_ENCODED_LEN: usize = $merged_ty::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> usize {
            $crate::utils::[< pack_ $sign $bits >](*self.numer(), *self.denom()).encoded_len()
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            $crate::utils::[< pack_ $sign $bits >](*self.numer(), *self.denom()).encode(buf)
          }

          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
          where
            Self: Sized,
          {
            let (bytes_read, merged) = $merged_ty::decode(buf)?;
            let (numer, denom) = $crate::utils::[< unpack_ $sign $bits >](merged);
            if denom == 0 {
              return Err(DecodeError::custom("denominator cannot be zero"));
            }
            Ok((bytes_read, Ratio::new_raw(numer, denom)))
          }
        }
      )*
    }
  };
  ($($bits:literal($merged_ty:ident)),+$(,)?) => {
    impl_varint_for_ratio!(@inner $(u::$bits($merged_ty)),+);
    impl_varint_for_ratio!(@inner $(i::$bits($merged_ty)),+);
  };
  (@const_inner $($sign:ident::$bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Returns the encoded length of the `Ratio<" $sign $bits ">` value."]
        #[inline]
        pub const fn [< encoded_ratio_ $sign $bits _len>](val: Ratio<[< $sign $bits >]>) -> usize {
          $crate::[< encoded_ $merged_ty _varint_len >] ($crate::utils::[< pack_ $sign $bits >](*val.numer(), *val.denom()))
        }

        #[doc = "Encodes the `Ratio<" $sign $bits ">` value."]
        #[inline]
        pub const fn [< encode_ratio_ $sign $bits >](val: Ratio<[< $sign $bits >]>) -> $crate::[< $merged_ty:camel VarintBuffer>] {
          $crate::[< encode_ $merged_ty _varint>] ($crate::utils::[< pack_ $sign $bits >](*val.numer(), *val.denom()))
        }

        #[doc = "Encodes the `Ratio<" $sign $bits ">` value into the provided buffer."]
        #[inline]
        pub const fn [< encode_ratio_ $sign $bits _to >](val: Ratio<[< $sign $bits >]>, buf: &mut [u8]) -> Result<usize, EncodeError> {
          $crate::[< encode_ $merged_ty _varint_to>] ($crate::utils::[< pack_ $sign $bits >](*val.numer(), *val.denom()), buf)
        }

        #[doc = "Decodes the `Ratio<" $sign $bits ">` value from the provided buffer."]
        #[inline]
        pub const fn [< decode_ratio_ $sign $bits >](buf: &[u8]) -> Result<(usize, Ratio<[< $sign $bits >]>), DecodeError> {
          match $crate::[< decode_ $merged_ty _varint >](buf) {
            Ok((bytes_read, merged)) => {
              let (numer, denom) = $crate::utils::[< unpack_ $sign $bits >](merged);
              if denom == 0 {
                Err(DecodeError::custom("denominator cannot be zero"))
              } else {
                Ok((bytes_read, Ratio::new_raw(numer, denom)))
              }
            }
            Err(e) => Err(e),
          }
        }
      )*
    }
  };
  (@const $($bits:literal($merged_ty:ident)),+$(,)?) => {
    impl_varint_for_ratio!(@const_inner $(u::$bits($merged_ty)),+);
    impl_varint_for_ratio!(@const_inner $(i::$bits($merged_ty)),+);
  }
}

impl_varint_for_ratio!(8(u16), 16(u32), 32(u64), 64(u128));
impl_varint_for_ratio!(@const 8(u16), 16(u32), 32(u64), 64(u128));

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
  struct ArbitraryRatio<T> {
    numer: T,
    denom: T,
  }

  macro_rules! impl_arbitrary_ratio {
    ($($ty:ty),+$(,)?) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryRatio<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let numer = <$ty>::arbitrary(g);
              let denom = loop {
                let denom = <$ty>::arbitrary(g);
                if denom != 0 {
                  break denom;
                }
              };
              Self { numer, denom }
            }
          }

          impl From<ArbitraryRatio<$ty>> for Ratio<$ty> {
            fn from(arbitrary: ArbitraryRatio<$ty>) -> Self {
              Ratio::new_raw(arbitrary.numer, arbitrary.denom)
            }
          }

          type [< Ratio $ty:camel >] = ArbitraryRatio<$ty>;
        )*
      }
    };
    (@ruint ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryRatio<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let numer = <$ty>::arbitrary(g);
              let denom = loop {
                let denom = <$ty>::arbitrary(g);
                if !denom.is_zero() {
                  break denom;
                }
              };
              Self { numer, denom }
            }
          }

          impl From<ArbitraryRatio<$ty>> for Ratio<$ty> {
            fn from(arbitrary: ArbitraryRatio<$ty>) -> Self {
              Ratio::new_raw(arbitrary.numer, arbitrary.denom)
            }
          }

          type [< RUintRatio $ty:camel >] = ArbitraryRatio<$ty>;
        )*
      }
    };
  }

  impl_arbitrary_ratio!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);

  fuzzy!(@varing_into (RatioU8(Ratio<u8>), RatioU16(Ratio<u16>), RatioU32(Ratio<u32>), RatioU64(Ratio<u64>), RatioI8(Ratio<i8>), RatioI16(Ratio<i16>), RatioI32(Ratio<i32>), RatioI64(Ratio<i64>),));
  fuzzy!(@varint_into (
    RatioU8(Ratio<u8>),
    RatioU16(Ratio<u16>),
    RatioU32(Ratio<u32>),
    RatioU64(Ratio<u64>),
    RatioI8(Ratio<i8>),
    RatioI16(Ratio<i16>),
    RatioI32(Ratio<i32>),
    RatioI64(Ratio<i64>),
  ));

  #[cfg(feature = "ruint_1")]
  impl_arbitrary_ratio!(@ruint (U64, U128, U192, U256, U384, U448, U512, U768, U1024, U2048, U4096,));

  #[cfg(feature = "ruint_1")]
  fuzzy!(@varint_into (
    RatioU128(Ratio<u128>),
    RatioI128(Ratio<i128>),
  ));

  #[cfg(feature = "ruint_1")]
  macro_rules! ratio_ruint_fuzzy {
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
              value.numer() == decoded.numer() && value.denom() == decoded.denom() && encoded_len == bytes_read
            } else {
              false
            }
          }
        }
      )*
    };
  }

  #[cfg(feature = "ruint_1")]
  ratio_ruint_fuzzy!(@varint_into (
    RUintRatioU64(Ratio<U64>),
    RUintRatioU128(Ratio<U128>),
    RUintRatioU192(Ratio<U192>),
    RUintRatioU256(Ratio<U256>),
    RUintRatioU384(Ratio<U384>),
    RUintRatioU448(Ratio<U448>),
    RUintRatioU512(Ratio<U512>),
    RUintRatioU768(Ratio<U768>),
    RUintRatioU1024(Ratio<U1024>),
    RUintRatioU2048(Ratio<U2048>),
    RUintRatioU4096(Ratio<U4096>),
  ));
}
