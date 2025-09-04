use std::ptr::{
  self,
  NonNull,
};

use getset::{
  Getters,
  MutGetters,
};
use tinyalloc_sys::{
  page::Page,
  size::{
    page_align,
    page_size,
  },
  vm::{
    MapError,
    Mapper,
  },
};

use crate::SEGMENT_SIZE;

#[repr(C)]
#[derive(Getters, MutGetters)]
pub struct Segment<'mapper> {
  #[getset(get = "pub", get_mut = "pub")]
  next: Option<NonNull<Segment<'mapper>>>,
  page: Page<'mapper>,
  #[getset(get = "pub")]
  capacity: usize,
  #[getset(get = "pub", get_mut = "pub")]
  used: usize,
  #[getset(get = "pub", get_mut = "pub")]
  mapped: usize,
  user: NonNull<[u8]>,
}

impl<'mapper> Segment<'mapper> {
  pub fn new(mapper: &'mapper dyn Mapper) -> Result<NonNull<Self>, MapError> {
    let page_size = page_size();
    let capacity = SEGMENT_SIZE / page_size;
    let mut internal = Page::new(mapper, SEGMENT_SIZE)?;
    let segment_ptr = internal.as_mut().as_ptr() as *mut Segment;
    let user_size = SEGMENT_SIZE - std::mem::size_of::<Segment>();
    let user_ptr = unsafe {
      ptr::slice_from_raw_parts_mut(segment_ptr.add(1) as *mut u8, user_size)
    };

    unsafe {
      segment_ptr.write(Segment {
        next: None,
        page: internal,
        capacity,
        used: 1,
        mapped: capacity,
        user: NonNull::new_unchecked(user_ptr),
      });
      Ok(NonNull::new_unchecked(segment_ptr))
    }
  }

  pub fn expand(&mut self, pages: usize) -> Result<(), MapError> {
    let new_mapped = self.mapped + pages;
    if new_mapped > self.capacity {
      return Err(MapError);
    }

    let page_sz = page_size();
    let segment_start = (self.page.ptr().as_ptr() as *const u8) as usize;

    for page_idx in self.mapped..new_mapped {
      let page_start = page_align(segment_start + (page_idx * page_sz));
      let page_ptr = unsafe {
        NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
          page_start as *mut u8,
          page_sz,
        ))
      };

      self.page.mapper().commit(page_ptr)?;
    }

    self.mapped = new_mapped;
    Ok(())
  }

  pub fn truncate(&mut self, pages: usize) -> Result<(), MapError> {
    if pages > self.mapped {
      return Err(MapError);
    }

    let new_mapped = self.mapped - pages;
    let page_sz = page_size();
    let segment_start = (self.page.ptr().as_ptr() as *const u8) as usize;

    for page_idx in new_mapped..self.mapped {
      let page_start = page_align(segment_start + (page_idx * page_sz));
      let page_ptr = unsafe {
        NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
          page_start as *mut u8,
          page_sz,
        ))
      };

      self.page.mapper().decommit(page_ptr)?;
    }

    self.mapped = new_mapped;
    Ok(())
  }

  pub fn collect(&mut self) -> Result<(), MapError> {
    if self.used < self.mapped {
      let pages_to_free = self.mapped - self.used;
      self.truncate(pages_to_free)?;
    }
    Ok(())
  }

  pub fn drop(segment: NonNull<Self>, recursive: bool) {
    unsafe {
      let segment_ref = segment.as_ref();
      if recursive {
        if let Some(next) = segment_ref.next() {
          Self::drop(*next, true);
        }
      }
      ptr::drop_in_place(segment.as_ptr());
    }
  }
}

impl<'mapper> AsRef<[u8]> for Segment<'mapper> {
  fn as_ref(&self) -> &[u8] {
    unsafe { self.user.as_ref() }
  }
}

impl<'mapper> AsMut<[u8]> for Segment<'mapper> {
  fn as_mut(&mut self) -> &mut [u8] {
    unsafe { self.user.as_mut() }
  }
}

#[cfg(test)]
mod tests {
  use std::mem;

  use super::*;
  #[cfg(unix)]
  use tinyalloc_sys::system::posix::PosixMapper;
  #[cfg(windows)]
  use tinyalloc_sys::system::windows::WindowsMapper;
  use tinyalloc_sys::vm::Mapper;

  #[cfg(unix)]
  static BACKING_MAPPER: PosixMapper = PosixMapper;
  #[cfg(windows)]
  static BACKING_MAPPER: WindowsMapper = WindowsMapper;

  static MAPPER: &dyn Mapper = &BACKING_MAPPER;

  #[test]
  fn test_segment_creation_and_layout() {
    let segment = Segment::new(MAPPER).expect("Failed to create segment");
    let segment_ref = unsafe { segment.as_ref() };

    let expected_capacity = SEGMENT_SIZE / page_size();
    let expected_user_size = SEGMENT_SIZE - mem::size_of::<Segment>();

    assert_eq!(
      *segment_ref.capacity(),
      expected_capacity,
      "Capacity should match segment size divided by page size"
    );
    assert_eq!(*segment_ref.used(), 1, "Used should be initialized to 1");
    assert!(
      segment_ref.next().is_none(),
      "Next pointer should be None for new segment"
    );
    assert_eq!(
      segment_ref.as_ref().len(),
      expected_user_size,
      "User area size should be segment size minus struct size"
    );

    let user_data = segment_ref.as_ref();
    assert!(!user_data.is_empty(), "User data area should not be empty");
    assert!(
      user_data.as_ptr() as usize > segment.as_ptr() as usize,
      "User data should be after segment struct"
    );

    let segment_end = segment.as_ptr() as usize + SEGMENT_SIZE;
    let user_end = user_data.as_ptr() as usize + user_data.len();
    assert_eq!(
      user_end, segment_end,
      "User data should extend to end of segment"
    );

    Segment::drop(segment, false);
  }

  #[test]
  fn test_segment_chain_management() {
    let mut segment1 =
      Segment::new(MAPPER).expect("Failed to create first segment");
    let mut segment2 =
      Segment::new(MAPPER).expect("Failed to create second segment");
    let segment3 =
      Segment::new(MAPPER).expect("Failed to create third segment");

    let segment1_ref = unsafe { segment1.as_mut() };
    segment1_ref.next_mut().replace(segment2);
    assert!(
      segment1_ref.next().is_some(),
      "First segment should link to second"
    );

    let segment2_ref = unsafe { segment2.as_mut() };
    segment2_ref.next_mut().replace(segment3);
    assert!(
      segment2_ref.next().is_some(),
      "Second segment should link to third"
    );

    let segment3_ref = unsafe { segment3.as_ref() };
    assert!(
      segment3_ref.next().is_none(),
      "Third segment should not link to anything"
    );

    assert_ne!(
      segment1, segment2,
      "Segments should have different addresses"
    );
    assert_ne!(
      segment2, segment3,
      "Segments should have different addresses"
    );

    Segment::drop(segment1, true);
  }

  #[test]
  fn test_segment_memory_access_and_mutations() {
    let mut segment = Segment::new(MAPPER).expect("Failed to create segment");
    let segment_ref = unsafe { segment.as_mut() };

    let initial_used = *segment_ref.used();
    *segment_ref.used_mut() = initial_used + 5;
    assert_eq!(
      *segment_ref.used(),
      initial_used + 5,
      "Used count should be mutable"
    );

    let user_data = segment_ref.as_mut();
    let original_len = user_data.len();

    user_data.fill(0xAA);
    assert!(
      user_data.iter().all(|&b| b == 0xAA),
      "All bytes should be set to 0xAA"
    );

    user_data[0] = 0xFF;
    user_data[original_len - 1] = 0xFF;
    assert_eq!(user_data[0], 0xFF, "First byte should be writable");
    assert_eq!(
      user_data[original_len - 1],
      0xFF,
      "Last byte should be writable"
    );

    let pattern = b"TinyAlloc Test Pattern";
    if user_data.len() >= pattern.len() {
      user_data[..pattern.len()].copy_from_slice(pattern);
      assert_eq!(
        &user_data[..pattern.len()],
        pattern,
        "Pattern should be written correctly"
      );
    }

    Segment::drop(segment, false);
  }

  #[test]
  fn test_segment_expand_and_truncate() {
    let mut segment = Segment::new(MAPPER).expect("Failed to create segment");
    let segment_ref = unsafe { segment.as_mut() };

    let initial_mapped = *segment_ref.mapped();
    assert_eq!(*segment_ref.used(), 1);

    segment_ref.truncate(1).expect("Failed to truncate");
    assert_eq!(*segment_ref.mapped(), initial_mapped - 1);

    segment_ref.expand(1).expect("Failed to expand");
    assert_eq!(*segment_ref.mapped(), initial_mapped);

    Segment::drop(segment, false);
  }

  #[test]
  fn test_segment_collect() {
    let mut segment = Segment::new(MAPPER).expect("Failed to create segment");
    let segment_ref = unsafe { segment.as_mut() };

    *segment_ref.used_mut() = 2;
    let initial_mapped = *segment_ref.mapped();

    segment_ref.collect().expect("Failed to collect");
    assert_eq!(*segment_ref.mapped(), 2);
    assert!(*segment_ref.mapped() < initial_mapped);

    Segment::drop(segment, false);
  }
}
