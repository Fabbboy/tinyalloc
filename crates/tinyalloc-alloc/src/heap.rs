use std::{
  alloc::Layout,
  ptr::NonNull,
};

use tinyalloc_list::List;
use tinyalloc_sys::mapper::Mapper;

use crate::{
  classes::class_init,
  config::SIZES,
  large::Large,
  queue::Queue,
};

pub struct Heap<'mapper, M>
where
  M: Mapper + ?Sized,
{
  classes: [Queue<'mapper>; SIZES],
  large: List<Large<'mapper, M>>,
}

impl<'mapper, M> Heap<'mapper, M>
where
  M: Mapper + ?Sized,
{
  pub fn new(mapper: &'mapper M) -> Self {
    let classes: [Queue<'mapper>; SIZES] =
      class_init(|class| Queue::new(class));
    Self {
      classes,
      large: List::new(),
    }
  }

  pub fn allocate(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
    todo!()
  }
  pub fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
    todo!()
  }
}
