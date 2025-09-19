use std::ptr::NonNull;

use tinyalloc_config::{
  classes::Class,
  config::QUEUE_THRESHOLD,
  metric,
};

#[cfg(feature = "metrics")]
use tinyalloc_config::metrics::MetricId;
use tinyalloc_list::List;

use crate::{ 
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
  Partial,
  Full,
}

pub struct Queue {
  class: &'static Class,
  free_list: List<Segment>,
  partial_list: List<Segment>,
  full_list: List<Segment>,
}

impl Queue {
  pub fn new(class: &'static Class) -> Queue {
    Queue {
      class,
      free_list: List::new(),
      partial_list: List::new(),
      full_list: List::new(),
    }
  }

  pub fn displace(&mut self, mut segment: NonNull<Segment>, mv: Position) {
    metric!(MetricId::QueueSegmentDisplace);

    let segment_ref = unsafe { segment.as_mut() };
    let current_position = segment_ref.current().clone();
    if current_position == mv {
      return;
    }

    match current_position {
      Position::Free => {
        let _ = self.free_list.remove(segment);
      }
      Position::Partial => {
        let _ = self.partial_list.remove(segment);
      }
      Position::Full => {
        let _ = self.full_list.remove(segment);
      }
    }

    match mv {
      Position::Free => {
        metric!(MetricId::SegmentStateTransitionPartialToFree);
        self.free_list.push(segment);
        segment_ref.set_current(Position::Free);
      }
      Position::Partial => {
        metric!(MetricId::SegmentStateTransitionFreeToPartial);
        self.partial_list.push(segment);
        segment_ref.set_current(Position::Partial);
      }
      Position::Full => {
        metric!(MetricId::SegmentStateTransitionPartialToFull);
        self.full_list.push(segment);
        segment_ref.set_current(Position::Full);
      }
    }
  }

  pub fn has_available(&self) -> bool {
    self.free_list.head().is_some() || self.partial_list.head().is_some()
  }

  pub fn get_available(&mut self) -> Option<NonNull<Segment>> {
    metric!(MetricId::QueueGetAvailable);

    if let Some(segment) = self.free_list.pop() {
      metric!(MetricId::QueueGetAvailableFromFree);
      Some(segment)
    } else if let Some(segment) = self.partial_list.pop() {
      metric!(MetricId::QueueGetAvailableFromPartial);
      Some(segment)
    } else {
      metric!(MetricId::QueueGetAvailableNone);
      None
    }
  }

  pub fn allocate(&mut self) -> Option<NonNull<u8>> {
    if let Some(mut segment) = self.get_available() {
      metric!(MetricId::SegmentAlloc);
      if let Some(ptr) = unsafe { segment.as_mut() }.alloc() {
        metric!(MetricId::SegmentAllocSuccess);
        self.update_state(segment);
        return Some(ptr);
      } else {
        metric!(MetricId::SegmentAllocFail);
      }
    }

    metric!(MetricId::QueueNewSegmentCreated);
    let mut new_segment = allocate_segment(self.class).ok()?;
    self.add_segment(new_segment);

    metric!(MetricId::SegmentAlloc);
    let ptr = unsafe { new_segment.as_mut() }.alloc()?;
    metric!(MetricId::SegmentAllocSuccess);
    self.update_state(new_segment);
    Some(ptr)
  }

  pub fn add_segment(&mut self, segment: NonNull<Segment>) {
    metric!(MetricId::QueueAddSegment);
    self.free_list.push(segment);
  }

  pub fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
    metric!(MetricId::SegmentPtrLookup);
    let segment = match self.segment_from_ptr(ptr) {
      Some(mut segment) => {
        metric!(MetricId::SegmentPtrLookupSuccess);
        unsafe { segment.as_mut() }
      }
      None => {
        metric!(MetricId::SegmentPtrLookupFail);
        return false;
      }
    };

    metric!(MetricId::SegmentDealloc);
    if !segment.dealloc(ptr) {
      metric!(MetricId::SegmentDeallocFail);
      return false;
    }
    metric!(MetricId::SegmentDeallocSuccess);

    if segment.is_empty() && self.free_list.count() > QUEUE_THRESHOLD {
      metric!(MetricId::QueueTrimFreeSegments);
      metric!(MetricId::QueueTrimSegmentsRemoved);
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
    } else if segment_ref.is_empty() {
      Position::Free
    } else {
      Position::Partial
    };

    self.displace(segment, new_state);
  }
}

impl Drop for Queue {
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
  use tinyalloc_config::classes::CLASSES;

use super::*; 

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
