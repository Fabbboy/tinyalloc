use std::ptr::NonNull;

use tinyalloc_list::List;

use crate::{
  classes::Class,
  config::FREE_SEGMENT_LIMIT,
  segment::Segment,
  static_::{
    allocate_segment,
    deallocate_segment,
    segment_from_ptr,
  },
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
      Move::Free => {
        self.free_list.push(segment);
        self.trim_free_segments();
      }
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

  pub fn allocate(&mut self) -> Option<NonNull<u8>> {
    if let Some(mut segment) = self.get_available() {
      if let Some(ptr) = unsafe { segment.as_mut() }.alloc() {
        self.update_segment_state(segment);
        return Some(ptr);
      }
    }

    let mut new_segment = allocate_segment(self.class).ok()?;
    self.add_segment(new_segment);

    let ptr = unsafe { new_segment.as_mut() }.alloc()?;
    self.update_segment_state(new_segment);
    Some(ptr)
  }

  pub fn add_segment(&mut self, segment: NonNull<Segment<'mapper>>) {
    self.free_list.push(segment);
  }

  pub fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
    if let Some(mut segment) = self.find_segment_with_ptr(ptr) {
      if unsafe { segment.as_mut() }.dealloc(ptr) {
        self.update_segment_state(segment);
        return true;
      }
    }
    false
  }

  fn find_segment_with_ptr(
    &self,
    ptr: NonNull<u8>,
  ) -> Option<NonNull<Segment<'mapper>>> {
    segment_from_ptr(ptr).map(|segment| segment.cast())
  }

  fn update_segment_state(&mut self, segment: NonNull<Segment<'mapper>>) {
    let segment_ref = unsafe { segment.as_ref() };

    let new_state = if segment_ref.is_empty() {
      Move::Free
    } else if segment_ref.is_full() {
      Move::Full
    } else {
      Move::Partial
    };

    self.displace(segment, new_state);
  }

  fn trim_free_segments(&mut self) {
    while self.free_list.count() > FREE_SEGMENT_LIMIT {
      let Some(segment) = self.free_list.pop_front() else {
        break;
      };

      if let Err(_) = deallocate_segment(segment.cast()) {
        self.free_list.push(segment);
        break;
      }
    }
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    classes::CLASSES,
    config::FREE_SEGMENT_LIMIT,
    static_::{
      allocate_segment,
      deallocate_segment,
    },
  };

  #[test]
  fn queue_basic_functionality() {
    let class = &CLASSES[0];
    let queue = Queue::new(class);

    assert!(
      !queue.has_available(),
      "New queue should have no available segments"
    );
  }

  #[test]
  fn trims_free_segments_when_threshold_exceeded() {
    let class = &CLASSES[0];
    let mut queue = Queue::new(class);

    for _ in 0..(FREE_SEGMENT_LIMIT + 5) {
      let segment =
        allocate_segment(class).expect("segment allocation should succeed");
      queue.displace(segment.cast(), Move::Free);
    }

    assert!(
      queue.free_list.count() <= FREE_SEGMENT_LIMIT,
      "free list should not exceed configured limit"
    );

    for segment in queue.free_list.drain() {
      let _ = deallocate_segment(segment.cast());
    }
  }
}
