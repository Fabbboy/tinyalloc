use getset::Getters;
use thiserror::Error;

use crate::numeric::{Bits, BitsRequire};

pub mod numeric;

#[derive(Debug, Error)]
pub enum BitmapError {
    #[error("Bitmap size insufficient: have {have} bits, need {need} bits")]
    InsufficientSize { have: usize, need: usize },
    #[error("Bitmap index {index} out of bounds (size {size})")]
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
            bits: total_bits,
        })
    }
}
