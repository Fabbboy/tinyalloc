use std::ptr::NonNull;

use tinyalloc_list::List;

use crate::{
  classes::Class,
  config::QUEUE_THRESHOLD,
  segment::Segment,
  static_::{
    allocate_segment,
    deallocate_segment,
    segment_from_ptr,
  },
};

#[derive(PartialEq, Clone, Default)]
pub enum Position {
  #[default]
  Free,
  Full,
}

pub struct Queue {
  class: &'static Class,
  free_list: List<Segment>,
  full_list: List<Segment>,
}

impl Queue {
  pub fn new(class: &'static Class) -> Queue {
    Queue {
      class,
      free_list: List::new(),
      full_list: List::new(),
    }
  }

  pub fn displace(&mut self, mut segment: NonNull<Segment>, mv: Position) {
    let segment_ref = unsafe { segment.as_mut() };
    let current_position = segment_ref.current().clone();
    if current_position == mv {
      return;
    }

    match current_position {
      Position::Free => {
        let _ = self.free_list.remove(segment);
      }
      Position::Full => {
        let _ = self.full_list.remove(segment);
      }
    }

    match mv {
      Position::Free => {
        self.free_list.push(segment);
        segment_ref.set_current(Position::Free);
      }
      Position::Full => {
        self.full_list.push(segment);
        segment_ref.set_current(Position::Full);
      }
    }
  }

  pub fn has_available(&self) -> bool {
    self.free_list.head().is_some()
  }

  pub fn get_available(&mut self) -> Option<NonNull<Segment>> {
    self.free_list.pop()
  }

  pub fn allocate(&mut self) -> Option<NonNull<u8>> {
    if let Some(mut segment) = self.get_available() {
      if let Some(ptr) = unsafe { segment.as_mut() }.alloc() {
        self.update_state(segment);
        return Some(ptr);
      }
    }

    let mut new_segment = allocate_segment(self.class).ok()?;
    self.add_segment(new_segment);

    let ptr = unsafe { new_segment.as_mut() }.alloc()?;
    self.update_state(new_segment);
    Some(ptr)
  }

  pub fn add_segment(&mut self, segment: NonNull<Segment>) {
    self.free_list.push(segment);
  }

  pub fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
    let segment = match self.segment_from_ptr(ptr) {
      Some(mut segment) => unsafe { segment.as_mut() },
      None => return false,
    };

    if !segment.dealloc(ptr) {
      return false;
    }

    if segment.is_empty() && self.free_list.count() > QUEUE_THRESHOLD {
      let segment_ptr = NonNull::from(segment);
      let _ = self.free_list.remove(segment_ptr);
      let _ = deallocate_segment(segment_ptr.cast());
    } else {
      self.update_state(NonNull::from(segment));
    }

    true
  }

  fn segment_from_ptr(&self, ptr: NonNull<u8>) -> Option<NonNull<Segment>> {
    segment_from_ptr(ptr).map(|segment| segment.cast())
  }

  fn update_state(&mut self, segment: NonNull<Segment>) {
    let segment_ref = unsafe { segment.as_ref() };

    let new_state = if segment_ref.is_full() {
      Position::Full
    } else {
      Position::Free
    };

    self.displace(segment, new_state);
  }
}

impl Drop for Queue {
  fn drop(&mut self) {
    for segment in self.free_list.drain() {
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
  use crate::classes::CLASSES;

  #[test]
  fn queue_basic_functionality() {
    let class = &CLASSES[0];
    let queue = Queue::new(class);

    assert!(
      !queue.has_available(),
      "New queue should have no available segments"
    );
  }
}
