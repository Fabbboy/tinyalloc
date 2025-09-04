use std::ptr::NonNull;

use tinyalloc_sys::{
  page::Page,
  vm::Mapper,
};

use crate::queue::Queue;

// Philosophy: syscalls are DAMN slow so DON'T ALLOCATE ANYTHING UNTIL WE ACTUALLY APPEND TO THE VECTOR
// Resizes are bigger in order to prevent often resizes
// Instead of unmapping decommit unused pages after a threshold
// if another threshold is reached, we unmap the whole thing

pub struct MappedVector<'mapper, T> {
  data: Option<NonNull<T>>,
  backing: Queue<Page<'mapper>>,
}

impl<'mapper, T> MappedVector<'mapper, T> {
  pub fn new(mapper: &'mapper dyn Mapper) -> Self {
    Self {
      data: None,
      backing: Queue::new(),
    }
  }
}
