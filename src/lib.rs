use std::{
  alloc::{
    GlobalAlloc,
    Layout,
  },
  cell::UnsafeCell,
  ptr::NonNull,
  sync::{
    Mutex,
    OnceLock,
  },
};

use tinyalloc_alloc::heap::Heap;
#[cfg(feature = "ffi")]
mod ffi;
mod init;

thread_local! {
    static LOCAL_HEAP: UnsafeCell<Heap<'static>> = UnsafeCell::new(Heap::new());
}

struct BootstrapHeap {
  heap: UnsafeCell<Heap<'static>>,
  lock: Mutex<()>,
}

unsafe impl Sync for BootstrapHeap {}
unsafe impl Send for BootstrapHeap {}

impl BootstrapHeap {
  fn new() -> Self {
    Self {
      heap: UnsafeCell::new(Heap::new()),
      lock: Mutex::new(()),
    }
  }

  fn with<R>(&self, f: impl FnOnce(&mut Heap<'static>) -> R) -> R {
    let _guard = self.lock.lock().unwrap();
    let heap = unsafe { &mut *self.heap.get() };
    f(heap)
  }
}

static BOOTSTRAP_HEAP: OnceLock<BootstrapHeap> = OnceLock::new();

fn with_heap<R>(f: impl FnOnce(&mut Heap<'static>) -> R) -> R {
  match LOCAL_HEAP.try_with(|heap| heap.get() as *mut Heap<'static>) {
    Ok(ptr) => {
      let heap = unsafe { &mut *ptr };
      f(heap)
    }
    Err(_) => {
      let bootstrap = BOOTSTRAP_HEAP.get_or_init(BootstrapHeap::new);
      bootstrap.with(f)
    }
  }
}

pub struct TinyAlloc;

unsafe impl GlobalAlloc for TinyAlloc {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    with_heap(|heap| {
      let mut mem: NonNull<[u8]> = match heap.allocate(layout) {
        Ok(mem) => mem,
        Err(_) => return std::ptr::null_mut(),
      };

      unsafe { mem.as_mut().as_mut_ptr() }
    })
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    with_heap(|heap| unsafe {
      let _ = heap.deallocate(
        NonNull::new_unchecked(ptr),
        layout,
      );
    })
  }
}
