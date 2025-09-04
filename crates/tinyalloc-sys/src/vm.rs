use std::ptr::NonNull;

#[derive(Debug)]
pub struct MapError;

pub trait Mapper
where
  Self: Sync + Send + 'static,
{
  fn map(
    &self,
    size: usize,
    committed: bool,
  ) -> Result<NonNull<[u8]>, MapError> {
    _ = size;
    _ = committed;
    return Err(MapError);
  }
  fn unmap(&self, ptr: NonNull<[u8]>) {
    _ = ptr;
  }
  fn commit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    _ = ptr;
    return Err(MapError);
  }
  fn decommit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    _ = ptr;
    return Err(MapError);
  }
  fn protect(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    _ = ptr;
    return Err(MapError);
  }
}
