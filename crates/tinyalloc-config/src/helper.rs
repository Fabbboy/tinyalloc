use crate::config::WORD;
use tinyalloc_sys::size::cache_line_size;

pub const MAX_ALIGN: usize = {
  let cache_line = cache_line_size();
  let min_align = WORD * 2;
  if cache_line > min_align {
    cache_line
  } else {
    min_align
  }
};

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
