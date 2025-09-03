use std::ptr::NonNull;
use tinyalloc_core::vm::{
  MapError,
  Mapper,
};

pub struct UnixMapper;

#[cfg(all(unix, not(target_os = "macos")))]
impl Mapper for UnixMapper {
  fn map(&self, size: usize) -> Result<NonNull<[u8]>, MapError> {
    _ = size;
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
  fn unprotect(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    _ = ptr;
    return Err(MapError);
  }
}
