#![no_main]

use core::num::*;
use libfuzzer_sys::fuzz_target;
use varing::*;

macro_rules! fuzzy {
    ($($ty:ident), +$(,)?) => {
        $(
            paste::paste! {
                fn [<check_ $ty>](value: [< NonZero $ty:camel >]) {
                    {
                        {
                            let mut buf = [0; <[< NonZero $ty:camel >]>::MAX_ENCODED_LEN];
                            let encoded_len = value.encode(&mut buf).unwrap();
                            assert!(!(encoded_len != value.encoded_len() || !(value.encoded_len() <= <[< NonZero $ty:camel >]>::MAX_ENCODED_LEN)));
                            let consumed = consume_varint(&buf).unwrap();
                            assert_eq!(consumed, encoded_len);

                            let (bytes_read, decoded) = <[< NonZero $ty:camel >]>::decode(&buf).unwrap();
                            assert!(value == decoded && encoded_len == bytes_read);
                        }
                    }
                }
            }
        )*
    };
}

fuzzy!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum Number {
  U8(NonZeroU8),
  U16(NonZeroU16),
  U32(NonZeroU32),
  U64(NonZeroU64),
  U128(NonZeroU128),
  I8(NonZeroI8),
  I16(NonZeroI16),
  I32(NonZeroI32),
  I64(NonZeroI64),
  I128(NonZeroI128),
}

impl Number {
  fn check(self) {
    match self {
      Self::U8(value) => check_u8(value),
      Self::U16(value) => check_u16(value),
      Self::U32(value) => check_u32(value),
      Self::U64(value) => check_u64(value),
      Self::U128(value) => check_u128(value),
      Self::I8(value) => check_i8(value),
      Self::I16(value) => check_i16(value),
      Self::I32(value) => check_i32(value),
      Self::I64(value) => check_i64(value),
      Self::I128(value) => check_i128(value),
    }
  }
}

fuzz_target!(|data: Number| {
  data.check();
});
