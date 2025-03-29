#![no_main]

use libfuzzer_sys::fuzz_target;

use ethereum_types::{U128, U256, U512, U64};
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

fuzzy!(U64, U128, U256, U512);

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum Number {
  U64(U64),
  U128(U128),
  U256(U256),
  U512(U512),
}

impl Number {
  fn check(self) {
    match self {
      Self::U64(value) => check_u64(value),
      Self::U128(value) => check_u128(value),
      Self::U256(value) => check_u256(value),
      Self::U512(value) => check_u512(value),
    }
  }
}

fuzz_target!(|data: Number| {
  data.check();
});
