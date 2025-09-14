use core::slice;
use enumset::enum_set;
use std::{
  num::NonZeroUsize,
  ptr::NonNull,
};
use tinyalloc_bitmap::{
  Bitmap,
  numeric::Bits,
};

use tinyalloc_sys::{
  MapError,
  mapper::Protection,
  region::Region,
};

use crate::{
  config::{
    WORD,
    align_up,
  },
  segment::Segment,
};

#[derive(Debug)]
pub enum ArenaError {
  MapError(MapError),
  Insufficient,
  SizeIsZero,
}

pub struct Arena<'mapper> {
  region: Region<'mapper>,
  bitmap: Bitmap<'mapper, usize>,
  user: &'mapper mut [u8],
}

impl<'mapper> Arena<'mapper> {
  pub fn new(size: usize) -> Result<NonNull<Self>, ArenaError> {
    let nonz = NonZeroUsize::new(size).ok_or(ArenaError::SizeIsZero)?;
    let region = Region::new(nonz).map_err(ArenaError::MapError)?;

    let arena_size = core::mem::size_of::<Self>();
    let total_size = align_up(arena_size, WORD);

    if total_size >= nonz.get() {
      return Err(ArenaError::Insufficient);
    }

    let base_ptr = region.as_ptr();
    let full_slice = unsafe { slice::from_raw_parts_mut(base_ptr, nonz.get()) };
    let (arena_slice, rest) = full_slice.split_at_mut(total_size);

    let activation_range = NonNull::new(arena_slice as *mut [u8]).unwrap();
    region
      .partial(
        activation_range,
        enum_set!(Protection::Read | Protection::Write),
      )
      .map_err(ArenaError::MapError)?;

    let aligned_rest =
      crate::config::align_slice(rest, core::mem::align_of::<usize>());
    let segments_possible = aligned_rest.len() / crate::config::SEGMENT_SIZE;
    if segments_possible == 0 {
      return Err(ArenaError::Insufficient);
    }

    let bitmap_bits = segments_possible;
    let bitmap_bytes = usize::bytes(bitmap_bits);
    let bitmap_words = usize::words(bitmap_bits);

    let (bitmap_slice, user_space) = aligned_rest.split_at_mut(bitmap_bytes);
    let bitmap_storage = unsafe {
      core::slice::from_raw_parts_mut(
        bitmap_slice.as_mut_ptr() as *mut usize,
        bitmap_words,
      )
    };
    let bitmap = Bitmap::zero(bitmap_storage);

    let arena = Self {
      region,
      bitmap,
      user: user_space,
    };

    let arena_ptr = base_ptr as *mut Self;
    unsafe {
      core::ptr::write(arena_ptr, arena);
    }

    Ok(NonNull::new(arena_ptr).unwrap())
  }

  pub fn allocate(&self) -> Result<NonNull<Segment<'mapper>>, ArenaError> {
    // find first free block
    // commit partially
    // mark as used
    // return pointer to segment
    todo!()
  }

  pub fn deallocate(&self, segment: NonNull<Segment<'mapper>>) {
    let _ = segment;
    // decommit partially
    // mark as free
    todo!()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::*;
  use tinyalloc_sys::GLOBAL_MAPPER;

  #[test]
  fn test_arena_construction() {
    let arena_result = Arena::new(ARENA_INITIAL_SIZE);
    assert!(arena_result.is_ok());
  }

  #[test]
  fn test_arena_insufficient_space() {
    let arena_result = Arena::new(1);
    assert!(matches!(arena_result, Err(ArenaError::Insufficient)));
  }
}
