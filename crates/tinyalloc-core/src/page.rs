use std::ptr::NonNull;

use getset::Getters;

use crate::vm::{
  MapError,
  Mapper,
};

#[derive(Getters)]
pub struct Page<'mapper> {
  mapper: &'mapper dyn Mapper,
  #[getset(get = "pub")]
  ptr: NonNull<[u8]>,
}

impl<'mapper> Page<'mapper> {
  pub fn new(mapper: &'mapper dyn Mapper, size: usize) -> Result<Self, MapError> {
    let ptr = mapper.map(size)?;
    Ok(Self { mapper, ptr })
  }

  pub fn commit(&mut self) -> Result<(), MapError> {
    self.mapper.commit(self.ptr)
  }

  pub fn decommit(&mut self) -> Result<(), MapError> {
    self.mapper.decommit(self.ptr)
  }

  pub fn protect(&mut self) -> Result<(), MapError> {
    self.mapper.protect(self.ptr)
  }

  pub fn unprotect(&mut self) -> Result<(), MapError> {
    self.mapper.unprotect(self.ptr)
  }
}
