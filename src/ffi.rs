use std::{
  alloc::{
    GlobalAlloc,
    Layout,
  },
  os::raw::c_void,
};

use libc::{
  self,
  c_int,
};

use crate::TinyAlloc;
use tinyalloc_alloc::config::{
  FFI_MAGIC,
  FFI_MAX_SIZE_THRESHOLD,
  FFI_MAX_USER_ALIGN,
  FFI_MIN_USER_ALIGN,
};

#[repr(C)]
struct Header {
  layout: Layout,
  user_offset: u32,
  user_align: u32,
  magic: u32,
}

#[repr(C)]
struct Trailer {
  magic: u32,
  user_offset: u32,
}

const TRAILER_SIZE: usize = std::mem::size_of::<Trailer>();

static GLOBAL_ALLOCATOR: TinyAlloc = TinyAlloc;

fn calculate_user_alignment(size: usize) -> usize {
  if size >= FFI_MAX_SIZE_THRESHOLD {
    FFI_MAX_USER_ALIGN
  } else {
    FFI_MIN_USER_ALIGN
  }
}

fn allocate_with_alignment(size: usize, align: usize) -> *mut c_void {
  if align == 0 || !align.is_power_of_two() {
    return std::ptr::null_mut();
  }

  let align = std::cmp::max(align, FFI_MIN_USER_ALIGN);

  if align > u32::MAX as usize {
    return std::ptr::null_mut();
  }

  let header_size = std::mem::size_of::<Header>();
  let trailer_size = TRAILER_SIZE;
  let layout_align = std::mem::align_of::<Header>();

  let total_size = match header_size
    .checked_add(trailer_size)
    .and_then(|v| v.checked_add(size))
    .and_then(|v| v.checked_add(align - 1))
  {
    Some(value) => value,
    None => return std::ptr::null_mut(),
  };

  let layout = match Layout::from_size_align(total_size, layout_align) {
    Ok(layout) => layout,
    Err(_) => return std::ptr::null_mut(),
  };

  let base_ptr = unsafe { GLOBAL_ALLOCATOR.alloc(layout) };

  if base_ptr.is_null() {
    return std::ptr::null_mut();
  }

  let base_addr = base_ptr as usize;
  let trailer_target = match base_addr
    .checked_add(header_size)
    .and_then(|addr| addr.checked_add(trailer_size))
  {
    Some(addr) => addr,
    None => {
      unsafe {
        GLOBAL_ALLOCATOR.dealloc(base_ptr, layout);
      }
      return std::ptr::null_mut();
    }
  };

  let mask = align - 1;
  let remainder = trailer_target & mask;
  let trailer_end = if remainder == 0 {
    trailer_target
  } else {
    match trailer_target.checked_add(align - remainder) {
      Some(val) => val,
      None => {
        unsafe {
          GLOBAL_ALLOCATOR.dealloc(base_ptr, layout);
        }
        return std::ptr::null_mut();
      }
    }
  };

  let user_offset = trailer_end - base_addr;
  let user_align = match u32::try_from(align) {
    Ok(value) => value,
    Err(_) => {
      unsafe {
        GLOBAL_ALLOCATOR.dealloc(base_ptr, layout);
      }
      return std::ptr::null_mut();
    }
  };

  let user_offset = match u32::try_from(user_offset) {
    Ok(value) => value,
    Err(_) => {
      unsafe {
        GLOBAL_ALLOCATOR.dealloc(base_ptr, layout);
      }
      return std::ptr::null_mut();
    }
  };

  write_header_and_get_user_ptr(base_ptr, layout, user_offset, user_align)
}

fn write_header_and_get_user_ptr(
  base_ptr: *mut u8,
  layout: Layout,
  user_offset: u32,
  user_align: u32,
) -> *mut c_void {
  let header_ptr = base_ptr as *mut Header;
  unsafe {
    header_ptr.write(Header {
      layout,
      user_offset,
      user_align,
      magic: FFI_MAGIC,
    });

    let trailer_ptr =
      base_ptr.add(user_offset as usize).sub(TRAILER_SIZE) as *mut Trailer;
    trailer_ptr.write(Trailer {
      magic: FFI_MAGIC,
      user_offset,
    });

    base_ptr.add(user_offset as usize) as *mut c_void
  }
}

fn find_header(user_ptr: *mut c_void) -> Option<*mut Header> {
  if user_ptr.is_null() {
    return None;
  }

  let ptr = user_ptr as *mut u8;
  if (ptr as usize) < TRAILER_SIZE {
    return None;
  }

  let trailer_ptr = unsafe { ptr.sub(TRAILER_SIZE) } as *mut Trailer;
  let trailer = unsafe { &*trailer_ptr };

  if trailer.magic != FFI_MAGIC {
    return None;
  }

  let header_offset = trailer.user_offset as usize;
  let min_offset = std::mem::size_of::<Header>() + TRAILER_SIZE;

  if header_offset < min_offset || (ptr as usize) < header_offset {
    return None;
  }

  let header_ptr = unsafe { ptr.sub(header_offset) } as *mut Header;
  let header = unsafe { &*header_ptr };

  if header.magic != FFI_MAGIC || header.user_offset != trailer.user_offset {
    return None;
  }

  Some(header_ptr)
}

fn normalize_alignment(alignment: usize) -> Option<usize> {
  if alignment == 0 || !alignment.is_power_of_two() {
    return None;
  }

  let pointer_align = std::mem::size_of::<*mut c_void>();
  if alignment < pointer_align {
    return None;
  }

  Some(alignment)
}

fn page_size() -> usize {
  unsafe {
    let page = libc::sysconf(libc::_SC_PAGESIZE);
    if page > 0 { page as usize } else { 4096 }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut c_void {
  let size = size.max(1);
  let align = calculate_user_alignment(size);
  allocate_with_alignment(size, align)
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(num: usize, size: usize) -> *mut c_void {
  let total_size = match num.checked_mul(size) {
    Some(size) => size,
    None => return std::ptr::null_mut(),
  };

  let align = calculate_user_alignment(total_size);
  let ptr = allocate_with_alignment(total_size, align);
  if !ptr.is_null() {
    unsafe {
      std::ptr::write_bytes(ptr, 0, total_size);
    }
  }

  ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn free(ptr: *mut c_void) {
  let header_ptr = match find_header(ptr) {
    Some(ptr) => ptr,
    None => return,
  };

  let header = unsafe { &*header_ptr };
  let user_ptr = ptr as *mut u8;
  let base_ptr = unsafe { user_ptr.sub(header.user_offset as usize) };
  unsafe {
    GLOBAL_ALLOCATOR.dealloc(base_ptr, header.layout);
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
  if ptr.is_null() {
    return malloc(size);
  }

  if size == 0 {
    free(ptr);
    return std::ptr::null_mut();
  }

  let header_ptr = match find_header(ptr) {
    Some(ptr) => ptr,
    None => return std::ptr::null_mut(),
  };
  let header = unsafe { &*header_ptr };
  let user_ptr = ptr as *mut u8;
  let base_ptr = unsafe { user_ptr.sub(header.user_offset as usize) };
  let available = header
    .layout
    .size()
    .saturating_sub(header.user_offset as usize);

  if size <= available {
    return ptr;
  }

  let new_ptr = allocate_with_alignment(size, header.user_align as usize);

  if new_ptr.is_null() {
    return std::ptr::null_mut();
  }

  let copy_size = std::cmp::min(size, available);

  unsafe {
    std::ptr::copy_nonoverlapping(
      ptr as *const u8,
      new_ptr as *mut u8,
      copy_size,
    );
  }

  unsafe {
    GLOBAL_ALLOCATOR.dealloc(base_ptr, header.layout);
  }
  new_ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc_usable_size(ptr: *mut c_void) -> usize {
  let header_ptr = match find_header(ptr) {
    Some(ptr) => ptr,
    None => return 0,
  };

  let header = unsafe { &*header_ptr };
  header
    .layout
    .size()
    .saturating_sub(header.user_offset as usize)
}

#[unsafe(no_mangle)]
pub extern "C" fn aligned_alloc(alignment: usize, size: usize) -> *mut c_void {
  let alignment = match normalize_alignment(alignment) {
    Some(align) => align,
    None => return std::ptr::null_mut(),
  };

  if size % alignment != 0 {
    return std::ptr::null_mut();
  }

  allocate_with_alignment(size, alignment)
}

#[unsafe(no_mangle)]
pub extern "C" fn posix_memalign(
  memptr: *mut *mut c_void,
  alignment: usize,
  size: usize,
) -> c_int {
  if memptr.is_null() {
    return libc::EINVAL;
  }

  let alignment = match normalize_alignment(alignment) {
    Some(align) => align,
    None => return libc::EINVAL,
  };

  let ptr = allocate_with_alignment(size, alignment);

  if ptr.is_null() {
    libc::ENOMEM
  } else {
    unsafe {
      *memptr = ptr;
    }
    0
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn memalign(alignment: usize, size: usize) -> *mut c_void {
  let alignment = match normalize_alignment(alignment) {
    Some(align) => align,
    None => return std::ptr::null_mut(),
  };

  let request = size.max(1);
  allocate_with_alignment(request, alignment)
}

#[unsafe(no_mangle)]
pub extern "C" fn valloc(size: usize) -> *mut c_void {
  let page = page_size();
  let request = size.max(page);
  allocate_with_alignment(request, page)
}

#[unsafe(no_mangle)]
pub extern "C" fn pvalloc(size: usize) -> *mut c_void {
  let page = page_size();
  let request = if size == 0 {
    page
  } else {
    match size.checked_add(page - 1) {
      Some(sum) => sum & !(page - 1),
      None => return std::ptr::null_mut(),
    }
  };

  allocate_with_alignment(request, page)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn fill_pattern(buf: &mut [u8]) {
    for (idx, byte) in buf.iter_mut().enumerate() {
      *byte = (idx & 0xFF) as u8;
    }
  }

  fn assert_prefix(buf: &[u8], expected_len: usize) {
    for idx in 0..expected_len {
      assert_eq!(buf[idx], (idx & 0xFF) as u8, "mismatch at {idx}");
    }
  }

  unsafe fn scenario_realloc_grows_preserves_contents() {
    let initial = 1024;
    let ptr = malloc(initial);
    assert!(!ptr.is_null());

    let slice =
      unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, initial) };
    fill_pattern(slice);

    let bigger = realloc(ptr, initial * 4);
    assert!(!bigger.is_null());

    let new_slice =
      unsafe { std::slice::from_raw_parts(bigger as *const u8, initial) };
    assert_prefix(new_slice, initial);

    free(bigger);
  }

  unsafe fn scenario_realloc_shrinks_in_place_or_copies_prefix() {
    let initial = 4096;
    let ptr = malloc(initial);
    assert!(!ptr.is_null());

    let slice =
      unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, initial) };
    fill_pattern(slice);

    let smaller = realloc(ptr, initial / 4);
    assert!(!smaller.is_null());

    let new_slice =
      unsafe { std::slice::from_raw_parts(smaller as *const u8, initial / 4) };
    assert_prefix(new_slice, initial / 4);

    free(smaller);
  }

  unsafe fn scenario_aligned_alloc_respects_alignment_and_reallocs() {
    let align = 64;
    let size = 512;
    let ptr = aligned_alloc(align, size);
    assert!(!ptr.is_null());
    assert_eq!((ptr as usize) % align, 0);

    let slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, size) };
    fill_pattern(slice);

    let bigger = realloc(ptr, size * 2);
    assert!(!bigger.is_null());
    assert_eq!((bigger as usize) % align, 0);

    let prefix =
      unsafe { std::slice::from_raw_parts(bigger as *const u8, size) };
    assert_prefix(prefix, size);

    free(bigger);
  }

  unsafe fn scenario_posix_memalign_validates_alignment() {
    let mut out: *mut c_void = std::ptr::null_mut();
    let err = posix_memalign(&mut out, 3, 128);
    assert_eq!(err, libc::EINVAL);
    assert!(out.is_null());

    let ok = posix_memalign(&mut out, 128, 256);
    assert_eq!(ok, 0);
    assert!(!out.is_null());
    assert_eq!((out as usize) % 128, 0);
    free(out);
  }

  unsafe fn scenario_valloc_and_pvalloc_return_page_aligned_memory() {
    let page = page_size();

    let ptr = valloc(1);
    assert!(!ptr.is_null());
    assert_eq!((ptr as usize) % page, 0);
    free(ptr);

    let pv = pvalloc(page / 2);
    assert!(!pv.is_null());
    assert_eq!((pv as usize) % page, 0);
    assert!(malloc_usable_size(pv) >= page);
    free(pv);
  }

  #[test]
  fn ffi_regression_suite() {
    unsafe {
      scenario_realloc_grows_preserves_contents();
      scenario_realloc_shrinks_in_place_or_copies_prefix();
      scenario_aligned_alloc_respects_alignment_and_reallocs();
      scenario_posix_memalign_validates_alignment();
      scenario_valloc_and_pvalloc_return_page_aligned_memory();
    }
  }
}
