use std::{
  alloc::{
    GlobalAlloc,
    Layout,
  },
  cell::UnsafeCell,
  mem,
  ptr::NonNull,
  sync::OnceLock,
  thread::{
    self,
    ThreadId,
  },
};

use getset::CloneGetters;
use spin::Mutex;
use tinyalloc_alloc::{
  config::{align_up, MAX_ALIGN},
  heap::Heap,
};

use crate::init::{
  is_td,
  td_register,
};
#[cfg(feature = "ffi")]
mod ffi;
mod init;

thread_local! {
    static LOCAL_HEAP: UnsafeCell<Heap<'static>> = UnsafeCell::new(Heap::new());
}

#[derive(CloneGetters)]
struct Header {
  #[getset(get_clone = "pub")]
  thread: ThreadId,
  #[getset(get_clone = "pub")]
  heap: NonNull<Heap<'static>>,
}

impl Header {
  fn new(heap: &mut Heap<'static>) -> Self {
    Self {
      thread: heap.thread(),
      heap: NonNull::new(heap as *mut Heap<'static>).unwrap(),
    }
  }

  fn user_ptr(&self) -> *mut u8 {
    let header_addr = self as *const Self as usize;
    let user_addr = align_up(header_addr + mem::size_of::<Self>(), MAX_ALIGN);
    user_addr as *mut u8
  }

  fn from_user(ptr: *mut u8) -> Option<&'static mut Self> {
    if ptr.is_null() {
      return None;
    }

    let user_addr = ptr as usize;
    if user_addr < mem::size_of::<Self>() + MAX_ALIGN {
      return None;
    }

    let max_header_end = user_addr - MAX_ALIGN + 1;
    let header_start = max_header_end - mem::size_of::<Self>();
    let header_ptr = header_start as *mut Self;

    unsafe { header_ptr.as_mut() }
  }

  fn total_size(layout: Layout) -> usize {
    let header_size = mem::size_of::<Self>();
    let user_size = layout.size();
    let padding = MAX_ALIGN - 1;
    header_size + padding + user_size
  }
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
    let _guard = self.lock.lock();
    let heap = unsafe { &mut *self.heap.get() };
    f(heap)
  }
}

static BOOTSTRAP_HEAP: OnceLock<BootstrapHeap> = OnceLock::new();

fn with_heap<R>(f: impl FnOnce(&mut Heap<'static>) -> R) -> R {
  td_register();
  if is_td() {
    let bootstrap = BOOTSTRAP_HEAP.get_or_init(BootstrapHeap::new);
    return bootstrap.with(f);
  }

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
      let total_size = Header::total_size(layout);
      let total_layout = unsafe {
        Layout::from_size_align_unchecked(total_size, layout.align())
      };

      let mut mem: NonNull<[u8]> = match heap.allocate(total_layout) {
        Ok(mem) => mem,
        Err(_) => return std::ptr::null_mut(),
      };

      unsafe {
        let header_ptr = mem.as_mut().as_mut_ptr() as *mut Header;
        let header = Header::new(heap);
        header_ptr.write(header);

        let header_ref = &*header_ptr;
        header_ref.user_ptr()
      }
    })
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    let header = match Header::from_user(ptr) {
      Some(header) => header,
      None => return,
    };

    let heap_ptr = header.heap();
    let header_ptr = header as *mut Header as *mut u8;
    let total_size = Header::total_size(layout);
    let total_layout = unsafe {
      Layout::from_size_align_unchecked(total_size, layout.align())
    };

    if let Some(bootstrap) = BOOTSTRAP_HEAP.get() {
      if heap_ptr.as_ptr() as *const _ == bootstrap.heap.get() {
        bootstrap.with(|heap| unsafe {
          let _ = heap.deallocate(NonNull::new_unchecked(header_ptr), total_layout);
        });
        return;
      }
    }

    if header.thread() == thread::current().id() {
      with_heap(|heap| unsafe {
        let _ = heap.deallocate(NonNull::new_unchecked(header_ptr), total_layout);
      })
    } else {
      unsafe {
        let heap = &mut *heap_ptr.as_ptr();
        let _ = heap.deallocate(NonNull::new_unchecked(header_ptr), total_layout);
      }
    }
  }
}
