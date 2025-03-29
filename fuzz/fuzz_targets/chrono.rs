#![no_main]

use ::chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use libfuzzer_sys::fuzz_target;
use varing::*;

macro_rules! fuzzy {
    ($($ty:ty), +$(,)?) => {
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

type ChronoUtc = DateTime<Utc>;

fuzzy!(Duration, NaiveDateTime, NaiveDate, NaiveTime, ChronoUtc);

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum Types {
  Duration(Duration),
  NaiveDateTime(NaiveDateTime),
  NaiveDate(NaiveDate),
  NaiveTime(NaiveTime),
  Utc(DateTime<Utc>),
}

impl Types {
  fn check(self) {
    match self {
      Self::Duration(value) => check_duration(value),
      Self::NaiveDateTime(value) => check_naive_date_time(value),
      Self::NaiveDate(value) => check_naive_date(value),
      Self::NaiveTime(value) => check_naive_time(value),
      Self::Utc(value) => check_chrono_utc(value),
    }
  }
}

fuzz_target!(|data: Types| {
  data.check();
});
