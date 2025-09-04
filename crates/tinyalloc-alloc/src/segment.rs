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
  capacity: usize,     // number of pages available
  used: usize,         // number of pages used
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
