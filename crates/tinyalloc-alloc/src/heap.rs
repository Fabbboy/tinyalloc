use std::{
  alloc::Layout,
  num::NonZeroUsize,
  ptr::NonNull,
  sync::OnceLock,
  thread::{
    self,
    ThreadId,
  },
};

use getset::Getters;
use spin::RwLock;
use tinyalloc_list::List;

use crate::{
  allocation::Allocation,
  arena::ArenaError,
  classes::{
    class_init,
    find_class,
  },
  config::{
    LARGE_SC_LIMIT,
    SIZES,
  },
  large::{
    Large,
    LargeError,
  },
  queue::Queue,
};

#[derive(Debug)]
pub enum HeapError {
  Arena(ArenaError),
  Large(LargeError),
  InvalidSize,
  InvalidPointer,
}

#[derive(Getters)]
pub struct Heap<'mapper> {
  thread: OnceLock<ThreadId>,
  classes: [Queue<'mapper>; SIZES],
  large: List<Large<'mapper>>,
  #[getset(get = "pub")]
  remote: RwLock<List<Allocation<'mapper>>>,
}

impl<'mapper> Heap<'mapper> {
  pub fn new() -> Self {
    let classes: [Queue<'mapper>; SIZES] =
      class_init(|class| Queue::new(class));
    Self {
      thread: OnceLock::new(),
      classes,
      large: List::new(),
      remote: RwLock::new(List::new()),
    }
  }

  pub fn thread(&self) -> ThreadId {
    *self.thread.get_or_init(|| thread::current().id())
  }

  pub fn allocate(
    &mut self,
    layout: Layout,
  ) -> Result<NonNull<[u8]>, HeapError> {
    let size = layout.size();

    if size == 0 {
      return Err(HeapError::InvalidSize);
    }

    if size > LARGE_SC_LIMIT {
      return self.alloc_large(layout);
    }

    self.alloc_small(layout)
  }
  fn alloc_small(
    &mut self,
    layout: Layout,
  ) -> Result<NonNull<[u8]>, HeapError> {
    let class = find_class(layout.size(), layout.align())
      .ok_or(HeapError::InvalidSize)?;
    let queue = &mut self.classes[class.id];

    let ptr = queue
      .allocate()
      .ok_or(HeapError::Arena(ArenaError::Insufficient))?;

    let slice =
      unsafe { core::slice::from_raw_parts_mut(ptr.as_ptr(), layout.size()) };
    NonNull::new(slice as *mut [u8]).ok_or(HeapError::InvalidPointer)
  }

  fn alloc_large(
    &mut self,
    layout: Layout,
  ) -> Result<NonNull<[u8]>, HeapError> {
    let size =
      NonZeroUsize::new(layout.size()).ok_or(HeapError::InvalidSize)?;
    let large_ptr = Large::new(size).map_err(HeapError::Large)?;

    let slice_ptr = unsafe { large_ptr.as_ref() }.user_slice();

    self.large.push(large_ptr);
    Ok(slice_ptr)
  }

  pub fn deallocate(
    &mut self,
    ptr: NonNull<u8>,
    layout: Layout,
  ) -> Result<(), HeapError> {
    let size = layout.size();

    if size == 0 {
      return Err(HeapError::InvalidSize);
    }

    if size > LARGE_SC_LIMIT {
      return self.dealloc_large(ptr);
    }

    self.dealloc_small(ptr, layout)
  }

  fn dealloc_small(
    &mut self,
    ptr: NonNull<u8>,
    layout: Layout,
  ) -> Result<(), HeapError> {
    let class = find_class(layout.size(), layout.align())
      .ok_or(HeapError::InvalidSize)?;

    let queue = &mut self.classes[class.id];
    if queue.deallocate(ptr) {
      Ok(())
    } else {
      Err(HeapError::InvalidPointer)
    }
  }

  fn dealloc_large(&mut self, ptr: NonNull<u8>) -> Result<(), HeapError> {
    let large_nn =
      Large::from_user_ptr(ptr).ok_or(HeapError::InvalidPointer)?;

    if self.large.remove(large_nn) {
      unsafe { core::ptr::drop_in_place(large_nn.as_ptr()) };
      Ok(())
    } else {
      Err(HeapError::InvalidPointer)
    }
  }
}
