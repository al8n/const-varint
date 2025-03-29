use time_0_3::{Date, Duration, Month, PrimitiveDateTime, Time, UtcDateTime};

use crate::{time_utils, DecodeError, EncodeError, U128VarintBuffer, Varint};

pub use time_utils::{DateBuffer, DateTimeBuffer, TimeBuffer};

macro_rules! impl_varint_for_time {
  ($($ty:ident($max:expr, $min:expr).$fn:ident), +$(,)?) => {
    paste::paste! {
      $(
        impl Varint for $ty {
          const MIN_ENCODED_LEN: usize = {
            assert!($min <= $max, concat!("`", stringify!($ty), "::MIN_ENCODED_LEN` must be less than or equal to `", stringify!($ty), "MAX_ENCODED_LEN`"));
            assert!($min > 0, concat!("`", stringify!($ty), "::MIN_ENCODED_LEN` must be greater than 0"));
            $min
          };

          const MAX_ENCODED_LEN: usize = {
            assert!($max >= $min, concat!("`", stringify!($ty), "::MAX_ENCODED_LEN` must be greater than or equal to `", stringify!($ty), "MIN_ENCODED_LEN`"));
            assert!($max > 0, concat!("`", stringify!($ty), "::MAX_ENCODED_LEN` must be greater than 0"));
            $max
          };

          fn encoded_len(&self) -> usize {
            [< encoded_ $fn _len >](self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            [< encode_ $fn _to >](self, buf)
          }

          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
          where
            Self: Sized,
          {
            [< decode_ $fn >](buf)
          }
        }
      )*
    }
  };
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Duration::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_duration_len(duration: &Duration) -> usize {
  time_utils::encoded_secs_and_subsec_nanos_len(
    duration.whole_seconds(),
    duration.subsec_nanoseconds(),
  )
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration(duration: &Duration) -> U128VarintBuffer {
  time_utils::encode_secs_and_subsec_nanos(duration.whole_seconds(), duration.subsec_nanoseconds())
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration_to(duration: &Duration, buf: &mut [u8]) -> Result<usize, EncodeError> {
  time_utils::encode_secs_and_subsec_nanos_to(
    duration.whole_seconds(),
    duration.subsec_nanoseconds(),
    buf,
  )
}

/// Decodes a `Duration` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_duration(buf: &[u8]) -> Result<(usize, Duration), DecodeError> {
  match time_utils::decode_secs_and_subsec_nanos(buf) {
    Ok((bytes_read, secs, nanos)) => Ok((bytes_read, Duration::new(secs, nanos))),
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Date::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_date_len(date: &Date) -> usize {
  time_utils::encoded_date_len(date.year(), date.month() as u8, date.day())
}

/// Encodes a `Date` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_date(date: &Date) -> DateBuffer {
  time_utils::encode_date(date.year(), date.month() as u8, date.day())
}

/// Encodes a `Date` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_date_to(date: &Date, buf: &mut [u8]) -> Result<usize, EncodeError> {
  time_utils::encode_date_to(date.year(), date.month() as u8, date.day(), buf)
}

/// Decodes a `Date` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_date(buf: &[u8]) -> Result<(usize, Date), DecodeError> {
  match time_utils::decode_date(buf) {
    Ok((bytes_read, year, month, day)) => {
      let month = match u8_to_month(month) {
        Ok(month) => month,
        Err(e) => return Err(e),
      };
      match Date::from_calendar_date(year, month, day) {
        Ok(date) => Ok((bytes_read, date)),
        Err(_) => Err(DecodeError::custom("invalid date value")),
      }
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`UtcDateTime::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_utc_len(dt: &UtcDateTime) -> usize {
  encoded_datetime_len(&PrimitiveDateTime::new(dt.date(), dt.time()))
}

/// Encodes a `UtcDateTime` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_utc(dt: &UtcDateTime) -> DateTimeBuffer {
  encode_datetime(&PrimitiveDateTime::new(dt.date(), dt.time()))
}

/// Encodes a `UtcDateTime` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_utc_to(dt: &UtcDateTime, buf: &mut [u8]) -> Result<usize, EncodeError> {
  encode_datetime_to(&PrimitiveDateTime::new(dt.date(), dt.time()), buf)
}

/// Decodes a `UtcDateTime` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_utc(buf: &[u8]) -> Result<(usize, UtcDateTime), DecodeError> {
  match decode_datetime(buf) {
    Ok((bytes_read, dt)) => Ok((bytes_read, UtcDateTime::new(dt.date(), dt.time()))),
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`PrimitiveDateTime::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_datetime_len(dt: &PrimitiveDateTime) -> usize {
  time_utils::encoded_datetime_len(
    dt.year(),
    dt.month() as u8,
    dt.day(),
    dt.hour(),
    dt.minute(),
    dt.second(),
    dt.nanosecond(),
  )
}

/// Encodes a `PrimitiveDateTime` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_datetime(dt: &PrimitiveDateTime) -> DateTimeBuffer {
  time_utils::encode_datetime(
    dt.year(),
    dt.month() as u8,
    dt.day(),
    dt.hour(),
    dt.minute(),
    dt.second(),
    dt.nanosecond(),
  )
}

/// Encodes a `PrimitiveDateTime` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_datetime_to(
  dt: &PrimitiveDateTime,
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  time_utils::encode_datetime_to(
    dt.year(),
    dt.month() as u8,
    dt.day(),
    dt.hour(),
    dt.minute(),
    dt.second(),
    dt.nanosecond(),
    buf,
  )
}

/// Decodes a `PrimitiveDateTime` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_datetime(buf: &[u8]) -> Result<(usize, PrimitiveDateTime), DecodeError> {
  match time_utils::decode_datetime(buf) {
    Ok((bytes_read, year, month, day, hour, minute, second, nano)) => {
      let month = match u8_to_month(month) {
        Ok(month) => month,
        Err(e) => return Err(e),
      };

      // Create date and time components
      let date = match Date::from_calendar_date(year, month, day) {
        Ok(date) => date,
        Err(_) => return Err(DecodeError::custom("invalid date value")),
      };
      let time = match Time::from_hms_nano(hour, minute, second, nano) {
        Ok(time) => time,
        Err(_) => return Err(DecodeError::custom("invalid time value")),
      };

      // Combine into PrimitiveDateTime
      Ok((bytes_read, PrimitiveDateTime::new(date, time)))
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Time::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_time_len(time: &Time) -> usize {
  time_utils::encoded_time_len(time.nanosecond(), time.second(), time.minute(), time.hour())
}

/// Encodes a `Time` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_time(time: &Time) -> TimeBuffer {
  time_utils::encode_time(time.nanosecond(), time.second(), time.minute(), time.hour())
}

/// Encodes a `Time` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_time_to(time: &Time, buf: &mut [u8]) -> Result<usize, EncodeError> {
  time_utils::encode_time_to(
    time.nanosecond(),
    time.second(),
    time.minute(),
    time.hour(),
    buf,
  )
}

/// Decodes a `Time` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_time(buf: &[u8]) -> Result<(usize, Time), DecodeError> {
  match time_utils::decode_time(buf) {
    Ok((bytes_read, nano, second, minute, hour)) => {
      // Create Time
      match Time::from_hms_nano(hour, minute, second, nano) {
        Ok(time) => Ok((bytes_read, time)),
        Err(_) => Err(DecodeError::custom("invalid time value")),
      }
    }
    Err(e) => Err(e),
  }
}

impl_varint_for_time!(
  Duration(i128::MAX_ENCODED_LEN, i128::MIN_ENCODED_LEN).duration,
  Time(TimeBuffer::CAPACITY, u64::MIN_ENCODED_LEN).time,
  PrimitiveDateTime(DateTimeBuffer::CAPACITY, 1).datetime,
  UtcDateTime(i128::MAX_ENCODED_LEN, 1).utc,
  Date(DateBuffer::CAPACITY, 1).date
);

const fn u8_to_month(val: u8) -> Result<Month, DecodeError> {
  Ok(match val {
    1 => Month::January,
    2 => Month::February,
    3 => Month::March,
    4 => Month::April,
    5 => Month::May,
    6 => Month::June,
    7 => Month::July,
    8 => Month::August,
    9 => Month::September,
    10 => Month::October,
    11 => Month::November,
    12 => Month::December,
    _ => return Err(DecodeError::custom("invalid month value")),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  extern crate std;

  type Utc = UtcDateTime;
  type Datetime = PrimitiveDateTime;

  #[derive(Debug, Clone, Copy)]
  struct DurationWrapper(Duration);

  impl quickcheck::Arbitrary for DurationWrapper {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
      let d: core::time::Duration = quickcheck::Arbitrary::arbitrary(g);

      Self(Duration::new(d.as_secs() as i64, d.subsec_nanos() as i32))
    }
  }

  #[quickcheck_macros::quickcheck]
  fn fuzzy_duration(value: DurationWrapper) -> bool {
    let value = value.0;
    let encoded = encode_duration(&value);
    if encoded.len() != encoded_duration_len(&value)
      || (encoded.len() > <Duration>::MAX_ENCODED_LEN)
    {
      return false;
    }

    let Ok(consumed) = crate::consume_varint(&encoded) else {
      return false;
    };
    if consumed != encoded.len() {
      return false;
    }

    if let Ok((bytes_read, decoded)) = decode_duration(&encoded) {
      value == decoded && encoded.len() == bytes_read
    } else {
      false
    }
  }

  #[quickcheck_macros::quickcheck]
  fn fuzzy_duration_varint(value: DurationWrapper) -> bool {
    let value = value.0;
    let mut buf = [0; <Duration>::MAX_ENCODED_LEN];
    let Ok(encoded_len) = value.encode(&mut buf) else {
      return false;
    };
    if encoded_len != value.encoded_len() || (value.encoded_len() > <Duration>::MAX_ENCODED_LEN) {
      return false;
    }

    let Ok(consumed) = crate::consume_varint(&buf) else {
      return false;
    };
    if consumed != encoded_len {
      return false;
    }

    if let Ok((bytes_read, decoded)) = <Duration>::decode(&buf) {
      value == decoded && encoded_len == bytes_read
    } else {
      false
    }
  }

  fuzzy!(@varing_ref(Date, Datetime, Time, Utc));
  fuzzy!(@varint(Date, PrimitiveDateTime, Time, UtcDateTime));
}
