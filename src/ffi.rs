use std::{
  alloc::{
    GlobalAlloc,
    Layout,
  },
  os::raw::c_void,
};

use crate::TinyAlloc;

static MAGIC: u32 = 0xDEADBEEF;

#[repr(C)]
struct Header {
  layout: Layout,
  magic: u32,
}

static GLOBAL_ALLOCATOR: TinyAlloc = TinyAlloc;

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut c_void {
  let header_size = std::mem::size_of::<Header>();
  let total_size = header_size + size;
  let layout = unsafe {
    Layout::from_size_align_unchecked(
      total_size,
      std::mem::align_of::<Header>(),
    )
  };

  let ptr = unsafe { GLOBAL_ALLOCATOR.alloc(layout) };
  if ptr.is_null() {
    return std::ptr::null_mut();
  }

  let header_ptr = ptr as *mut Header;
  unsafe {
    header_ptr.write(Header {
      layout,
      magic: MAGIC,
    });
    (header_ptr.add(1)) as *mut c_void
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(num: usize, size: usize) -> *mut c_void {
  let total_size = num.checked_mul(size);
  if total_size.is_none() {
    return std::ptr::null_mut();
  }
  let total_size = total_size.unwrap();
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
  if ptr.is_null() {
    return;
  }

  let header_ptr = unsafe { (ptr as *mut Header).offset(-1) };
  let header = unsafe { header_ptr.read() };
  if header.magic != MAGIC {
    return;
  }
  unsafe {
    GLOBAL_ALLOCATOR.dealloc(header_ptr as *mut u8, header.layout);
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
  if ptr.is_null() {
    return malloc(size);
  }

  let header_ptr = unsafe { (ptr as *mut Header).offset(-1) };
  let header_ref = unsafe { &*header_ptr };
  if header_ref.magic != MAGIC {
    return std::ptr::null_mut();
  }

  let new_ptr = malloc(size);
  if new_ptr.is_null() {
    return std::ptr::null_mut();
  }

  let available = header_ref
    .layout
    .size()
    .saturating_sub(std::mem::size_of::<Header>());
  let copy_size = std::cmp::min(size, available);
  unsafe {
    std::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
  }
  free(ptr);
  new_ptr
}
