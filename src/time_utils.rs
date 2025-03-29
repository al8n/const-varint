use crate::{
  decode_i128_varint, decode_i32_varint, decode_u128_varint, decode_u64_varint,
  encode_i128_varint_to, encode_i32_varint_to, encode_u128_varint, encode_u128_varint_to,
  encode_u64_varint_to, encoded_i128_varint_len, encoded_i32_varint_len, encoded_u128_varint_len,
  encoded_u64_varint_len, DecodeError, EncodeError, U128VarintBuffer,
};

#[inline]
pub(crate) const fn date_time_to_merged(
  year: i32,
  month: u8,
  day: u8,
  hour: u8,
  minute: u8,
  second: u8,
  nano: u32,
) -> i128 {
  // Date components
  let year = year as i128;
  let month = month as i128;
  let day = day as i128;
  // Time components
  let hour = hour as i128;
  let minute = minute as i128;
  let second = second as i128;
  let nano = nano as i128;

  // Place components with likely smaller values in the lower bits
  (nano & 0x7FFFFFFF) |              // Nanoseconds in lowest 31 bits
  ((second & 0x3F) << 31) |          // Seconds (6 bits) shifted by 31 bits
  ((minute & 0x3F) << 37) |          // Minutes (6 bits) shifted by 37 bits
  ((hour & 0x1F) << 43) |            // Hours (5 bits) shifted by 43 bits
  ((day & 0x1F) << 48) |             // Day (5 bits) shifted by 48 bits
  ((month & 0xF) << 53) |            // Month (4 bits) shifted by 53 bits
  (year << 57) // Year in highest bits (from bit 57)
}

#[inline]
pub(crate) const fn merged_to_date_time(
  merged: i128,
) -> (
  i32, // Year
  u8,  // Month
  u8,  // Day
  u8,  // Hour
  u8,  // Minute
  u8,  // Second
  u32, // Nanosecond
) {
  // Extract components
  let nano = (merged & 0x7FFFFFFF) as u32; // 31 bits for nanoseconds
  let second = ((merged >> 31) & 0x3F) as u8; // 6 bits for seconds
  let minute = ((merged >> 37) & 0x3F) as u8; // 6 bits for minutes
  let hour = ((merged >> 43) & 0x1F) as u8; // 5 bits for hours
  let day = ((merged >> 48) & 0x1F) as u8; // 5 bits for day
  let month = ((merged >> 53) & 0xF) as u8; // 4 bits for month
  let year = (merged >> 57) as i32; // Preserve sign bit for negative years

  (year, month, day, hour, minute, second, nano)
}

#[allow(unused)]
#[inline]
pub(crate) const fn encode_datetime(
  year: i32,
  month: u8,
  day: u8,
  hour: u8,
  minute: u8,
  second: u8,
  nano: u32,
) -> DateTimeBuffer {
  let merged = date_time_to_merged(year, month, day, hour, minute, second, nano);
  let mut buf = [0; DateTimeBuffer::CAPACITY + 1];
  let (data_buf, len_buf) = buf.split_at_mut(DateTimeBuffer::CAPACITY);
  let len = match encode_i128_varint_to(merged, data_buf) {
    Ok(len) => len,
    Err(_) => panic!("invalid datetime"),
  };
  len_buf[0] = len as u8;
  DateTimeBuffer::new(buf)
}

#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) const fn encode_datetime_to(
  year: i32,
  month: u8,
  day: u8,
  hour: u8,
  minute: u8,
  second: u8,
  nano: u32,
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  let merged = date_time_to_merged(year, month, day, hour, minute, second, nano);
  encode_i128_varint_to(merged, buf)
}

#[inline]
pub(crate) const fn encoded_datetime_len(
  year: i32,
  month: u8,
  day: u8,
  hour: u8,
  minute: u8,
  second: u8,
  nano: u32,
) -> usize {
  let merged = date_time_to_merged(year, month, day, hour, minute, second, nano);
  encoded_i128_varint_len(merged)
}

#[allow(clippy::type_complexity)]
#[inline]
pub(crate) const fn decode_datetime(
  buf: &[u8],
) -> Result<(usize, i32, u8, u8, u8, u8, u8, u32), DecodeError> {
  match decode_i128_varint(buf) {
    Ok((bytes_read, merged)) => {
      let (year, month, day, hour, minute, second, nano) = merged_to_date_time(merged);
      Ok((bytes_read, year, month, day, hour, minute, second, nano))
    }
    Err(e) => Err(e),
  }
}

#[inline]
pub(crate) const fn time_to_merged(nano: u32, second: u8, minute: u8, hour: u8) -> u64 {
  let nano = nano as u64 & 0x7FFF_FFFF; // 31 bits
  let second = (second as u64 & 0x3F) << 31; // 6 bits
  let minute = (minute as u64 & 0x3F) << 37; // 6 bits
  let hour = (hour as u64 & 0x1F) << 43; // 5 bits

  // Combine all components
  nano | second | minute | hour
}

#[inline]
pub(crate) const fn merged_to_time(merged: u64) -> (u32, u8, u8, u8) {
  let nano = (merged & 0x7FFF_FFFF) as u32; // 31 bits
  let second = ((merged >> 31) & 0x3F) as u8; // 6 bits
  let minute = ((merged >> 37) & 0x3F) as u8; // 6 bits
  let hour = ((merged >> 43) & 0x1F) as u8; // 5 bits

  (nano, second, minute, hour)
}

#[inline]
pub(crate) const fn decode_time(buf: &[u8]) -> Result<(usize, u32, u8, u8, u8), DecodeError> {
  match decode_u64_varint(buf) {
    Ok((bytes_read, merged)) => {
      let (nano, second, minute, hour) = merged_to_time(merged);
      Ok((bytes_read, nano, second, minute, hour))
    }
    Err(e) => Err(e),
  }
}

#[inline]
pub(crate) const fn encoded_time_len(nano: u32, second: u8, minute: u8, hour: u8) -> usize {
  let merged = time_to_merged(nano, second, minute, hour);
  encoded_u64_varint_len(merged)
}

#[inline]
pub(crate) const fn encode_time_to(
  nano: u32,
  second: u8,
  minute: u8,
  hour: u8,
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  let merged = time_to_merged(nano, second, minute, hour);
  encode_u64_varint_to(merged, buf)
}

#[allow(unused)]
#[inline]
pub(crate) const fn encode_time(nano: u32, second: u8, minute: u8, hour: u8) -> TimeBuffer {
  let merged = time_to_merged(nano, second, minute, hour);
  let mut buf = [0; TimeBuffer::CAPACITY + 1];
  let (data_buf, len_buf) = buf.split_at_mut(TimeBuffer::CAPACITY);
  let len = match encode_u64_varint_to(merged, data_buf) {
    Ok(len) => len,
    Err(_) => panic!("invalid time"),
  };
  len_buf[0] = len as u8;
  TimeBuffer::new(buf)
}

#[inline]
pub(crate) const fn date_to_merged(year: i32, month: u8, day: u8) -> i32 {
  let day = day as i32; // 1-31: 5 bits
  let month = month as i32; // 1-12: 4 bits

  // Place smallest values in lower bits for LEB128 efficiency
  day | (month << 5) | (year << 9)
}

#[inline]
pub(crate) const fn merged_to_date(merged: i32) -> (i32, u8, u8) {
  let day = (merged & 0x1F) as u8; // 5 bits
  let month = ((merged >> 5) & 0xF) as u8; // 4 bits
  let year = merged >> 9; // Remaining 16 bits

  (year, month, day)
}

#[inline]
pub(crate) const fn decode_date(buf: &[u8]) -> Result<(usize, i32, u8, u8), DecodeError> {
  match decode_i32_varint(buf) {
    Ok((bytes_read, merged)) => {
      let (year, month, day) = merged_to_date(merged);
      Ok((bytes_read, year, month, day))
    }
    Err(e) => Err(e),
  }
}

#[inline]
pub(crate) const fn encoded_date_len(year: i32, month: u8, day: u8) -> usize {
  let merged = date_to_merged(year, month, day);
  encoded_i32_varint_len(merged)
}

#[inline]
pub(crate) const fn encode_date_to(
  year: i32,
  month: u8,
  day: u8,
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  let merged = date_to_merged(year, month, day);
  encode_i32_varint_to(merged, buf)
}

#[allow(unused)]
#[inline]
pub(crate) const fn encode_date(year: i32, month: u8, day: u8) -> DateBuffer {
  let merged = date_to_merged(year, month, day);
  let mut buf = [0; DateBuffer::CAPACITY + 1];
  let (data_buf, len_buf) = buf.split_at_mut(DateBuffer::CAPACITY);
  let len = match encode_i32_varint_to(merged, data_buf) {
    Ok(len) => len,
    Err(_) => panic!("invalid date"),
  };
  len_buf[0] = len as u8;
  DateBuffer::new(buf)
}

#[inline]
pub(crate) const fn secs_and_subsec_nanos_to_merged(secs: i64, nanos: i32) -> u128 {
  // zigzag encode the values
  let secs = super::utils::zigzag_encode_i64(secs) as u128;
  let nanos = super::utils::zigzag_encode_i32(nanos) as u128;

  // Place smallest values in lower bits for LEB128 efficiency
  nanos | (secs << 32)
}

#[inline]
pub(crate) const fn merged_to_secs_and_subsec_nanos(merged: u128) -> (i64, i32) {
  // 1. Split out nanos (lower 32 bits) and secs (upper bits)
  let nanos_zz = (merged & 0xFFFF_FFFF) as u32;
  let secs_zz = (merged >> 32) as u64;

  // 2. ZigZag decode each component
  let nanos = super::utils::zigzag_decode_i32(nanos_zz);
  let secs = super::utils::zigzag_decode_i64(secs_zz);
  (secs, nanos)
}

#[inline]
pub(crate) const fn encode_secs_and_subsec_nanos(secs: i64, nanos: i32) -> U128VarintBuffer {
  encode_u128_varint(secs_and_subsec_nanos_to_merged(secs, nanos))
}

#[inline]
pub(crate) const fn encode_secs_and_subsec_nanos_to(
  secs: i64,
  nanos: i32,
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  let merged = secs_and_subsec_nanos_to_merged(secs, nanos);
  encode_u128_varint_to(merged, buf)
}

#[inline]
pub(crate) const fn encoded_secs_and_subsec_nanos_len(secs: i64, nanos: i32) -> usize {
  let merged = secs_and_subsec_nanos_to_merged(secs, nanos);
  encoded_u128_varint_len(merged)
}

#[inline]
pub(crate) const fn decode_secs_and_subsec_nanos(
  buf: &[u8],
) -> Result<(usize, i64, i32), DecodeError> {
  match decode_u128_varint(buf) {
    Ok((bytes_read, merged)) => {
      let (secs, nanos) = merged_to_secs_and_subsec_nanos(merged);
      Ok((bytes_read, secs, nanos))
    }
    Err(e) => Err(e),
  }
}

macro_rules! time_buffer {
  ($(
    $(#[$meta:meta])*
    $ty:ident($len: literal)
  ),+$(,)?) => {
    $(
      $(#[$meta])*
      pub struct $ty([u8; { $len + 1 }]);

      impl AsRef<[u8]> for $ty {
        #[inline]
        fn as_ref(&self) -> &[u8] {
          self
        }
      }

      impl core::ops::Deref for $ty {
        type Target = [u8];

        #[inline]
        fn deref(&self) -> &Self::Target {
          let len = self.0[$len] as usize;
          &self.0[..len]
        }
      }

      impl core::borrow::Borrow<[u8]> for $ty {
        #[inline]
        fn borrow(&self) -> &[u8] {
          self
        }
      }

      impl $ty {
        pub(crate) const CAPACITY: usize = $len;

        pub(crate) const fn new(buffer: [u8; { $len + 1 }]) -> Self {
          Self(buffer)
        }

        /// Returns the length of buffer.
        #[inline]
        pub const fn len(&self) -> usize {
          self.0[$len] as usize
        }

        /// Returns `true` if the buffer is empty.
        #[inline]
        pub const fn is_empty(&self) -> bool {
          self.0[$len] == 0
        }

        /// Returns the buffer as a slice.
        #[inline]
        pub const fn as_slice(&self) -> &[u8] {
          let (data, _) = self.0.split_at($len);
          data
        }
      }
    )*
  };
}

time_buffer!(
  /// A buffer for storing LEB128 encoded [`NaiveDate`] or [`Date`] value.
  ///
  /// [`NaiveDate`]: https://docs.rs/chrono/latest/chrono/struct.NaiveDate.html
  /// [`Date`]: https://docs.rs/time/latest/time/struct.Date.html
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  DateBuffer(5),
  /// A buffer for storing LEB128 encoded [`NaiveTime`] or [`Time`] value.
  ///
  /// [`NaiveTime`]: https://docs.rs/chrono/latest/chrono/struct.NaiveTime.html
  /// [`Time`]: https://docs.rs/time/latest/time/struct.Time.html
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  TimeBuffer(7),
  /// A buffer for storing LEB128 encoded [`NaiveDateTime`] or [`PrimitiveDateTime`] value.
  ///
  /// [`NaiveDateTime`]: https://docs.rs/chrono/latest/chrono/struct.NaiveDateTime.html
  /// [`PrimitiveDateTime`]: https://docs.rs/time/latest/time/struct.PrimitiveDateTime.html
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  DateTimeBuffer(12),
);
