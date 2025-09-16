use std::{
  alloc::{
    GlobalAlloc,
    Layout,
  },
  os::raw::c_void,
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
  user_offset: u16,
  magic: u32,
}

static GLOBAL_ALLOCATOR: TinyAlloc = TinyAlloc;

fn calculate_user_alignment(size: usize) -> usize {
  if size >= FFI_MAX_SIZE_THRESHOLD {
    FFI_MAX_USER_ALIGN
  } else {
    FFI_MIN_USER_ALIGN
  }
}

fn calculate_allocation_layout(size: usize) -> (Layout, u16) {
  let user_align = calculate_user_alignment(size);
  let header_size = std::mem::size_of::<Header>();
  let padding = (user_align - (header_size % user_align)) % user_align;
  let user_offset = header_size + padding;
  let total_size = user_offset + size;
  let alloc_align = std::cmp::max(std::mem::align_of::<Header>(), user_align);

  let layout =
    unsafe { Layout::from_size_align_unchecked(total_size, alloc_align) };

  (layout, user_offset as u16)
}

fn write_header_and_get_user_ptr(
  ptr: *mut u8,
  layout: Layout,
  user_offset: u16,
) -> *mut c_void {
  let header_ptr = ptr as *mut Header;
  unsafe {
    header_ptr.write(Header {
      layout,
      user_offset,
      magic: FFI_MAGIC,
    });

    let user_ptr = ptr.add(user_offset as usize);
    user_ptr as *mut c_void
  }
}

fn find_header(user_ptr: *mut c_void) -> Option<*mut Header> {
  if user_ptr.is_null() {
    return None;
  }

  let ptr = user_ptr as *mut u8;
  let header_size = std::mem::size_of::<Header>();

  for &user_align in &[FFI_MIN_USER_ALIGN, FFI_MAX_USER_ALIGN] {
    let padding = (user_align - (header_size % user_align)) % user_align;
    let expected_offset = header_size + padding;

    let header_ptr = unsafe { ptr.sub(expected_offset) } as *mut Header;
    let header = unsafe { header_ptr.read() };

    if header.magic == FFI_MAGIC
      && header.user_offset as usize == expected_offset
    {
      return Some(header_ptr);
    }
  }

  None
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut c_void {
  if size == 0 {
    return std::ptr::null_mut();
  }

  let (layout, user_offset) = calculate_allocation_layout(size);
  let ptr = unsafe { GLOBAL_ALLOCATOR.alloc(layout) };

  if ptr.is_null() {
    return std::ptr::null_mut();
  }

  write_header_and_get_user_ptr(ptr, layout, user_offset)
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(num: usize, size: usize) -> *mut c_void {
  let total_size = match num.checked_mul(size) {
    Some(size) => size,
    None => return std::ptr::null_mut(),
  };

  let ptr = malloc(total_size);
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

  let header = unsafe { header_ptr.read() };
  unsafe {
    GLOBAL_ALLOCATOR.dealloc(header_ptr as *mut u8, header.layout);
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
  let new_ptr = malloc(size);

  if new_ptr.is_null() {
    return std::ptr::null_mut();
  }

  let available = header
    .layout
    .size()
    .saturating_sub(header.user_offset as usize);
  let copy_size = std::cmp::min(size, available);

  unsafe {
    std::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
  }

  free(ptr);
  new_ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc_usable_size(ptr: *mut c_void) -> usize {
  let header_ptr = match find_header(ptr) {
    Some(ptr) => ptr,
    None => return 0,
  };

  let header = unsafe { header_ptr.read() };
  header
    .layout
    .size()
    .saturating_sub(header.user_offset as usize)
}
