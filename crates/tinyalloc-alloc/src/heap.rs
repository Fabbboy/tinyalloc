use std::{
  alloc::Layout,
  ptr::NonNull,
};

use tinyalloc_list::List;

use crate::{
  classes::class_init,
  config::SIZES,
  large::Large,
  queue::Queue,
};

pub struct Heap<'mapper> {
  classes: [Queue<'mapper>; SIZES],
  large: List<Large<'mapper>>,
}

impl<'mapper> Heap<'mapper> {
  pub fn new() -> Self {
    let classes: [Queue<'mapper>; SIZES] =
      class_init(|class| Queue::new(class));
    Self {
      classes,
      large: List::new(),
    }
  }

  pub fn allocate(&mut self, _layout: Layout) -> Option<NonNull<[u8]>> {
    todo!()
  }
  pub fn deallocate(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
    todo!()
  }
}
