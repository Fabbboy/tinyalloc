use std::ptr::NonNull;

use tinyalloc_sys::vm::Mapper;

use crate::segment::Segment;

pub struct Heap<'mapper> {
  segments: Option<NonNull<Segment<'mapper>>>,
  mapper: &'mapper dyn Mapper,
}

impl<'mapper> Heap<'mapper> {
  pub fn new(mapper: &'mapper dyn Mapper) -> Self {
    Self {
      segments: None,
      mapper,
    }
  }
}

impl<'mapper> Drop for Heap<'mapper> {
  fn drop(&mut self) {
    if let Some(segment) = self.segments {
      Segment::drop_all(segment);
    }
  }
}
