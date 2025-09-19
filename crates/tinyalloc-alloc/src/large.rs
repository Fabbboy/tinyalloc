use std::{
  num::NonZeroUsize,
  ptr::NonNull,
};

use getset::Getters;
use tinyalloc_config::helper::align_up;
use tinyalloc_list::{
  HasLink,
  Link,
};
use tinyalloc_sys::{
  MapError,
  region::Region,
  size::{
    cache_line_size,
    page_align_ptr,
  },
};

#[derive(Debug)]
pub enum LargeError {
  MapError(MapError),
  SizeOverflow,
}

#[derive(Getters)]
pub struct Large {
  _region: Region,
  pub user: &'static mut [u8],
  link: Link<Large>,
}

impl Large {
  pub fn new(size: NonZeroUsize) -> Result<NonNull<Self>, LargeError> {
    let self_size = core::mem::size_of::<Self>();
    let cache_line = cache_line_size();
    let user_offset = align_up(self_size, cache_line);
    let total_size = size
      .get()
      .checked_add(user_offset)
      .ok_or(LargeError::SizeOverflow)?;
    let mut region = Region::new(NonZeroUsize::new(total_size).unwrap())
      .map_err(LargeError::MapError)?;
    region.activate().map_err(LargeError::MapError)?;
    let ptr = region.as_ptr();
    let user = unsafe {
      std::slice::from_raw_parts_mut(ptr.add(user_offset), size.get())
    };

    let large = Self {
      _region: region,
      user,
      link: Link::new(),
    };

    let large_ptr = ptr as *mut Self;
    unsafe { large_ptr.write(large) };

    NonNull::new(large_ptr).ok_or(LargeError::SizeOverflow)
  }

  pub fn user_slice(&self) -> NonNull<[u8]> {
    NonNull::new(self.user as *const [u8] as *mut [u8]).unwrap()
  }

  pub fn contains_ptr(&self, ptr: NonNull<u8>) -> bool {
    let user_start = self.user.as_ptr() as *mut u8;
    let user_end = unsafe { user_start.add(self.user.len()) };
    ptr.as_ptr() >= user_start && ptr.as_ptr() < user_end
  }

  pub fn from_user_ptr(ptr: NonNull<u8>) -> Option<NonNull<Self>> {
    let page_start = page_align_ptr(ptr.as_ptr());
    let large_ptr = page_start as *mut Self;
    let large_nn = NonNull::new(large_ptr)?;

    let large = unsafe { large_nn.as_ref() };
    if large.contains_ptr(ptr) {
      Some(large_nn)
    } else {
      None
    }
  }
}

impl HasLink<Large> for Large {
  fn link(&self) -> &Link<Large> {
    &self.link
  }

  fn link_mut(&mut self) -> &mut Link<Large> {
    &mut self.link
  }
}
