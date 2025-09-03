use getset::Getters;

use crate::{
  error::BitmapError,
  numeric::{
    Bits,
    BitsRequire,
  },
};

#[derive(Debug, Getters)]
pub struct Bitmap<'slice, T>
where
  T: Bits + BitsRequire,
{
  #[getset(get = "pub")]
  store: &'slice mut [T],
  #[getset(get = "pub")]
  total_bits: usize,
}

impl<'slice, T> Bitmap<'slice, T>
where
  T: Bits + BitsRequire,
{
  pub const fn words(fields: usize) -> usize {
    (fields + T::BITS - 1) / T::BITS
  }

  pub const fn within(store: &'slice mut [T], fields: usize) -> Result<Self, BitmapError> {
    let total_bits = store.len() * T::BITS;
    if fields > total_bits {
      return Err(BitmapError::InsufficientSize {
        have: total_bits,
        need: fields,
      });
    }
    Ok(Self { store, total_bits })
  }

  pub fn clear_all(&mut self) {
    for word in self.store.iter_mut() {
      *word = T::zero();
    }
  }

  pub fn set_all(&mut self) {
    for word in self.store.iter_mut() {
      *word = T::max();
    }
  }

  fn position(&self, index: usize) -> Result<(usize, usize), BitmapError> {
    if index >= self.total_bits {
      return Err(BitmapError::OutOfBounds {
        index,
        size: self.total_bits,
      });
    }
    let word_index = index / T::BITS;
    let bit_index = index % T::BITS;
    Ok((word_index, bit_index))
  }
}
