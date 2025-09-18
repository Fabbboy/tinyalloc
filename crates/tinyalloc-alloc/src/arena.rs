use std::{
  cell::UnsafeCell,
  num::NonZeroUsize,
  ptr::NonNull,
  slice,
};

use spin::Mutex;

use enumset::enum_set;
use tinyalloc_array::Array;
use tinyalloc_bitmap::{
  Bitmap,
  BitmapError,
  numeric::Bits,
};

use tinyalloc_sys::{
  MapError,
  mapper::Protection,
  region::Region,
  size::page_size,
};

use crate::{
  classes::Class,
  config::{
    SEGMENT_SIZE,
    WORD,
    align_slice,
    align_up,
  },
  segment::{
    Segment,
    SegmentError,
  },
};

#[derive(Debug)]
pub enum ArenaError {
  MapError(MapError),
  Insufficient,
  SizeIsZero,
  Bitmap(BitmapError),
  Segment(SegmentError),
}

pub const ARENA_CACHE_SIZE: usize = 8;

pub struct Arena<'mapper> {
  region: Region<'mapper>,
  bitmap: UnsafeCell<Bitmap<'mapper, usize>>,
  user: UnsafeCell<&'mapper mut [u8]>,
  segment_count: usize,
  cache: UnsafeCell<Array<usize, ARENA_CACHE_SIZE>>,
  lock: Mutex<()>,
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

    let aligned_rest = align_slice(rest, core::mem::align_of::<usize>());
    let segments_possible = aligned_rest.len() / SEGMENT_SIZE;
    if segments_possible == 0 {
      return Err(ArenaError::Insufficient);
    }

    let bitmap_bytes = usize::bytes(segments_possible);
    if bitmap_bytes >= aligned_rest.len() {
      return Err(ArenaError::Insufficient);
    }

    let (bitmap_region, user_region) = aligned_rest.split_at_mut(bitmap_bytes);
    let user_space = align_slice(user_region, page_size());
    let segment_count = user_space.len() / SEGMENT_SIZE;
    if segment_count == 0 {
      return Err(ArenaError::Insufficient);
    }

    let bitmap_words = usize::words(segment_count);
    let bitmap_bytes = bitmap_words * core::mem::size_of::<usize>();
    if bitmap_bytes > bitmap_region.len() {
      return Err(ArenaError::Insufficient);
    }
    let (bitmap_slice, _) = bitmap_region.split_at_mut(bitmap_bytes);
    let bitmap_storage = unsafe {
      core::slice::from_raw_parts_mut(
        bitmap_slice.as_mut_ptr() as *mut usize,
        bitmap_words,
      )
    };
    let bitmap = Bitmap::zero(bitmap_storage, segment_count)
      .map_err(ArenaError::Bitmap)?;

    let arena = Self {
      region,
      bitmap: UnsafeCell::new(bitmap),
      user: UnsafeCell::new(user_space),
      segment_count,
      cache: UnsafeCell::new(Array::new()),
      lock: Mutex::new(()),
    };

    let arena_ptr = base_ptr as *mut Self;
    unsafe {
      core::ptr::write(arena_ptr, arena);
    }

    Ok(NonNull::new(arena_ptr).unwrap())
  }

  pub fn allocate(
    &self,
    class: &'static Class,
  ) -> Result<NonNull<Segment<'mapper>>, ArenaError> {
    let _guard = self.lock.lock();

    let bitmap = unsafe { &mut *self.bitmap.get() };
    let user = unsafe { &mut *self.user.get() };
    let cache = unsafe { &mut *self.cache.get() };

    let segment_index = if let Some(cached_index) = cache.pop() {
      cached_index
    } else {
      let free_bit = bitmap.find_first_clear();
      match free_bit {
        Some(index) => index,
        None => return Err(ArenaError::Insufficient),
      }
    };

    let segment_offset = segment_index * SEGMENT_SIZE;
    if segment_offset + SEGMENT_SIZE > user.len() {
      return Err(ArenaError::Insufficient);
    }

    let user_ptr = user.as_mut_ptr();
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

    let segment =
      Segment::new(class, segment_slice).map_err(ArenaError::Segment)?;
    let _ = bitmap.set(segment_index);

    Ok(segment)
  }

  pub fn deallocate(
    &self,
    segment: NonNull<Segment<'mapper>>,
  ) -> Result<(), ArenaError> {
    let _guard = self.lock.lock();

    let bitmap = unsafe { &mut *self.bitmap.get() };
    let user = unsafe { &*self.user.get() };
    let cache = unsafe { &mut *self.cache.get() };

    let segment_ptr = segment.as_ptr() as *mut u8;
    let user_start = user.as_ptr() as *mut u8;

    if segment_ptr < user_start {
      return Err(ArenaError::Insufficient);
    }

    let segment_offset =
      unsafe { segment_ptr.offset_from(user_start) } as usize;
    let segment_index = segment_offset / SEGMENT_SIZE;

    if segment_index >= self.segment_count {
      return Err(ArenaError::Insufficient);
    }

    if !bitmap.get(segment_index).unwrap_or(false) {
      return Err(ArenaError::Insufficient);
    }

    let segment_slice =
      unsafe { slice::from_raw_parts_mut(segment_ptr, SEGMENT_SIZE) };
    let segment_range = NonNull::new(segment_slice as *mut [u8]).unwrap();

    self
      .region
      .partial(segment_range, enum_set!())
      .map_err(ArenaError::MapError)?;
    let _ = cache.push(segment_index);
    let _ = bitmap.clear(segment_index);

    Ok(())
  }

  pub fn has_space(&self) -> bool {
    let _guard = self.lock.lock();
    let bitmap = unsafe { &*self.bitmap.get() };
    let cache = unsafe { &*self.cache.get() };
    !cache.is_empty() || bitmap.find_first_clear().is_some()
  }

  pub fn user_start(&self) -> *const u8 {
    let user = unsafe { &*self.user.get() };
    user.as_ptr()
  }

  pub fn user_len(&self) -> usize {
    let user = unsafe { &*self.user.get() };
    user.len()
  }
}

impl<'mapper> Drop for Arena<'mapper> {
  fn drop(&mut self) {
    let _guard = self.lock.lock();
    let bitmap = unsafe { &mut *self.bitmap.get() };

    while let Some(segment_index) = bitmap.find_first_set() {
      let _ = bitmap.clear(segment_index);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::*;

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
