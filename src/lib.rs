use std::{
  alloc::{
    GlobalAlloc,
    Layout,
  },
  cell::UnsafeCell,
  ptr::NonNull,
  sync::Once,
};

use tinyalloc_alloc::heap::Heap;

thread_local! {
    static GLOBAL_HEAP: UnsafeCell<Heap<'static>> = UnsafeCell::new(Heap::new());
}

pub struct TinyAlloc;

unsafe impl GlobalAlloc for TinyAlloc {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    GLOBAL_HEAP.with(|heap| {
      let heap = unsafe { &mut *heap.get() };
      let mut mem: NonNull<[u8]> = match heap.allocate(layout) {
        Ok(mem) => mem,
        Err(_) => return std::ptr::null_mut(),
      };

      unsafe { mem.as_mut().as_mut_ptr() }
    })
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    GLOBAL_HEAP.with(|heap| {
      let heap = unsafe { &mut *heap.get() };
      unsafe {
        let _ = heap.deallocate(
          NonNull::new_unchecked(
            core::slice::from_raw_parts_mut(ptr, layout.size()).as_mut_ptr(),
          ),
          layout,
        );
      }
    })
  }
}
