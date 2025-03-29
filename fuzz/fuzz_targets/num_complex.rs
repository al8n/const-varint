#![no_main]
#![allow(clippy::large_enum_variant)]

use libfuzzer_sys::fuzz_target;
use num_complex::Complex;
use ruint::aliases::*;
use varing::{consume_varint, num_complex::*, Varint};

fuzz_target!(|data: Sum| {
  data.check();
});

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
struct ArbitraryComplex<T> {
  re: T,
  im: T,
}

macro_rules! fuzzy {
    ($($ty:ty), +$(,)?) => {
        $(
            paste::paste! {
                type [<ArbitraryComplex $ty:camel>] = ArbitraryComplex<$ty>;

                fn [<check_complex_ $ty:snake>](value: [< ArbitraryComplex $ty:camel >]) {
                    let value: Complex<$ty> = value.into();
                    {
                        {
                            let encoded = [< encode_complex_ $ty:snake >](value);
                            assert!(!(encoded.len() != [< encoded_complex_ $ty:snake _len >] (value) || !(encoded.len() <= <Complex<$ty>>::MAX_ENCODED_LEN)));

                            let consumed = consume_varint(&encoded).unwrap();
                            assert_eq!(consumed, encoded.len());

                            let (bytes_read, decoded) = [< decode_complex_ $ty:snake >](&encoded).unwrap();
                            assert!(value == decoded && encoded.len() == bytes_read);
                        }

                        {
                            let mut buf = [0; <Complex<$ty>>::MAX_ENCODED_LEN];
                            let encoded_len = value.encode(&mut buf).unwrap();
                            assert!(!(encoded_len != value.encoded_len() || !(value.encoded_len() <= <Complex<$ty>>::MAX_ENCODED_LEN)));
                            let consumed = consume_varint(&buf).unwrap();
                            assert_eq!(consumed, encoded_len);

                            let (bytes_read, decoded) = <Complex<$ty>>::decode(&buf).unwrap();
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
                type [<ArbitraryComplex $ty:camel>] = ArbitraryComplex<$ty>;

                fn [<check_complex_ $ty:snake>](value: [< ArbitraryComplex $ty:camel >]) {
                    let value: Complex<$ty> = value.into();
                    {
                        {
                            let mut buf = [0; <Complex<$ty>>::MAX_ENCODED_LEN];
                            let encoded_len = value.encode(&mut buf).unwrap();
                            assert!(!(encoded_len != value.encoded_len() || !(value.encoded_len() <= <Complex<$ty>>::MAX_ENCODED_LEN)));
                            let consumed = consume_varint(&buf).unwrap();
                            assert_eq!(consumed, encoded_len);

                            let (bytes_read, decoded) = <Complex<$ty>>::decode(&buf).unwrap();
                            assert!(value == decoded && encoded_len == bytes_read);
                        }
                    }
                }
            }
        )*
    };
}

impl<T> From<ArbitraryComplex<T>> for Complex<T> {
  fn from(ac: ArbitraryComplex<T>) -> Self {
    Complex {
      re: ac.re,
      im: ac.im,
    }
  }
}

fuzzy!(u8, u16, u32, u64, i8, i16, i32, i64);
fuzzy!(@varint_only u128, i128);
fuzzy!(@varint_only U256, U320, U384, U448, U512, U768, U1024, U2048, U4096);

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum Types {
  U8(ArbitraryComplexU8),
  U16(ArbitraryComplexU16),
  U32(ArbitraryComplexU32),
  U64(ArbitraryComplexU64),
  I8(ArbitraryComplexI8),
  I16(ArbitraryComplexI16),
  I32(ArbitraryComplexI32),
  I64(ArbitraryComplexI64),
  U128(ArbitraryComplexU128),
  I128(ArbitraryComplexI128),
}

impl Types {
  fn check(self) {
    match self {
      Self::U8(value) => check_complex_u8(value),
      Self::U16(value) => check_complex_u16(value),
      Self::U32(value) => check_complex_u32(value),
      Self::U64(value) => check_complex_u64(value),
      Self::I8(value) => check_complex_i8(value),
      Self::I16(value) => check_complex_i16(value),
      Self::I32(value) => check_complex_i32(value),
      Self::I64(value) => check_complex_i64(value),
      Self::U128(value) => check_complex_u128(value),
      Self::I128(value) => check_complex_i128(value),
    }
  }
}

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum RuintTypes {
  U256(ArbitraryComplexU256),
  U320(ArbitraryComplexU320),
  U384(ArbitraryComplexU384),
  U448(ArbitraryComplexU448),
  U512(ArbitraryComplexU512),
  U768(ArbitraryComplexU768),
  U1024(ArbitraryComplexU1024),
  U2048(ArbitraryComplexU2048),
  U4096(ArbitraryComplexU4096),
}

impl RuintTypes {
  fn check(self) {
    match self {
      Self::U256(value) => check_complex_u256(value),
      Self::U320(value) => check_complex_u320(value),
      Self::U384(value) => check_complex_u384(value),
      Self::U448(value) => check_complex_u448(value),
      Self::U512(value) => check_complex_u512(value),
      Self::U768(value) => check_complex_u768(value),
      Self::U1024(value) => check_complex_u1024(value),
      Self::U2048(value) => check_complex_u2048(value),
      Self::U4096(value) => check_complex_u4096(value),
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
