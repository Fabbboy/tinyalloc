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
