use std::ptr::NonNull;

use tinyalloc_bitmap::{
  Bitmap,
  BitmapError,
};
use tinyalloc_list::{
  HasLink,
  Link,
};

use crate::{
  classes::{
    Class,
    Segmentation,
  },
  config::align_slice,
};

pub struct Segment<'mapper> {
  class: &'static Class,
  link: Link<Segment<'mapper>>,
  bitmap: Bitmap<'mapper, usize>,
  user: &'mapper mut [u8],
}

#[derive(Debug)]
pub enum SegmentError {
  InsufficientCapacity { class_id: usize },
  Bitmap(BitmapError),
}

impl From<BitmapError> for SegmentError {
  fn from(err: BitmapError) -> Self {
    SegmentError::Bitmap(err)
  }
}

impl<'mapper> Segment<'mapper> {
  pub fn new(
    class: &'static Class,
    slice: &'mapper mut [u8],
  ) -> Result<NonNull<Self>, SegmentError> {
    let self_size = core::mem::size_of::<Self>();
    let (segment_slice, rest) = slice.split_at_mut(self_size);

    let Segmentation {
      bitmap: bitmap_slice,
      rest: bitmap_rest,
    } = class.segment::<usize>(rest);
    let user_aligned = align_slice(bitmap_rest, class.align.0);
    let object_capacity = user_aligned.len() / class.size.0;
    if object_capacity == 0 {
      return Err(SegmentError::InsufficientCapacity { class_id: class.id });
    }
    let bitmap = Bitmap::zero(bitmap_slice, object_capacity)?;

    let segment_ptr = segment_slice.as_mut_ptr() as *mut Self;
    unsafe {
      core::ptr::write(
        segment_ptr,
        Self {
          class,
          link: Link::new(),
          bitmap,
          user: user_aligned,
        },
      );
      Ok(NonNull::new_unchecked(segment_ptr))
    }
  }

  pub fn is_full(&self) -> bool {
    self.bitmap.find_first_clear().is_none()
  }

  pub fn is_empty(&self) -> bool {
    self.bitmap.is_clear()
  }

  pub fn contains_ptr(&self, ptr: NonNull<u8>) -> bool {
    let user_start = self.user.as_ptr() as *mut u8;
    let user_end = unsafe { user_start.add(self.user.len()) };
    ptr.as_ptr() >= user_start && ptr.as_ptr() < user_end
  }

  fn ptr_from_index(&mut self, bit_index: usize) -> Option<NonNull<u8>> {
    let offset = bit_index * self.class.size.0;
    if offset >= self.user.len() {
      return None;
    }

    let end = match offset.checked_add(self.class.size.0) {
      Some(end) => end,
      None => return None,
    };

    if end > self.user.len() {
      return None;
    }
    let ptr = unsafe { self.user.as_ptr().add(offset) };
    NonNull::new(ptr as *mut u8)
  }

  fn index_from_ptr(&mut self, ptr: NonNull<u8>) -> Option<usize> {
    let user_start = self.user.as_mut_ptr() as *mut u8;
    let user_end = unsafe { user_start.add(self.user.len()) };
    let ptr_addr = ptr.as_ptr();

    if ptr_addr < user_start || ptr_addr >= user_end {
      return None;
    }

    let offset = unsafe { ptr_addr.offset_from(user_start) as usize };
    let object_size = self.class.size.0;
    if offset % object_size != 0 {
      return None;
    }

    Some(offset / object_size)
  }

  pub fn alloc(&mut self) -> Option<NonNull<u8>> {
    let bit_index = self.bitmap.find_first_clear()?;
    self.bitmap.set(bit_index).ok()?;
    self.ptr_from_index(bit_index)
  }

  pub fn dealloc(&mut self, ptr: NonNull<u8>) -> bool {
    let bit_index = match self.index_from_ptr(ptr) {
      Some(index) => index,
      None => return false,
    };
    self.bitmap.clear(bit_index).is_ok()
  }
}

impl<'mapper> HasLink<Segment<'mapper>> for Segment<'mapper> {
  fn link(&self) -> &Link<Segment<'mapper>> {
    &self.link
  }

  fn link_mut(&mut self) -> &mut Link<Segment<'mapper>> {
    &mut self.link
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    classes::CLASSES,
    config::{
      SEGMENT_SIZE,
      SIZES,
    },
  };

  #[test]
  fn segment_largest_class_utilization() {
    let mut buffer = vec![0u8; SEGMENT_SIZE];
    let largest_class = &CLASSES[SIZES - 1];
    let segment_ptr = Segment::new(largest_class, &mut buffer)
      .expect("segment must initialize for largest class");
    let segment = unsafe { segment_ptr.as_ref() };

    let user_space = segment.user.len();
    let object_size = largest_class.size.0;
    let max_objects = user_space / object_size;
    let remainder = user_space % object_size;
    let utilization =
      ((user_space - remainder) as f64 / user_space as f64) * 100.0;

    println!(
      "Largest class: size={}, objects={}, remainder={}, utilization={:.1}%",
      object_size, max_objects, remainder, utilization
    );

    assert!(max_objects >= 2, "Should fit at least 2 large objects");
    assert_eq!(remainder, 30624, "Expected remainder for largest class");
    assert!(
      (utilization - 76.6).abs() < 0.1,
      "Expected ~76.6% utilization"
    );
  }

  #[test]
  fn segment_smallest_class_utilization() {
    let mut buffer = vec![0u8; SEGMENT_SIZE];
    let smallest_class = &CLASSES[0];
    let segment_ptr = Segment::new(smallest_class, &mut buffer)
      .expect("segment must initialize for smallest class");
    let segment = unsafe { segment_ptr.as_ref() };

    let user_space = segment.user.len();
    let object_size = smallest_class.size.0;
    let max_objects = user_space / object_size;
    let remainder = user_space % object_size;

    println!(
      "Smallest class: size={}, objects={}, remainder={}",
      object_size, max_objects, remainder
    );

    assert_eq!(object_size, 8, "First class should be 8 bytes");
    assert_eq!(remainder, 0, "Should have perfect fit for 8-byte objects");
    assert!(max_objects > 16000, "Should fit many small objects");
  }

  #[test]
  fn segment_space_utilization_analysis() {
    let mut perfect_fits = 0;
    let mut worst_utilization = 100.0;
    let mut worst_class = 0;

    println!(
      "Class | Size    | Objects | Remainder | Utilization | Bitmap Words"
    );
    println!(
      "------|---------|---------|-----------|-------------|-------------"
    );

    for (i, class) in CLASSES.iter().enumerate() {
      let mut buffer = vec![0u8; SEGMENT_SIZE];
      let segment_ptr = Segment::new(class, &mut buffer)
        .expect("segment must initialize for class");
      let segment = unsafe { segment_ptr.as_ref() };

      let user_space = segment.user.len();
      let object_size = class.size.0;
      let max_objects = user_space / object_size;
      let remainder = user_space % object_size;
      let utilization =
        ((user_space - remainder) as f64 / user_space as f64) * 100.0;
      let bitmap_words = segment.bitmap.store().len();

      println!(
        "{:5} | {:7} | {:7} | {:9} | {:10.1}% | {:11}",
        i, object_size, max_objects, remainder, utilization, bitmap_words
      );

      if remainder == 0 {
        perfect_fits += 1;
      }

      if utilization < worst_utilization {
        worst_utilization = utilization;
        worst_class = i;
      }

      assert!(remainder < object_size);
      assert!(
        utilization > 60.0,
        "Class {} has poor utilization: {:.1}%",
        i,
        utilization
      );
    }

    println!("\nSummary:");
    println!("Perfect fits: {}/{}", perfect_fits, SIZES);
    println!(
      "Worst utilization: {:.1}% (class {})",
      worst_utilization, worst_class
    );
    println!("Segment size: {} bytes", SEGMENT_SIZE);

    assert!(
      perfect_fits >= 3,
      "Should have at least 3 perfect fit classes"
    );
    assert!(
      worst_utilization > 70.0,
      "Worst case should be > 70% utilization"
    );
  }

  #[test]
  fn segment_alloc_dealloc_basic() {
    let mut buffer = vec![0u8; SEGMENT_SIZE];
    let class = &CLASSES[0];
    let mut segment_ptr =
      Segment::new(class, &mut buffer).expect("segment must initialize");
    let segment = unsafe { segment_ptr.as_mut() };

    let ptr1 = segment.alloc().expect("Should allocate first object");
    assert!(
      !segment.bitmap.is_clear(),
      "Bitmap should not be clear after allocation"
    );

    let ptr2 = segment.alloc().expect("Should allocate second object");
    assert_ne!(ptr1, ptr2, "Should get different pointers");

    assert!(
      segment.dealloc(ptr1),
      "Should successfully deallocate first object"
    );

    assert!(
      segment.dealloc(ptr2),
      "Should successfully deallocate second object"
    );

    assert!(
      segment.bitmap.is_clear(),
      "Bitmap should be clear after all deallocations"
    );

    let ptr3 = segment.alloc().expect("Should be able to reallocate");
    assert_eq!(ptr1, ptr3, "Should reuse first slot");
  }

  #[test]
  fn segment_bitmap_sizing_correctness() {
    for class in CLASSES.iter() {
      let mut buffer = vec![0u8; SEGMENT_SIZE];
      let segment_ptr = Segment::new(class, &mut buffer)
        .expect("segment must initialize for bitmap sizing");
      let segment = unsafe { segment_ptr.as_ref() };

      let max_objects = segment.user.len() / class.size.0;
      let bitmap_words_needed =
        (max_objects + usize::BITS as usize - 1) / usize::BITS as usize;
      let actual_bitmap_words = segment.bitmap.store().len();

      assert!(
        actual_bitmap_words >= bitmap_words_needed,
        "Bitmap too small: need {} words, have {} for class size {}",
        bitmap_words_needed,
        actual_bitmap_words,
        class.size.0
      );

      assert!(
        actual_bitmap_words <= bitmap_words_needed + 16,
        "Bitmap oversized: need {} words, have {} for class size {}",
        bitmap_words_needed,
        actual_bitmap_words,
        class.size.0
      );
    }
  }
}
