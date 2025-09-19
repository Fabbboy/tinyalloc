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
  used: usize,
}

impl<'slice, T> Bitmap<'slice, T>
where
  T: Bits + BitsRequire,
{
  #[inline(always)]
  pub const fn words(fields: usize) -> usize {
    (fields + T::BITS - 1) / T::BITS
  }

  #[inline(always)]
  pub fn available(&self) -> usize {
    self.store.len() * T::BITS
  }

  #[inline(always)]
  pub fn store(&self) -> &[T] {
    self.store
  }

  #[inline(always)]
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

    let mut bitmap = Self {
      store,
      bits,
      used: 0,
    };
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

    let mut bitmap = Self {
      store,
      bits,
      used: 0,
    };
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

  #[inline]
  pub fn set(&mut self, index: usize) -> Result<(), BitmapError> {
    let (word_index, bit_index) = self.position(index)?;
    self.store[word_index] = self.store[word_index].set(bit_index);
    self.used += 1;
    Ok(())
  }

  #[inline]
  pub fn clear(&mut self, index: usize) -> Result<(), BitmapError> {
    let (word_index, bit_index) = self.position(index)?;
    self.store[word_index] = self.store[word_index].clear(bit_index);
    self.used -= 1;
    Ok(())
  }

  #[inline]
  pub fn flip(&mut self, index: usize) -> Result<(), BitmapError> {
    let (wi, bi) = self.position(index)?;
    let old = self.store[wi];
    let new = old.flip(bi);
    match (old == T::zero(), new == T::zero()) {
      (true, false) => self.used += 1,
      (false, true) => self.used -= 1,
      _ => {}
    }
    self.store[wi] = new;
    Ok(())
  }

  #[inline]
  pub fn get(&self, index: usize) -> Result<bool, BitmapError> {
    let (word_index, bit_index) = self.position(index)?;
    Ok(self.store[word_index].get(bit_index))
  }

  pub fn clear_all(&mut self) {
    for word in self.store.iter_mut() {
      *word = T::zero();
    }
    self.used = 0;
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
    self.used = self.bits;
  }

  pub fn find_fs(&self) -> Option<usize> {
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

  pub fn find_fc(&self) -> Option<usize> {
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

  #[inline]
  pub fn is_clear(&self) -> bool {
    self.used == 0
  }

  #[inline]
  pub fn one_clear(&self) -> bool {
    self.used < self.bits
  }
}
