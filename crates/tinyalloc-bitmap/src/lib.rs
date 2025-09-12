use getset::Getters;

use crate::numeric::{Bits, BitsRequire};

pub mod numeric;
#[cfg(test)]
pub mod tests;

#[derive(Debug)]
pub enum BitmapError {
    InsufficientSize { have: usize, need: usize },
    OutOfBounds { index: usize, size: usize },
}

#[derive(Debug, Getters)]
pub struct Bitmap<'bitmap, T>
where
    T: Bits + BitsRequire,
{
    #[getset(get = "pub")]
    store: &'bitmap mut [T],
    #[getset(get = "pub")]
    bits: usize,
}

impl<'bitmap, T> Bitmap<'bitmap, T>
where
    T: Bits + BitsRequire,
{
    pub const fn words(fields: usize) -> usize {
        (fields + T::BITS - 1) / T::BITS
    }

    pub const fn available(store: &'bitmap [T]) -> usize {
        store.len() * T::BITS
    }

    const fn position(&self, index: usize) -> Result<(usize, usize), BitmapError> {
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

    pub const fn within(store: &'bitmap mut [T], fields: usize) -> Result<Self, BitmapError> {
        let total_bits = store.len() * T::BITS;
        if fields > total_bits {
            return Err(BitmapError::InsufficientSize {
                have: total_bits,
                need: fields,
            });
        }
        Ok(Self {
            store,
            bits: fields,
        })
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
}
