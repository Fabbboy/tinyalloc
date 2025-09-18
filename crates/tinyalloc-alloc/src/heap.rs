use std::{
  alloc::Layout,
  num::NonZeroUsize,
  ptr::NonNull,
  thread::{
    self,
    ThreadId,
  },
};

use getset::CloneGetters;
use spin::RwLock;
use tinyalloc_list::List;

use crate::{
  allocation::Allocation, arena::ArenaError, classes::{
    class_init,
    find_class,
  }, config::{
    LARGE_SC_LIMIT,
    SIZES,
  }, large::{
    Large,
    LargeError,
  }, queue::Queue
};

#[derive(Debug)]
pub enum HeapError {
  Arena(ArenaError),
  Large(LargeError),
  InvalidSize,
  InvalidPointer,
}

#[derive(CloneGetters)]
pub struct Heap<'mapper> {
  #[getset(get_clone = "pub")]
  thread: ThreadId,
  classes: [Queue<'mapper>; SIZES],
  large: List<Large<'mapper>>,
  _remote: RwLock<List<Allocation<'mapper>>>,
}

impl<'mapper> Heap<'mapper> {
  pub fn new() -> Self {
    let classes: [Queue<'mapper>; SIZES] =
      class_init(|class| Queue::new(class));
    Self {
      thread: thread::current().id(),
      classes,
      large: List::new(),
      _remote: RwLock::new(List::new()),
    }
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

    debug_assert!(
      ptr.as_ptr() as usize % layout.align() == 0,
      "Allocated pointer {:p} does not meet alignment requirement {}",
      ptr.as_ptr(),
      layout.align()
    );

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

    debug_assert!(
      slice_ptr.as_ptr() as *const u8 as usize % layout.align() == 0,
      "Large allocated pointer {:p} does not meet alignment requirement {}",
      slice_ptr.as_ptr(),
      layout.align()
    );

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
