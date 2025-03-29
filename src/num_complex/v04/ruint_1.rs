use crate::{ruint_impl::Packable, DecodeError, EncodeError, Varint};
use ::ruint_1::{aliases::*, Uint};
use num_complex_0_4::Complex;

impl_varint_for_complex!(128(U256));

macro_rules! impl_varint_for_complex_ruint {
  ($($bits:literal),+$(,)?) => {
    paste::paste! {
      $(
        impl Varint for Complex<Uint<$bits, { $bits / 64 } >> {
          const MIN_ENCODED_LEN: usize = Uint::<{$bits * 2}, {($bits * 2) / 64}>::MAX_ENCODED_LEN;
          const MAX_ENCODED_LEN: usize = Uint::<{$bits * 2}, {($bits * 2) / 64}>::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> usize {
            Packable::<Uint::<{$bits * 2}, {($bits * 2) / 64}>>::pack(self.re, self.im).encoded_len()
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            Packable::<Uint::<{$bits * 2}, {($bits * 2) / 64}>>::pack(self.re, self.im).encode(buf)
          }

          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
          where
            Self: Sized,
          {
            let (bytes_read, merged) = Uint::< { $bits * 2 }, {($bits * 2) / 64}>::decode(buf)?;
            let (re, im) = Packable::<Uint::<{$bits * 2}, {($bits * 2) / 64}>>::unpack(merged);
            Ok((bytes_read, Complex { re, im }))
          }
        }
      )*
    }
  };
}

// TODO: this can be implemented for all Uint<BITS, LIMBS> when feature(const_generics) is stable
impl_varint_for_complex_ruint!(64, 128, 192, 256, 320, 384, 448, 512, 768, 1024, 2048, 4096,);
