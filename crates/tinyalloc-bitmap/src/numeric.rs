pub trait BitsRequire
where
  Self: Sized + Copy + PartialEq + Eq,
{
}

pub trait Bits
where
  Self: BitsRequire,
{
  const BITS: usize;

  fn zero() -> Self;
  fn max() -> Self;
}

macro_rules! impl_bits {
    ($($t:ty),*) => {
        $(
            impl BitsRequire for $t {}
            impl Bits for $t {
                const BITS: usize = <$t>::BITS as usize;

                fn zero() -> Self {
                    0
                }

                fn max() -> Self {
                    1
                }
            }
        )*
    };
}

impl_bits!(u8, u16, u32, u64, usize);
