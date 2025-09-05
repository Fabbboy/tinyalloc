use std::{
  ptr::{
    self,
    NonNull,
  },
  slice,
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

#[derive(Getters, MutGetters)]
pub struct Segment<'mapper> {
  #[getset(get = "pub", get_mut = "pub")]
  next: Option<NonNull<Segment<'mapper>>>,
  page: Page<'mapper>,
  #[getset(get = "pub")]
  capacity: usize,
  #[getset(get = "pub")]
  used: usize,
  #[getset(get = "pub")]
  mapped: usize,
  user: NonNull<[u8]>,
}

impl<'mapper> Segment<'mapper> {
  pub fn new(mapper: &'mapper dyn Mapper) -> Result<NonNull<Self>, MapError> {
    let page_size = page_size();
    let capacity = SEGMENT_SIZE / page_size;
    let mut internal = Page::new(mapper, SEGMENT_SIZE, false)?;
    let page_sz = page_size;
    let base_ptr = unsafe { (*internal.ptr().as_ptr()).as_mut_ptr() };
    let first_page_ptr = unsafe {
      NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(base_ptr, page_sz))
    };
    internal.mapper().commit(first_page_ptr)?;

    let segment_ptr = base_ptr as *mut Segment;
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
        mapped: 1,
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

  pub fn manage(&mut self) -> Result<(), MapError> {
    if self.used < self.mapped {
      let pages_to_free = self.mapped - self.used;
      self.truncate(pages_to_free)?;
    } else if self.used > self.mapped {
      let pages_to_add = self.used - self.mapped;
      self.expand(pages_to_add)?;
    }
    Ok(())
  }

  /*pub fn drop(segment: NonNull<Self>, recursive: bool) {
    unsafe {
      let segment_ref = segment.as_ref();
      if recursive {
        if let Some(next) = segment_ref.next() {
          Self::drop(*next, true);
        }
      }
      ptr::drop_in_place(segment.as_ptr());
    }
  }*/

  pub unsafe fn drop(segment: NonNull<Self>) {
    unsafe {
      ptr::drop_in_place(segment.as_ptr());
    }
  }

  pub unsafe fn drop_all(segment: NonNull<Self>) {
    let mut current = Some(segment);
    while let Some(seg) = current {
      unsafe {
        let seg_ref = seg.as_ref();
        current = seg_ref.next;
        Self::drop(seg);
      }
    }
  }
}

impl<'mapper> Segment<'mapper> {
  pub fn as_slice(&self) -> Option<&[u8]> {
    if self.mapped > 0 {
      let len = self.mapped * page_size() - std::mem::size_of::<Segment>();
      if len <= self.user.len() {
        // Safety: `user` points to the beginning of the user-accessible memory
        // for this segment, and `len` is bounded by the mapped pages.
        Some(unsafe {
          slice::from_raw_parts(self.user.as_ptr() as *const u8, len)
        })
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn as_slice_mut(&mut self) -> Option<&mut [u8]> {
    if self.mapped > 0 {
      let len = self.mapped * page_size() - std::mem::size_of::<Segment>();
      if len <= self.user.len() {
        // Safety: same reasoning as `as_slice` but for mutable access.
        Some(unsafe {
          slice::from_raw_parts_mut(self.user.as_ptr() as *mut u8, len)
        })
      } else {
        None
      }
    } else {
      None
    }
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
    let expected_user_size = page_size() - mem::size_of::<Segment>();

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
      segment_ref
        .as_slice()
        .expect("segment should expose mapped slice")
        .len(),
      expected_user_size,
      "Initial committed user area should be one page minus struct size"
    );

    let user_data = segment_ref
      .as_slice()
      .expect("segment should expose mapped slice");
    assert!(!user_data.is_empty(), "User data area should not be empty");
    assert!(
      user_data.as_ptr() as usize > segment.as_ptr() as usize,
      "User data should be after segment struct"
    );

    let segment_end = segment.as_ptr() as usize + SEGMENT_SIZE;
    let user_end = user_data.as_ptr() as usize + user_data.len();
    assert!(
      user_end <= segment_end,
      "User data should not exceed segment bounds",
    );

    unsafe {
      Segment::drop(segment);
    }
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

    unsafe {
      Segment::drop_all(segment1);
    }
  }

  #[test]
  fn test_segment_memory_access_and_mutations() {
    let mut segment = Segment::new(MAPPER).expect("Failed to create segment");
    let segment_ref = unsafe { segment.as_mut() };

    let initial_used = *segment_ref.used();
    segment_ref.used = initial_used + 5;
    assert_eq!(
      *segment_ref.used(),
      initial_used + 5,
      "Used count should be mutable"
    );

    segment_ref
      .expand(segment_ref.capacity() - 1)
      .expect("Failed to expand for test");

    let user_data = segment_ref
      .as_slice_mut()
      .expect("segment should expose mapped mutable slice");
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

    unsafe {
      Segment::drop(segment);
    }
  }

  #[test]
  fn test_segment_expand_and_truncate() {
    let mut segment = Segment::new(MAPPER).expect("Failed to create segment");
    let segment_ref = unsafe { segment.as_mut() };

    let initial_mapped = *segment_ref.mapped();
    assert_eq!(*segment_ref.used(), 1);
    assert_eq!(initial_mapped, 1);

    segment_ref.expand(5).expect("Failed to expand");
    assert_eq!(*segment_ref.mapped(), initial_mapped + 5);

    segment_ref.truncate(3).expect("Failed to truncate");
    assert_eq!(*segment_ref.mapped(), initial_mapped + 2);

    unsafe {
      Segment::drop(segment);
    }
  }

  #[test]
  fn test_segment_collect() {
    let mut segment = Segment::new(MAPPER).expect("Failed to create segment");
    let segment_ref = unsafe { segment.as_mut() };

    segment_ref.used = 2;
    segment_ref.manage().expect("Failed to collect");
    assert_eq!(*segment_ref.mapped(), 2);

    segment_ref.used = 1;
    segment_ref.manage().expect("Failed to collect");
    assert_eq!(*segment_ref.mapped(), 1);

    unsafe {
      Segment::drop(segment);
    }
  }
}
