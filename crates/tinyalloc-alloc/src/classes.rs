use std::array;

use tinyalloc_bitmap::numeric::Bits;

use crate::config::{
  LARGE_ALIGN_RATIO,
  LARGE_SC_LIMIT,
  MEDIUM_ALIGN_LIMIT,
  MEDIUM_SC_LIMIT,
  MIN_ALIGN,
  MIN_SIZE,
  SIZES,
  SMALL_ALIGN_LIMIT,
  SMALL_SC_LIMIT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Align(pub usize);

pub struct Segmentation<'mapper, B>
where
  B: Bits,
{
  pub bitmap: &'mapper mut [B],
  pub rest: &'mapper mut [u8],
}

#[derive(Debug, Clone, Copy, PartialEq)]
//Cannot use getset as getset does NOT produce const getters
pub struct Class {
  pub size: Size,
  pub align: Align,
  pub id: usize,
}

impl Class {
  pub const fn new(size: usize, align: usize, id: usize) -> Self {
    Self {
      size: Size(size),
      align: Align(align),
      id,
    }
  }

  pub fn segment<'mapper, B>(
    &self,
    heap: &'mapper mut [u8],
  ) -> Segmentation<'mapper, B>
  where
    B: Bits,
  {
    let objects_per_heap = heap.len() / self.size.0;
    let bitmap_bits = objects_per_heap;
    let bitmap_bytes = B::bytes(bitmap_bits);
    let bitmap_words = B::words(bitmap_bits);

    let (bitmap_slice, rest) = heap.split_at_mut(bitmap_bytes);
    let bitmap = unsafe {
      core::slice::from_raw_parts_mut(
        bitmap_slice.as_mut_ptr() as *mut B,
        bitmap_words,
      )
    };

    Segmentation { bitmap, rest }
  }
}

const fn size_to_align(size: usize) -> usize {
  if size <= SMALL_ALIGN_LIMIT {
    MIN_ALIGN
  } else if size <= MEDIUM_ALIGN_LIMIT {
    SMALL_ALIGN_LIMIT
  } else if size <= LARGE_SC_LIMIT {
    MEDIUM_ALIGN_LIMIT
  } else {
    size / LARGE_ALIGN_RATIO
  }
}

const fn classes() -> [Class; SIZES] {
  let mut classes = [Class::new(0, 0, 0); SIZES];
  let mut i = 0;
  let mut size = MIN_SIZE;

  while i < SIZES {
    let align = size_to_align(size);
    classes[i] = Class::new(size, align, i);

    if size < SMALL_SC_LIMIT {
      size += align;
    } else if size < MEDIUM_SC_LIMIT {
      size += align * 2;
    } else if size < LARGE_SC_LIMIT {
      size += align * 4;
    } else {
      size *= 2;
    }

    i += 1;
  }
  classes
}

pub static CLASSES: [Class; SIZES] = classes();

#[inline(always)]
pub const fn find_class(size: usize) -> Option<&'static Class> {
  if size == 0 {
    return None;
  }

  let mut i = 0;
  while i < SIZES {
    if size <= CLASSES[i].size.0 {
      return Some(&CLASSES[i]);
    }
    i += 1;
  }
  None
}

pub fn class_init<T>(f: impl Fn(&'static Class) -> T) -> [T; SIZES] {
  array::from_fn(|i| f(&CLASSES[i]))
}
