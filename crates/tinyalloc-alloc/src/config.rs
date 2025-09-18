pub const SIZES: usize = 32;
pub const ONE: usize = 1;
pub const WORD: usize = core::mem::size_of::<usize>();
pub const MAX_ALIGN: usize = if WORD == 8 { 16 } else { 8 };

pub const SHIFT: usize = WORD.trailing_zeros() as usize;
pub const MIN_ALIGN: usize = WORD;
pub const MIN_SIZE: usize = MIN_ALIGN;

pub const ARENA_SHIFT: usize = 23 + SHIFT;
pub const ARENA_INITIAL_SIZE: usize = 1 << ARENA_SHIFT;
pub const ARENA_GROWTH: usize = 2;
pub const ARENA_STEP: usize = 4;
pub const ARENA_LIMIT: usize = 80;

pub const SEGMENT_SHIFT: usize = 14 + SHIFT;
pub const SEGMENT_SIZE: usize = 1 << SEGMENT_SHIFT;

pub const SMALL_SC_LIMIT: usize = 1 << (SHIFT + 5);
pub const MEDIUM_SC_LIMIT: usize = 1 << (SHIFT + 10);
pub const LARGE_SC_LIMIT: usize = 1 << (SHIFT + 13);

pub const SMALL_ALIGN_LIMIT: usize = SMALL_SC_LIMIT / 4;
pub const MEDIUM_ALIGN_LIMIT: usize = MEDIUM_SC_LIMIT / 8;
pub const LARGE_ALIGN_RATIO: usize = 8;

pub const fn align_up(size: usize, align: usize) -> usize {
  (size + align - 1) & !(align - 1)
}

pub fn align_slice(slice: &mut [u8], align: usize) -> &mut [u8] {
  let start_addr = slice.as_ptr() as usize;
  let aligned_addr = align_up(start_addr, align);
  let offset = aligned_addr - start_addr;

  if offset >= slice.len() {
    return &mut [];
  }

  &mut slice[offset..]
}
