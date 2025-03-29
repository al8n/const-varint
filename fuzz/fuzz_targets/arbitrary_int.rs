#![no_main]

use arbitrary_int::*;
use libfuzzer_sys::fuzz_target;
use varing::{arbitrary_int::*, consume_varint, Varint};

fuzz_target!(|data: Types| {
  data.check();
});

macro_rules! fuzzy {
    ($($ty:ty), +$(,)?) => {
        $(
            paste::paste! {
                fn [<check_ $ty>](value: $ty) {
                    {
                        {
                            let encoded = [< encode_ $ty _varint >](value);
                            assert!(!(encoded.len() != [< encoded_ $ty _varint_len >] (value) || !(encoded.len() <= <$ty>::MAX_ENCODED_LEN)));

                            let consumed = consume_varint(&encoded).unwrap();
                            assert_eq!(consumed, encoded.len());

                            let (bytes_read, decoded) = [< decode_ $ty _varint >](&encoded).unwrap();
                            assert!(value == decoded && encoded.len() == bytes_read);
                        }

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

macro_rules! gen_storage {
    ($($storage:ident($start:literal..=$end:literal)),+$(,)?) => {
        $(
            seq_macro::seq!(
                N in $start..=$end {
                    #[derive(Debug, Clone, Copy)]
                    enum $storage {
                        #(
                            U~N(u~N),
                        )*
                    }

                    fuzzy!(#(u~N,)*);

                    impl<'a> arbitrary::Arbitrary<'a> for $storage {
                        fn arbitrary(g: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                            Ok(match g.int_in_range($start..=$end)? {
                                #(
                                    N => Self::U~N(u~N::new(g.int_in_range(u~N::MIN.value()..=u~N::MAX.value())?)),
                                )*
                                _ => unreachable!(),
                            })
                        }
                    }

                    impl $storage {
                        fn check(self) {
                            match self {
                                #(
                                    Self::U~N(value) => check_u~N(value),
                                )*
                            }
                        }
                    }
                }
            );
        )*
    };
}

gen_storage!(
  U8(1..=7),
  U16(9..=15),
  U32(17..=31),
  U64(33..=63),
  U128(65..=127),
);

#[derive(Copy, Clone, Debug, arbitrary::Arbitrary)]
enum Types {
  U8(U8),
  U16(U16),
  U32(U32),
  U64(U64),
  U128(U128),
}

impl Types {
  fn check(self) {
    match self {
      Self::U128(value) => value.check(),
      Self::U32(value) => value.check(),
      Self::U16(value) => value.check(),
      Self::U8(value) => value.check(),
      Self::U64(value) => value.check(),
    }
  }
}
