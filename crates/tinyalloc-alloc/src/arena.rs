use std::{
  num::NonZeroUsize,
  ptr::NonNull,
  slice,
};

use enumset::enum_set;
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
  classes::Class,
  config::{
    SEGMENT_SIZE,
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
  segment_count: usize,
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
      segment_count: segments_possible,
    };

    let arena_ptr = base_ptr as *mut Self;
    unsafe {
      core::ptr::write(arena_ptr, arena);
    }

    Ok(NonNull::new(arena_ptr).unwrap())
  }

  pub fn allocate(
    &mut self,
    class: &'static Class,
  ) -> Result<NonNull<Segment<'mapper>>, ArenaError> {
    let free_bit = self.bitmap.find_first_clear();
    let segment_index = match free_bit {
      Some(index) => index,
      None => return Err(ArenaError::Insufficient),
    };

    let segment_offset = segment_index * SEGMENT_SIZE;
    if segment_offset + SEGMENT_SIZE > self.user.len() {
      return Err(ArenaError::Insufficient);
    }

    let user_ptr = self.user.as_mut_ptr();
    let segment_slice = unsafe {
      slice::from_raw_parts_mut(user_ptr.add(segment_offset), SEGMENT_SIZE)
    };

    let segment_range = NonNull::new(segment_slice as *mut [u8]).unwrap();

    self
      .region
      .partial(
        segment_range,
        enum_set!(Protection::Read | Protection::Write),
      )
      .map_err(ArenaError::MapError)?;

    let segment = Segment::new(class, segment_slice);
    let _ = self.bitmap.set(segment_index);

    Ok(segment)
  }

  pub fn deallocate(
    &mut self,
    segment: NonNull<Segment<'mapper>>,
  ) -> Result<(), ArenaError> {
    let segment_ptr = segment.as_ptr() as *mut u8;
    let user_start = self.user.as_ptr() as *mut u8;

    if segment_ptr < user_start {
      return Err(ArenaError::Insufficient);
    }

    let segment_offset =
      unsafe { segment_ptr.offset_from(user_start) } as usize;
    let segment_index = segment_offset / SEGMENT_SIZE;

    if segment_index >= self.segment_count {
      return Err(ArenaError::Insufficient);
    }

    let segment_slice =
      unsafe { slice::from_raw_parts_mut(segment_ptr, SEGMENT_SIZE) };
    let segment_range = NonNull::new(segment_slice as *mut [u8]).unwrap();

    self
      .region
      .partial(segment_range, enum_set!())
      .map_err(ArenaError::MapError)?;
    let _ = self.bitmap.clear(segment_index);

    Ok(())
  }

  pub fn has_space(&self) -> bool {
    self.bitmap.find_first_clear().is_some()
  }

  pub fn user_start(&self) -> *const u8 {
    self.user.as_ptr()
  }

  pub fn user_len(&self) -> usize {
    self.user.len()
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
