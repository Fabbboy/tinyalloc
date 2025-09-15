use std::{
  num::NonZeroUsize,
  ptr::NonNull,
};

use getset::Getters;
use tinyalloc_list::{
  HasLink,
  Link,
};
use tinyalloc_sys::{
  MapError,
  region::Region,
};

#[derive(Debug)]
pub enum LargeError {
  MapError(MapError),
  SizeOverflow,
}

#[derive(Getters)]
pub struct Large<'mapper> {
  region: Region<'mapper>,
  pub user: &'mapper mut [u8],
  link: Link<Large<'mapper>>,
}

impl<'mapper> Large<'mapper> {
  pub fn new(size: NonZeroUsize) -> Result<NonNull<Self>, LargeError> {
    let self_size = core::mem::size_of::<Self>();
    let total_size = size
      .get()
      .checked_add(self_size)
      .ok_or(LargeError::SizeOverflow)?;
    let mut region = Region::new(NonZeroUsize::new(total_size).unwrap())
      .map_err(LargeError::MapError)?;
    region.activate().map_err(LargeError::MapError)?;
    let ptr = region.as_ptr();
    let user =
      unsafe { std::slice::from_raw_parts_mut(ptr.add(self_size), size.get()) };

    let large = Self {
      region,
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
}

impl<'mapper> HasLink<Large<'mapper>> for Large<'mapper> {
  fn link(&self) -> &Link<Large<'mapper>> {
    &self.link
  }

  fn link_mut(&mut self) -> &mut Link<Large<'mapper>> {
    &mut self.link
  }
}
