use std::ptr::NonNull;

use tinyalloc_sys::vm::Mapper;

use crate::segment::Segment;

pub struct Heap<'mapper> {
  full_list: Option<NonNull<Segment<'mapper>>>,
  partial_list: Option<NonNull<Segment<'mapper>>>,
  free_list: Option<NonNull<Segment<'mapper>>>,
  mapper: &'mapper dyn Mapper,
}

impl<'mapper> Heap<'mapper> {
  pub fn new(mapper: &'mapper dyn Mapper) -> Self {
    Self {
      full_list: None,
      partial_list: None,
      free_list: None,
      mapper,
    }
  }
}

impl<'mapper> Drop for Heap<'mapper> {
  fn drop(&mut self) {
    if let Some(segment) = self.full_list {
      unsafe {
        Segment::drop_all(segment);
      }
    }
    if let Some(segment) = self.partial_list {
      unsafe {
        Segment::drop_all(segment);
      }
    }
    if let Some(segment) = self.free_list {
      unsafe {
        Segment::drop_all(segment);
      }
    }
  }
}
