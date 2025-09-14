#[cfg(unix)]
use std::num::NonZeroUsize;
#[cfg(unix)]
use std::ptr::NonNull;

#[cfg(unix)]
use crate::mapper::Protection;
use crate::{
  MapError,
  mapper::{
    Mapper,
    MapperRequires,
  },
  size::{
    page_align,
    page_align_slice,
  },
};
#[cfg(unix)]
use enumset::EnumSet;

#[cfg(unix)]
mod unix {
  pub const NULL: *mut libc::c_void = std::ptr::null_mut();
  pub const TRASH: i32 = -1;

  pub const PERM_WRITE: i32 = libc::PROT_WRITE;
  pub const PERM_READ: i32 = libc::PROT_READ;

  pub const PERM_RW: i32 = PERM_READ | PERM_WRITE;
  pub const PERM_NONE: i32 = libc::PROT_NONE;

  pub const DONTNEED: i32 = libc::MADV_DONTNEED;

  pub const MAP_FLAGS: i32 = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;
}

#[derive(Clone, Debug)]
pub struct PosixMapper;

impl MapperRequires for PosixMapper {}

#[cfg(unix)]
impl PosixMapper {
  fn to_prot(prot: EnumSet<Protection>) -> i32 {
    let rf = prot.contains(Protection::Read);
    let wf = prot.contains(Protection::Write);
    match (rf, wf) {
      (true, true) => unix::PERM_RW,
      (true, false) => unix::PERM_READ,
      (false, true) => unix::PERM_WRITE,
      (false, false) => unix::PERM_NONE,
    }
  }
}

#[cfg(unix)]
impl Mapper for PosixMapper {
  fn map(&self, size: NonZeroUsize) -> Result<NonNull<[u8]>, MapError> {
    let aligned_size = page_align(size.get());
    let ptr = unsafe {
      libc::mmap(
        unix::NULL,
        aligned_size,
        unix::PERM_NONE,
        unix::MAP_FLAGS,
        unix::TRASH,
        0,
      )
    };

    if ptr == libc::MAP_FAILED {
      return Err(MapError::OutOfMemory);
    }

    let slice =
      unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, aligned_size) };
    Ok(NonNull::new(slice).unwrap())
  }

  fn unmap(&self, ptr: NonNull<[u8]>) {
    let size = ptr.len();
    unsafe { libc::munmap(ptr.as_ptr() as *mut libc::c_void, size) };
  }

  fn decommit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    let aligned_slice = page_align_slice(ptr);
    let cptr = self.cptr(aligned_slice.as_ptr() as *mut u8);
    let res =
      unsafe { libc::madvise(cptr, aligned_slice.len(), unix::DONTNEED) };
    if res != 0 {
      return Err(MapError::DecommitFailed);
    }
    self.protect(ptr, EnumSet::empty())?;
    Ok(())
  }

  fn protect(
    &self,
    ptr: NonNull<[u8]>,
    prot: EnumSet<Protection>,
  ) -> Result<(), MapError> {
    let prot_flags = Self::to_prot(prot);
    let aligned_slice = page_align_slice(ptr);
    let cptr = self.cptr(aligned_slice.as_ptr() as *mut u8);
    let res = unsafe { libc::mprotect(cptr, aligned_slice.len(), prot_flags) };
    if res != 0 {
      return Err(MapError::ProtectFailed);
    }
    Ok(())
  }
}

#[cfg(not(unix))]
impl Mapper for PosixMapper {}

#[cfg(all(test, unix))]
mod tests {
  use std::num::NonZero;

  use enumset::EnumSet;

  use crate::{
    GLOBAL_MAPPER,
    mapper::Protection,
    posix::{
      PosixMapper,
      unix,
    },
  };

  #[test]
  fn test_to_prot_all_combinations() {
    assert_eq!(PosixMapper::to_prot(EnumSet::empty()), unix::PERM_NONE);
    assert_eq!(
      PosixMapper::to_prot(Protection::Read.into()),
      unix::PERM_READ
    );
    assert_eq!(
      PosixMapper::to_prot(Protection::Write.into()),
      unix::PERM_WRITE
    );
    assert_eq!(PosixMapper::to_prot(EnumSet::all()), unix::PERM_RW);
  }

  #[test]
  fn test_map_and_unmap() {
    let result = GLOBAL_MAPPER.map(NonZero::new(4096).unwrap());
    assert!(result.is_ok());

    let ptr = result.unwrap();
    assert!(ptr.len() >= 4096);

    GLOBAL_MAPPER.unmap(ptr);
  }

  #[test]
  fn test_commit_and_protect() {
    let ptr = GLOBAL_MAPPER.map(NonZero::new(4096).unwrap()).unwrap();

    let commit_result = GLOBAL_MAPPER.protect(ptr, EnumSet::all());
    assert!(commit_result.is_ok());

    let protect_result = GLOBAL_MAPPER.protect(ptr, Protection::Read.into());
    assert!(protect_result.is_ok());

    GLOBAL_MAPPER.unmap(ptr);
  }

  #[test]
  fn test_protect_with_different_permissions() {
    let ptr = GLOBAL_MAPPER.map(NonZero::new(4096).unwrap()).unwrap();

    assert!(GLOBAL_MAPPER.protect(ptr, EnumSet::empty()).is_ok());
    assert!(GLOBAL_MAPPER.protect(ptr, Protection::Read.into()).is_ok());
    assert!(GLOBAL_MAPPER.protect(ptr, Protection::Write.into()).is_ok());
    assert!(GLOBAL_MAPPER.protect(ptr, EnumSet::all()).is_ok());

    GLOBAL_MAPPER.unmap(ptr);
  }

  #[test]
  fn test_decommit() {
    let ptr = GLOBAL_MAPPER.map(NonZero::new(4096).unwrap()).unwrap();
    GLOBAL_MAPPER.protect(ptr, EnumSet::all()).unwrap();

    let decommit_result = GLOBAL_MAPPER.decommit(ptr);
    assert!(decommit_result.is_ok());
    GLOBAL_MAPPER.unmap(ptr);
  }

  #[test]
  fn test_large_allocation() {
    let size = 1024 * 1024;
    let ptr = GLOBAL_MAPPER.map(NonZero::new(size).unwrap()).unwrap();
    assert!(ptr.len() >= size);

    GLOBAL_MAPPER.protect(ptr, EnumSet::all()).unwrap();

    GLOBAL_MAPPER.unmap(ptr);
  }
}
