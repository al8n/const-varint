use crate::{ruint_impl::Packable, DecodeError, EncodeError, Varint};
use ::ruint_1::{aliases::*, Uint};
use num_rational_0_4::Ratio;

impl_varint_for_ratio!(128(U256));

macro_rules! impl_varint_for_ratio_ruint {
  ($($bits:literal),+$(,)?) => {
    paste::paste! {
      $(
        impl Varint for Ratio<Uint<$bits, { $bits / 64 } >> {
          const MIN_ENCODED_LEN: usize = Uint::<{$bits * 2}, {($bits * 2) / 64}>::MAX_ENCODED_LEN;

          const MAX_ENCODED_LEN: usize = Uint::<{$bits * 2}, {($bits * 2) / 64}>::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> usize {
            Packable::<Uint::<{$bits * 2}, {($bits * 2) / 64}>>::pack(*self.numer(), *self.denom()).encoded_len()
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            Packable::<Uint::<{$bits * 2}, {($bits * 2) / 64}>>::pack(*self.numer(), *self.denom()).encode(buf)
          }

          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
          where
            Self: Sized,
          {
            let (bytes_read, merged) = Uint::< { $bits * 2 }, {($bits * 2) / 64}>::decode(buf)?;
            let (numer, denom): (Uint<$bits, { $bits / 64 } >, Uint<$bits, { $bits / 64 } >) = Packable::<Uint::<{$bits * 2}, {($bits * 2) / 64}>>::unpack(merged);
            if denom.is_zero() {
              return Err(DecodeError::custom("denominator cannot be zero"));
            }
            Ok((bytes_read, Ratio::new_raw(numer, denom)))
          }
        }
      )*
    }
  };
}

// TODO: this can be implemented for all Uint<BITS, LIMBS> when feature(const_generics) is stable
impl_varint_for_ratio_ruint!(64, 128, 192, 256, 320, 384, 448, 512, 768, 1024, 2048, 4096,);
