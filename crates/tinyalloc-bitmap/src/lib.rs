use crate::numeric::{
  Bits,
  BitsRequire,
};

pub mod numeric;
#[cfg(test)]
pub mod tests;

#[derive(Debug)]
pub enum BitmapError {
  InsufficientSize { have: usize, need: usize },
  OutOfBounds { index: usize, size: usize },
}

#[derive(Debug)]
pub struct Bitmap<'slice, T>
where
  T: Bits + BitsRequire,
{
  store: &'slice mut [T],
  bits: usize,
}

impl<'slice, T> Bitmap<'slice, T>
where
  T: Bits + BitsRequire,
{
  pub const fn words(fields: usize) -> usize {
    (fields + T::BITS - 1) / T::BITS
  }

  pub fn available(&self) -> usize {
    self.store.len() * T::BITS
  }

  pub fn store(&self) -> &[T] {
    self.store
  }

  pub fn bits(&self) -> usize {
    self.bits
  }

  const fn position(
    &self,
    index: usize,
  ) -> Result<(usize, usize), BitmapError> {
    if index >= self.bits {
      return Err(BitmapError::OutOfBounds {
        index,
        size: self.bits,
      });
    }
    let word_index = index / T::BITS;
    let bit_index = index % T::BITS;
    Ok((word_index, bit_index))
  }

  pub fn zero(
    store: &'slice mut [T],
    bits: usize,
  ) -> Result<Self, BitmapError> {
    let available = store.len() * T::BITS;
    if bits > available {
      return Err(BitmapError::InsufficientSize {
        have: available,
        need: bits,
      });
    }

    let mut bitmap = Self { store, bits };
    bitmap.clear_all();
    Ok(bitmap)
  }

  pub fn one(store: &'slice mut [T], bits: usize) -> Result<Self, BitmapError> {
    let available = store.len() * T::BITS;
    if bits > available {
      return Err(BitmapError::InsufficientSize {
        have: available,
        need: bits,
      });
    }

    let mut bitmap = Self { store, bits };
    bitmap.set_all();
    Ok(bitmap)
  }

  pub fn check(&self, fields: usize) -> Result<(), BitmapError> {
    let total_bits = self.store.len() * T::BITS;
    if fields > total_bits {
      return Err(BitmapError::InsufficientSize {
        have: total_bits,
        need: fields,
      });
    }
    Ok(())
  }

  pub fn expect(&self, fields: usize) -> Result<(), BitmapError> {
    let total_bits = self.store.len() * T::BITS;
    if fields > total_bits {
      return Err(BitmapError::InsufficientSize {
        have: total_bits,
        need: fields,
      });
    }
    Ok(())
  }

  pub fn set(&mut self, index: usize) -> Result<(), BitmapError> {
    let (word_index, bit_index) = self.position(index)?;
    self.store[word_index] = self.store[word_index].set(bit_index);
    Ok(())
  }

  pub fn clear(&mut self, index: usize) -> Result<(), BitmapError> {
    let (word_index, bit_index) = self.position(index)?;
    self.store[word_index] = self.store[word_index].clear(bit_index);
    Ok(())
  }

  pub fn flip(&mut self, index: usize) -> Result<(), BitmapError> {
    let (word_index, bit_index) = self.position(index)?;
    self.store[word_index] = self.store[word_index].flip(bit_index);
    Ok(())
  }

  pub fn get(&self, index: usize) -> Result<bool, BitmapError> {
    let (word_index, bit_index) = self.position(index)?;
    Ok(self.store[word_index].get(bit_index))
  }

  pub fn clear_all(&mut self) {
    for word in self.store.iter_mut() {
      *word = T::zero();
    }
  }

  pub fn set_all(&mut self) {
    let full_words = self.bits / T::BITS;

    for word in self.store[..full_words].iter_mut() {
      *word = T::max();
    }

    let remaining_bits = self.bits % T::BITS;
    if remaining_bits > 0 && full_words < self.store.len() {
      let mask = T::max() >> (T::BITS - remaining_bits);
      self.store[full_words] = mask;
    }
  }

  pub fn find_first_set(&self) -> Option<usize> {
    for (word_index, &word) in self.store.iter().enumerate() {
      if word != T::zero() {
        let bit_offset = word.trailing_zeros() as usize;
        let global_index = word_index * T::BITS + bit_offset;
        if global_index < self.bits {
          return Some(global_index);
        }
      }
    }
    None
  }

  pub fn find_first_clear(&self) -> Option<usize> {
    for (word_index, &word) in self.store.iter().enumerate() {
      let inverted = word ^ T::max();
      if inverted != T::zero() {
        let bit_offset = inverted.trailing_zeros() as usize;
        let global_index = word_index * T::BITS + bit_offset;
        if global_index < self.bits {
          return Some(global_index);
        }
      }
    }
    None
  }

  pub fn is_clear(&self) -> bool {
    self.find_first_set().is_none()
  }
}
