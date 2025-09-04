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
  size::page_size,
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
  capacity: usize, // number of pages available
  #[getset(get = "pub", get_mut = "pub")]
  used: usize, // number of pages used
  user: NonNull<[u8]>, // pointer to the user data area
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
        user: NonNull::new_unchecked(user_ptr),
      });
      Ok(NonNull::new_unchecked(segment_ptr))
    }
  }

  pub fn drop(segment: NonNull<Self>, recursive: bool) {
    unsafe {
      let segment_ref = segment.as_ref();
      if recursive {
        if let Some(next) = segment_ref.next() {
          Self::drop(*next, true);
        }
      }
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
  fn test_segment_allocation() {
    let segment = Segment::new(MAPPER).expect("Segment allocation failed");
    let segment_ref = unsafe { segment.as_ref() };
    assert_eq!(*segment_ref.capacity(), SEGMENT_SIZE / page_size());
    assert_eq!(*segment_ref.used(), 1);
    assert_eq!(*segment_ref.next(), None);
    assert_eq!(
      segment_ref.as_ref().len(),
      SEGMENT_SIZE - mem::size_of::<Segment>()
    );
    Segment::drop(segment, false);
  }

  #[test]
  fn test_segment_linking() {
    let mut segment1 =
      Segment::new(MAPPER).expect("Segment 1 allocation failed");
    let segment2 = Segment::new(MAPPER).expect("Segment 2 allocation failed");
    let segment1_ref = unsafe { segment1.as_mut() };
    segment1_ref.next_mut().replace(segment2);
    assert_eq!(*segment1_ref.next(), Some(segment2));
    Segment::drop(segment1, true);
  }
}
