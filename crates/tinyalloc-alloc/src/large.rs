use std::num::NonZeroUsize;

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

pub struct Large<'mapper> {
  region: Region<'mapper>,
  user: &'mapper mut [u8],
  link: Link<Large<'mapper>>,
}

impl<'mapper> Large<'mapper> {
  pub fn new(size: NonZeroUsize) -> Result<Self, LargeError> {
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
    Ok(Self {
      region,
      user,
      link: Link::new(),
    })
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
