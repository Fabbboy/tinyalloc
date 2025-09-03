use std::ptr::NonNull;

use enumset::{
  EnumSet,
  EnumSetType,
};
use getset::Getters;

use crate::vm::{
  MapError,
  Mapper,
};

#[derive(EnumSetType, Debug)]
pub enum PageFlag {
  Mapped,
  Committed,
  Protected,
}

#[derive(Getters)]
pub struct Page<'mapper> {
  mapper: &'mapper dyn Mapper,
  #[getset(get = "pub")]
  ptr: NonNull<[u8]>,
  #[getset(get = "pub")]
  flags: EnumSet<PageFlag>,
}

impl<'mapper> Page<'mapper> {
  pub fn new(mapper: &'mapper dyn Mapper, size: usize) -> Result<Self, MapError> {
    let ptr = mapper.map(size)?;
    Ok(Self {
      mapper,
      ptr,
      flags: PageFlag::Mapped | PageFlag::Committed,
    })
  }

  pub fn commit(&mut self) -> Result<(), MapError> {
    self.mapper.commit(self.ptr)?;
    self.flags |= PageFlag::Committed;
    self.flags -= PageFlag::Protected;
    Ok(())
  }

  pub fn decommit(&mut self) -> Result<(), MapError> {
    self.mapper.decommit(self.ptr)?;
    self.flags -= PageFlag::Committed;
    Ok(())
  }

  pub fn protect(&mut self) -> Result<(), MapError> {
    self.mapper.protect(self.ptr)?;
    self.flags |= PageFlag::Protected;
    Ok(())
  }

  pub fn is_mapped(&self) -> bool {
    self.flags.contains(PageFlag::Mapped)
  }

  pub fn is_committed(&self) -> bool {
    self.flags.contains(PageFlag::Committed)
  }

  pub fn is_protected(&self) -> bool {
    self.flags.contains(PageFlag::Protected)
  }
}

impl<'mapper> Drop for Page<'mapper> {
  fn drop(&mut self) {
    if self.is_mapped() {
      self.mapper.unmap(self.ptr);
    }
  }
}

impl<'mapper> AsRef<[u8]> for Page<'mapper> {
  fn as_ref(&self) -> &[u8] {
    unsafe { self.ptr.as_ref() }
  }
}

impl<'mapper> AsMut<[u8]> for Page<'mapper> {
  fn as_mut(&mut self) -> &mut [u8] {
    unsafe { self.ptr.as_mut() }
  }
}
