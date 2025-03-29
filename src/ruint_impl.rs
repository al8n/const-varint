#[cfg(feature = "ruint_1")]
mod v1;
#[cfg(feature = "ruint_1")]
#[cfg(any(feature = "num-rational_0_4", feature = "num-complex_0_4"))]
pub(crate) use v1::*;
