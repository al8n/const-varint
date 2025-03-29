#![no_main]

use libfuzzer_sys::fuzz_target;

use ruint::aliases::{
  U0, U1, U1024, U128, U16, U2048, U256, U32, U320, U384, U4096, U448, U512, U64, U768,
};
use varing::{consume_varint, Varint};

macro_rules! fuzzy {
    ($($ty:ident), +$(,)?) => {
        $(
            paste::paste! {
                fn [<check_ $ty:snake>](value: $ty) {
                    {
                        {
                            let mut buf = [0; <$ty>::MAX_ENCODED_LEN];
                            let encoded_len = value.encode(&mut buf).unwrap();
                            assert!(!(encoded_len != value.encoded_len() || !(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN)));
                            let consumed = consume_varint(&buf).unwrap();
                            assert_eq!(consumed, encoded_len);

                            let (bytes_read, decoded) = <$ty>::decode(&buf).unwrap();
                            assert!(value == decoded && encoded_len == bytes_read);
                        }
                    }
                }
            }
        )*
    };
}

fuzzy!(U0, U1, U16, U32, U64, U128, U256, U320, U384, U448, U512, U768, U1024, U2048, U4096);

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
#[allow(clippy::large_enum_variant)]
enum Number {
  U0(U0),
  U1(U1),
  U16(U16),
  U32(U32),
  U64(U64),
  U128(U128),
  U256(U256),
  U320(U320),
  U384(U384),
  U448(U448),
  U512(U512),
  U768(U768),
  U1024(U1024),
  U2048(U2048),
  U4096(U4096),
}

impl Number {
  fn check(self) {
    match self {
      Self::U0(value) => check_u0(value),
      Self::U1(value) => check_u1(value),
      Self::U16(value) => check_u16(value),
      Self::U32(value) => check_u32(value),
      Self::U64(value) => check_u64(value),
      Self::U128(value) => check_u128(value),
      Self::U256(value) => check_u256(value),
      Self::U320(value) => check_u320(value),
      Self::U384(value) => check_u384(value),
      Self::U448(value) => check_u448(value),
      Self::U512(value) => check_u512(value),
      Self::U768(value) => check_u768(value),
      Self::U1024(value) => check_u1024(value),
      Self::U2048(value) => check_u2048(value),
      Self::U4096(value) => check_u4096(value),
    }
  }
}

fuzz_target!(|data: Number| {
  data.check();
});
