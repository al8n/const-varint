#![no_main]

use ::chrono::{Datelike, Timelike};
use ::time::{Date, Duration, Month, PrimitiveDateTime, Time, UtcDateTime};
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

type ChronoUtc = ::chrono::DateTime<::chrono::Utc>;
type ChronoDuration = ::chrono::Duration;
type ChronoNaiveDateTime = ::chrono::NaiveDateTime;
type ChronoNaiveDate = ::chrono::NaiveDate;
type ChronoNaiveTime = ::chrono::NaiveTime;

type Utc = UtcDateTime;
type DateTime = PrimitiveDateTime;

fuzzy!(Duration, Date, DateTime, Time, Utc);

trait IntoTime {
  type Target;

  fn into_time(self) -> Self::Target;
}

impl IntoTime for ChronoDuration {
  type Target = Duration;

  fn into_time(self) -> Self::Target {
    Duration::new(self.num_seconds(), self.subsec_nanos())
  }
}

impl IntoTime for ChronoNaiveTime {
  type Target = Time;

  fn into_time(self) -> Self::Target {
    Time::from_hms(self.hour() as u8, self.minute() as u8, self.second() as u8).unwrap()
  }
}

impl IntoTime for ChronoNaiveDate {
  type Target = Date;

  fn into_time(self) -> Self::Target {
    Date::from_calendar_date(
      self.year(),
      Month::try_from(self.month() as u8).unwrap(),
      self.day() as u8,
    )
    .unwrap()
  }
}

impl IntoTime for ChronoNaiveDateTime {
  type Target = DateTime;

  fn into_time(self) -> Self::Target {
    DateTime::new(self.date().into_time(), self.time().into_time())
  }
}

impl IntoTime for ChronoUtc {
  type Target = Utc;

  fn into_time(self) -> Self::Target {
    let utc = self.naive_utc();
    Utc::new(utc.date().into_time(), utc.time().into_time())
  }
}

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum Types {
  Duration(ChronoDuration),
  NaiveDateTime(ChronoNaiveDateTime),
  NaiveDate(ChronoNaiveDate),
  NaiveTime(ChronoNaiveTime),
  Utc(ChronoUtc),
}

impl Types {
  fn check(self) {
    match self {
      Self::Duration(value) => check_duration(value.into_time()),
      Self::NaiveDateTime(value) => check_date_time(value.into_time()),
      Self::NaiveDate(value) => check_date(value.into_time()),
      Self::NaiveTime(value) => check_time(value.into_time()),
      Self::Utc(value) => check_utc(value.into_time()),
    }
  }
}

fuzz_target!(|data: Types| {
  data.check();
});
