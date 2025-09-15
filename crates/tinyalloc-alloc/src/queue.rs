use std::ptr::NonNull;

use tinyalloc_list::List;

use crate::{
  classes::Class,
  segment::Segment,
};

pub enum Move {
  Free,
  Partial,
  Full,
}

pub struct Queue<'mapper> {
  class: &'static Class,
  free_list: List<Segment<'mapper>>,
  partial_list: List<Segment<'mapper>>,
  full_list: List<Segment<'mapper>>,
}

impl<'mapper> Queue<'mapper> {
  pub fn new(class: &'static Class) -> Queue<'mapper> {
    Queue {
      class,
      free_list: List::new(),
      partial_list: List::new(),
      full_list: List::new(),
    }
  }

  pub fn displace(&mut self, segment: NonNull<Segment<'mapper>>, mv: Move) {
    let _ = self.free_list.remove(segment)
      || self.partial_list.remove(segment)
      || self.full_list.remove(segment);

    match mv {
      Move::Free => self.free_list.push(segment),
      Move::Partial => self.partial_list.push(segment),
      Move::Full => self.full_list.push(segment),
    }
  }

  pub fn has_available(&self) -> bool {
    let free_available = self.free_list.head().is_some();
    let partial_available = self.partial_list.head().is_some();
    free_available || partial_available
  }

  pub fn get_available(&mut self) -> Option<NonNull<Segment<'mapper>>> {
    self.free_list.pop().or_else(|| self.partial_list.pop())
  }

  pub fn add_segment(&mut self, segment: NonNull<Segment<'mapper>>) {
    self.free_list.push(segment);
  }
}

impl<'mapper> Drop for Queue<'mapper> {
  fn drop(&mut self) {
    for segment in self.free_list.drain() {
      let _ = segment;
    }
    for segment in self.partial_list.drain() {
      let _ = segment;
    }
    for segment in self.full_list.drain() {
      let _ = segment;
    }
  }
}
