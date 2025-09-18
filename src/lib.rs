use std::{
  alloc::{
    GlobalAlloc,
    Layout,
  },
  cell::UnsafeCell,
  num::NonZeroUsize,
  ptr::NonNull,
  sync::OnceLock,
  thread::{
    self,
  },
};

use spin::Mutex;
use tinyalloc_alloc::{
  allocation::{
    Allocation,
    AllocationOwner,
  },
  heap::Heap,
};
use tinyalloc_sys::{
  GLOBAL_MAPPER,
  MapError,
  mapper::Protection,
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

impl TinyAlloc {
  pub fn os_alloc(
    &self,
    size: NonZeroUsize,
  ) -> Result<NonNull<[u8]>, MapError> {
    let mapped = GLOBAL_MAPPER.map(size)?;
    GLOBAL_MAPPER.protect(mapped, Protection::Read | Protection::Write)?;
    Ok(mapped)
  }

  pub fn os_dealloc(&self, ptr: NonNull<[u8]>) {
    GLOBAL_MAPPER.unmap(ptr)
  }

  fn write_allocation(
    &self,
    owner: AllocationOwner<'static>,
    layout: Layout,
    mem: NonNull<[u8]>,
  ) -> *mut u8 {
    unsafe {
      let header_ptr = mem.as_ptr() as *mut Allocation;
      let user_raw_ptr = Allocation::calc_user_ptr(header_ptr);
      let alloc_ptr = header_ptr as *mut u8;

      let allocation = Allocation::new(owner, layout, alloc_ptr, user_raw_ptr);
      header_ptr.write(allocation);
      user_raw_ptr
    }
  }
}

unsafe impl GlobalAlloc for TinyAlloc {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let total_size = Allocation::total_size(layout);
    let total_layout =
      unsafe { Layout::from_size_align_unchecked(total_size, layout.align()) };

    if let Some(ptr) = with_heap(|heap| {
      heap.allocate(total_layout).ok().map(|mem| {
        let heap_ptr = heap as *mut Heap<'static>;
        self.write_allocation(
          AllocationOwner::Heap(heap_ptr),
          total_layout,
          mem,
        )
      })
    }) {
      return ptr;
    }

    let size = match NonZeroUsize::new(total_size) {
      Some(size) => size,
      None => return std::ptr::null_mut(),
    };

    match self.os_alloc(size) {
      Ok(os_mem) => self.write_allocation(
        AllocationOwner::Mapper(os_mem),
        total_layout,
        os_mem,
      ),
      Err(_) => std::ptr::null_mut(),
    }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
    let allocation = match Allocation::from(ptr) {
      Some(allocation) => allocation,
      None => return,
    };

    let allocation_ref = unsafe { &*allocation };

    if let Some(mapped_slice) = unsafe { allocation_ref.map_range() } {
      self.os_dealloc(mapped_slice);
      return;
    }

    let heap = match unsafe { allocation_ref.heap_ptr() } {
      Some(heap) => heap,
      None => return,
    };

    let header_ptr = allocation as *mut u8;
    let total_layout = allocation_ref.full();

    if let Some(bootstrap) = BOOTSTRAP_HEAP.get() {
      if heap as *const _ == bootstrap.heap.get() {
        bootstrap.with(|heap| unsafe {
          let _ =
            heap.deallocate(NonNull::new_unchecked(header_ptr), total_layout);
        });
        return;
      }
    }

    if allocation_ref.thread() == Some(thread::current().id()) {
      with_heap(|heap| unsafe {
        let _ =
          heap.deallocate(NonNull::new_unchecked(header_ptr), total_layout);
      })
    } else {
      let remote_list = heap.remote();
      let mut remote_guard = remote_list.write();
      if let Some(allocation_nn) = NonNull::new(allocation) {
        remote_guard.push(allocation_nn);
      }
    }
  }
}
