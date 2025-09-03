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

  fn leading_zeros(self) -> u32;
  fn trailing_zeros(self) -> u32;

  fn set(self, bit: usize) -> Self;
  fn clear(self, bit: usize) -> Self;
  fn flip(self, bit: usize) -> Self;
  fn get(self, bit: usize) -> bool;
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
                    <$t>::MAX
                }

                fn leading_zeros(self) -> u32 {
                    <$t>::leading_zeros(self)
                }

                fn trailing_zeros(self) -> u32 {
                    <$t>::trailing_zeros(self)
                }

                fn set(self, bit: usize) -> Self {
                    self | (1 << bit)
                }

                fn clear(self, bit: usize) -> Self {
                    self & !(1 << bit)
                }

                fn flip(self, bit: usize) -> Self {
                    self ^ (1 << bit)
                }

                fn get(self, bit: usize) -> bool {
                    (self & (1 << bit)) != 0
                }
            }
        )*
    };
}

impl_bits!(u8, u16, u32, u64, usize);
