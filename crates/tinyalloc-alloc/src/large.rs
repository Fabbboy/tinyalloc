use std::num::NonZeroUsize;

use tinyalloc_list::{
  HasLink,
  Link,
};
use tinyalloc_sys::{
  MapError,
  mapper::Mapper,
  region::Region,
};

#[derive(Debug)]
pub enum LargeError {
  MapError(MapError),
  SizeOverflow,
}

pub struct Large<'mapper, M>
where
  M: Mapper + ?Sized,
{
  region: Region<'mapper, M>,
  user: &'mapper mut [u8],
  link: Link<Large<'mapper, M>>,
}

impl<'mapper, M> Large<'mapper, M>
where
  M: Mapper + ?Sized,
{
  pub fn new(
    size: NonZeroUsize,
    mapper: &'mapper M,
  ) -> Result<Self, LargeError> {
    let self_size = core::mem::size_of::<Self>();
    let total_size = size
      .get()
      .checked_add(self_size)
      .ok_or(LargeError::SizeOverflow)?;
    let mut region =
      Region::new(mapper, NonZeroUsize::new(total_size).unwrap())
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

impl<'mapper, M> HasLink<Large<'mapper, M>> for Large<'mapper, M>
where
  M: Mapper + ?Sized,
{
  fn link(&self) -> &Link<Large<'mapper, M>> {
    &self.link
  }

  fn link_mut(&mut self) -> &mut Link<Large<'mapper, M>> {
    &mut self.link
  }
}
