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
    REMOTE_BATCH_SIZE,
    REMOTE_CHECK_FREQUENCY,
    REMOTE_MAX_BATCH,
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
  operations: usize,
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
      operations: 0,
    }
  }

  pub fn thread(&self) -> ThreadId {
    *self.thread.get_or_init(|| thread::current().id())
  }

  pub fn allocate(
    &mut self,
    layout: Layout,
  ) -> Result<NonNull<[u8]>, HeapError> {
    self.operations = self.operations.wrapping_add(1);
    self.free_remote()?;

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
    self.operations = self.operations.wrapping_add(1);
    self.free_remote()?;

    self.deallocate_internal(ptr, layout)
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

  fn free_remote(&mut self) -> Result<(), HeapError> {
    if !self.should_free_remote()? {
      return Ok(());
    }

    self.free_remote_batched()
  }

  fn should_free_remote(&self) -> Result<bool, HeapError> {
    let remote_len = {
      let guard = self.remote.read();
      guard.count()
    };

    if remote_len == 0 {
      return Ok(false);
    }

    let should_process = remote_len >= REMOTE_BATCH_SIZE
      || self.operations % REMOTE_CHECK_FREQUENCY == 0;

    Ok(should_process)
  }

  fn free_remote_batched(&mut self) -> Result<(), HeapError> {
    let mut guard = match self.remote.try_write() {
      Some(guard) => guard,
      None => return Ok(()),
    };

    let mut processed = 0;
    while processed < REMOTE_MAX_BATCH && !guard.is_empty() {
      if let Some(allocation_nn) = guard.pop() {
        let (header_ptr, layout) = self.extract_info(allocation_nn);

        drop(guard);

        self.deallocate_internal(header_ptr, layout)?;

        guard = match self.remote.try_write() {
          Some(guard) => guard,
          None => return Ok(()),
        };

        processed += 1;
      }
    }

    Ok(())
  }

  fn extract_info(
    &self,
    allocation_nn: NonNull<Allocation<'mapper>>,
  ) -> (NonNull<u8>, Layout) {
    let allocation_ref = unsafe { allocation_nn.as_ref() };
    let header_ptr =
      unsafe { NonNull::new_unchecked(allocation_nn.as_ptr() as *mut u8) };
    let layout = allocation_ref.full();
    (header_ptr, layout)
  }

  fn deallocate_internal(
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
}

impl<'mapper> Drop for Heap<'mapper> {
  fn drop(&mut self) {
    for large in self.large.drain() {
      let _ = unsafe { core::ptr::drop_in_place(large.as_ptr()) };
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_empty_remote_list() {
    let heap = Heap::new();
    assert!(!heap.should_free_remote().unwrap());
    assert_eq!(heap.remote.read().count(), 0);
  }

  #[test]
  fn test_operation_counter_increment() {
    let mut heap = Heap::new();
    let layout = Layout::from_size_align(8, 8).unwrap();

    let initial_ops = heap.operations;
    let _ = heap.allocate(layout);
    assert_eq!(heap.operations, initial_ops + 1);
  }

  #[test]
  fn test_should_process_remote_logic() {
    let mut heap = Heap::new();

    assert!(!heap.should_free_remote().unwrap());

    heap.operations = REMOTE_CHECK_FREQUENCY;
    assert!(!heap.should_free_remote().unwrap());
  }
}
