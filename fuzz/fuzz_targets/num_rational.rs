#![no_main]
#![allow(clippy::large_enum_variant)]

use libfuzzer_sys::fuzz_target;
use num_rational::Ratio;
use num_traits::{One, Zero};
use ruint::aliases::*;
use varing::{consume_varint, num_rational::*, Varint};

fuzz_target!(|data: Sum| {
  data.check();
});

#[derive(Debug, Clone, Copy)]
struct ArbitraryRatio<T> {
  numer: T,
  denom: T,
}

impl<'a, T> arbitrary::Arbitrary<'a> for ArbitraryRatio<T>
where
  T: arbitrary::Arbitrary<'a> + Copy + Zero + One + Eq,
{
  fn arbitrary(g: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
    let numer = T::arbitrary(g)?;
    let denom = {
      let denom = T::arbitrary(g)?;
      if denom == T::zero() {
        T::one()
      } else {
        denom
      }
    };
    Ok(Self { numer, denom })
  }
}

macro_rules! fuzzy {
    ($($ty:ty), +$(,)?) => {
        $(
            paste::paste! {
                type [<ArbitraryRatio $ty:camel>] = ArbitraryRatio<$ty>;

                fn [<check_ratio_ $ty:snake>](value: [< ArbitraryRatio $ty:camel >]) {
                    let value: Ratio<$ty> = value.into();
                    {
                        {
                            let encoded = [< encode_ratio_ $ty:snake >](value);
                            assert!(!(encoded.len() != [< encoded_ratio_ $ty:snake _len >] (value) || !(encoded.len() <= <Ratio<$ty>>::MAX_ENCODED_LEN)));

                            let consumed = consume_varint(&encoded).unwrap();
                            assert_eq!(consumed, encoded.len());

                            let (bytes_read, decoded) = [< decode_ratio_ $ty:snake >](&encoded).unwrap();
                            assert!(value == decoded && encoded.len() == bytes_read);
                        }

                        {
                            let mut buf = [0; <Ratio<$ty>>::MAX_ENCODED_LEN];
                            let encoded_len = value.encode(&mut buf).unwrap();
                            assert!(!(encoded_len != value.encoded_len() || !(value.encoded_len() <= <Ratio<$ty>>::MAX_ENCODED_LEN)));
                            let consumed = consume_varint(&buf).unwrap();
                            assert_eq!(consumed, encoded_len);

                            let (bytes_read, decoded) = <Ratio<$ty>>::decode(&buf).unwrap();
                            assert!(value == decoded && encoded_len == bytes_read);
                        }
                    }
                }
            }
        )*
    };
    (@varint_only $($ty:ty), +$(,)?) => {
        $(
            paste::paste! {
                type [<ArbitraryRatio $ty:camel>] = ArbitraryRatio<$ty>;

                fn [<check_ratio_ $ty:snake>](value: [< ArbitraryRatio $ty:camel >]) {
                    let value: Ratio<$ty> = value.into();
                    {
                        {
                            let mut buf = [0; <Ratio<$ty>>::MAX_ENCODED_LEN];
                            let encoded_len = value.encode(&mut buf).unwrap();
                            assert!(!(encoded_len != value.encoded_len() || !(value.encoded_len() <= <Ratio<$ty>>::MAX_ENCODED_LEN)));
                            let consumed = consume_varint(&buf).unwrap();
                            assert_eq!(consumed, encoded_len);

                            let (bytes_read, decoded) = <Ratio<$ty>>::decode(&buf).unwrap();
                            assert!(value.numer().eq(decoded.numer()) && value.denom().eq(decoded.denom()) && encoded_len == bytes_read);
                        }
                    }
                }
            }
        )*
    };
}

impl<T> From<ArbitraryRatio<T>> for Ratio<T> {
  fn from(ac: ArbitraryRatio<T>) -> Self {
    Ratio::new_raw(ac.numer, ac.denom)
  }
}

fuzzy!(u8, u16, u32, u64, i8, i16, i32, i64);
fuzzy!(@varint_only u128, i128);
fuzzy!(@varint_only U256, U320, U384, U448, U512, U768, U1024, U2048, U4096);

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum Types {
  U8(ArbitraryRatioU8),
  U16(ArbitraryRatioU16),
  U32(ArbitraryRatioU32),
  U64(ArbitraryRatioU64),
  I8(ArbitraryRatioI8),
  I16(ArbitraryRatioI16),
  I32(ArbitraryRatioI32),
  I64(ArbitraryRatioI64),
  U128(ArbitraryRatioU128),
  I128(ArbitraryRatioI128),
}

impl Types {
  fn check(self) {
    match self {
      Self::U8(value) => check_ratio_u8(value),
      Self::U16(value) => check_ratio_u16(value),
      Self::U32(value) => check_ratio_u32(value),
      Self::U64(value) => check_ratio_u64(value),
      Self::I8(value) => check_ratio_i8(value),
      Self::I16(value) => check_ratio_i16(value),
      Self::I32(value) => check_ratio_i32(value),
      Self::I64(value) => check_ratio_i64(value),
      Self::U128(value) => check_ratio_u128(value),
      Self::I128(value) => check_ratio_i128(value),
    }
  }
}

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum RuintTypes {
  U256(ArbitraryRatioU256),
  U320(ArbitraryRatioU320),
  U384(ArbitraryRatioU384),
  U448(ArbitraryRatioU448),
  U512(ArbitraryRatioU512),
  U768(ArbitraryRatioU768),
  U1024(ArbitraryRatioU1024),
  U2048(ArbitraryRatioU2048),
  U4096(ArbitraryRatioU4096),
}

impl RuintTypes {
  fn check(self) {
    match self {
      Self::U256(value) => check_ratio_u256(value),
      Self::U320(value) => check_ratio_u320(value),
      Self::U384(value) => check_ratio_u384(value),
      Self::U448(value) => check_ratio_u448(value),
      Self::U512(value) => check_ratio_u512(value),
      Self::U768(value) => check_ratio_u768(value),
      Self::U1024(value) => check_ratio_u1024(value),
      Self::U2048(value) => check_ratio_u2048(value),
      Self::U4096(value) => check_ratio_u4096(value),
    }
  }
}

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum Sum {
  Types(Types),
  RuintTypes(RuintTypes),
}

impl Sum {
  fn check(self) {
    match self {
      Self::Types(value) => value.check(),
      Self::RuintTypes(value) => value.check(),
    }
  }
}
