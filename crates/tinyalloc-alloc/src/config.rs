use tinyalloc_sys::size::cache_line_size;

pub const SIZES: usize = 84;
pub const ONE: usize = 1;
pub const WORD: usize = core::mem::size_of::<usize>();
pub const MAX_ALIGN: usize = {
  let cache_line = cache_line_size();
  let min_align = WORD * 2;
  if cache_line > min_align {
    cache_line
  } else {
    min_align
  }
};

pub const SHIFT: usize = WORD.trailing_zeros() as usize;
pub const MIN_ALIGN: usize = WORD;
pub const MIN_SIZE: usize = MIN_ALIGN;

pub const ARENA_SHIFT: usize = 23 + SHIFT;
pub const ARENA_INITIAL_SIZE: usize = 1 << ARENA_SHIFT;
pub const ARENA_GROWTH: usize = 2;
pub const ARENA_STEP: usize = 4;
pub const ARENA_LIMIT: usize = 80;

pub const SEGMENT_SHIFT: usize = 16 + SHIFT;
pub const SEGMENT_SIZE: usize = 1 << SEGMENT_SHIFT;

pub const SMALL_SC_LIMIT: usize = 1 << (SHIFT + 5);
pub const MEDIUM_SC_LIMIT: usize = 1 << (SHIFT + 10);
pub const LARGE_SC_LIMIT: usize = 1 << (SHIFT + 15);

pub const SMALL_ALIGN_LIMIT: usize = SMALL_SC_LIMIT / 4;
pub const MEDIUM_ALIGN_LIMIT: usize = MEDIUM_SC_LIMIT / 8;
pub const LARGE_ALIGN_RATIO: usize = 8;

pub const REMOTE_BATCH_SIZE: usize = 32;
pub const REMOTE_CHECK_FREQUENCY: usize = 16;
pub const REMOTE_MAX_BATCH: usize = 64;

pub const FREE_SEGMENT_LIMIT: usize = 12;

pub const fn align_up(size: usize, align: usize) -> usize {
  if align <= 1 {
    return size;
  }
  let mask = align - 1;

  if align & mask == 0 {
    let add = size.saturating_add(mask);
    return add & !mask;
  }

  let add = size.saturating_add(mask);
  (add / align).saturating_mul(align)
}

pub fn align_slice(slice: &mut [u8], align: usize) -> &mut [u8] {
  if align <= 1 {
    return slice;
  }

  let start = slice.as_mut_ptr() as usize;
  let aligned = align_up(start, align);
  let offset = aligned.saturating_sub(start).min(slice.len());
  &mut slice[offset..]
}
